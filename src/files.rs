//! Akses file atas perintah user: browser file, lampiran (`/attach`, `@path`). Tidak ada baca otomatis.

use std::{
    env, fs,
    io::Read,
    path::{Path, PathBuf},
};

use ratatui::crossterm::event::KeyCode;

/// Batas baca satu file dan total lampiran per pesan (teks saja).
pub const MAX_FILE: usize = 100_000;
pub const MAX_TOTAL: usize = 300_000;
const MAX_ENTRIES: usize = 5_000;
const MAX_LISTING: usize = 300;

/// File atau folder yang dilampirkan user ke pesan berikutnya.
pub struct Attachment {
    pub path: PathBuf,
    /// Teks pendek di layar, mis. "main.rs (12.3 KB)".
    pub label: String,
    /// Blok yang disambung ke pesan user waktu dikirim ke model.
    pub block: String,
    pub size: usize,
}

/// "~" dan "~/x" jadi folder home; jalur relatif dihitung dari folder kerja.
pub fn expand(raw: &str) -> PathBuf {
    let raw = raw.trim().trim_matches(|c: char| c == '"' || c == '\'');
    let home = || env::var_os("HOME").map(PathBuf::from).unwrap_or_else(|| PathBuf::from("/"));
    let path = if raw == "~" {
        home()
    } else if let Some(rest) = raw.strip_prefix("~/") {
        home().join(rest)
    } else {
        PathBuf::from(raw)
    };
    if path.is_absolute() {
        path
    } else {
        env::current_dir().unwrap_or_default().join(path)
    }
}

pub fn human_size(bytes: u64) -> String {
    const UNITS: [&str; 4] = ["B", "KB", "MB", "GB"];
    let mut value = bytes as f64;
    let mut unit = 0;
    while value >= 1024.0 && unit < UNITS.len() - 1 {
        value /= 1024.0;
        unit += 1;
    }
    if unit == 0 {
        format!("{bytes} B")
    } else {
        format!("{value:.1} {}", UNITS[unit])
    }
}

/// Baca file teks (atau daftar isi folder) jadi lampiran. File biner ditolak.
pub fn read_attachment(path: &Path) -> Result<Attachment, String> {
    let shown = path.display().to_string();
    let meta = fs::metadata(path).map_err(|e| format!("{shown}: {e}"))?;
    let name = path.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_else(|| shown.clone());

    if meta.is_dir() {
        let read = fs::read_dir(path).map_err(|e| format!("{shown}: {e}"))?;
        let mut names: Vec<String> = read
            .filter_map(|e| e.ok())
            .map(|e| {
                let is_dir = e.metadata().map(|m| m.is_dir()).unwrap_or(false);
                let n = e.file_name().to_string_lossy().to_string();
                if is_dir {
                    format!("{n}/")
                } else {
                    n
                }
            })
            .collect();
        names.sort();
        let total = names.len();
        names.truncate(MAX_LISTING);
        let mut listing = names.join("\n");
        if total > MAX_LISTING {
            listing.push_str(&format!("\n... ({} lainnya)", total - MAX_LISTING));
        }
        let block = format!("\n\n[Lampiran dari user: isi folder {shown}]\n```\n{listing}\n```");
        return Ok(Attachment { path: path.to_path_buf(), label: format!("{name}/ ({total} item)"), size: block.len(), block });
    }

    let mut bytes = Vec::new();
    fs::File::open(path)
        .and_then(|file| file.take(MAX_FILE as u64 + 1).read_to_end(&mut bytes))
        .map_err(|e| format!("{shown}: {e}"))?;
    if bytes.contains(&0) {
        return Err(format!("{name} bukan file teks (biner), nggak bisa dilampirkan"));
    }
    let truncated = bytes.len() > MAX_FILE;
    bytes.truncate(MAX_FILE);
    let text = String::from_utf8_lossy(&bytes);
    let note = if truncated {
        format!(" (dipotong: {} pertama dari {})", human_size(MAX_FILE as u64), human_size(meta.len()))
    } else {
        String::new()
    };
    let block = format!("\n\n[Lampiran dari user: {shown}{note}]\n```\n{text}\n```");
    Ok(Attachment { path: path.to_path_buf(), label: format!("{name} ({})", human_size(meta.len())), size: block.len(), block })
}

pub struct Entry {
    pub name: String,
    pub is_dir: bool,
    pub size: u64,
}

/// Hasil menekan tombol di browser.
pub enum Action {
    None,
    Close,
    /// Lampirkan `path`; `stay` = browser tetap terbuka buat memilih lagi.
    Attach { path: PathBuf, stay: bool },
}

/// Browser file satu folder, dengan filter ketik-langsung. Hanya membaca daftar nama; isi file
/// baru dibaca kalau user melampirkannya.
pub struct Browser {
    pub dir: PathBuf,
    entries: Vec<Entry>,
    pub filter: String,
    pub selected: usize,
    pub show_hidden: bool,
    pub error: Option<String>,
    /// Umur popup dalam tick, untuk animasi buka.
    pub age: u8,
}

impl Browser {
    pub fn new(dir: PathBuf) -> Self {
        let mut browser =
            Self { dir, entries: Vec::new(), filter: String::new(), selected: 0, show_hidden: false, error: None, age: 0 };
        browser.reload();
        browser
    }

    fn reload(&mut self) {
        self.selected = 0;
        self.filter.clear();
        match fs::read_dir(&self.dir) {
            Ok(read) => {
                let mut entries: Vec<Entry> = read
                    .filter_map(|e| e.ok())
                    .take(MAX_ENTRIES)
                    .map(|e| {
                        let meta = fs::metadata(e.path()).ok();
                        Entry {
                            name: e.file_name().to_string_lossy().to_string(),
                            is_dir: meta.as_ref().map(|m| m.is_dir()).unwrap_or(false),
                            size: meta.map(|m| m.len()).unwrap_or(0),
                        }
                    })
                    .collect();
                entries.sort_by(|a, b| {
                    b.is_dir.cmp(&a.is_dir).then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
                });
                self.entries = entries;
            }
            Err(e) => {
                self.entries.clear();
                self.error = Some(format!("{}: {e}", self.dir.display()));
            }
        }
    }

    /// Pindah folder; kalau folder tujuan tidak bisa dibaca, tetap di sini dan tampilkan alasannya.
    fn go_to(&mut self, dir: PathBuf) {
        match fs::read_dir(&dir) {
            Ok(_) => {
                self.dir = dir;
                self.error = None;
                self.reload();
            }
            Err(e) => self.error = Some(format!("{}: {e}", dir.display())),
        }
    }

    fn go_up(&mut self) {
        if let Some(parent) = self.dir.parent() {
            let parent = parent.to_path_buf();
            self.go_to(parent);
        }
    }

    /// Entri yang lolos filter dan aturan file tersembunyi.
    pub fn visible(&self) -> Vec<&Entry> {
        let q = self.filter.to_lowercase();
        self.entries
            .iter()
            .filter(|e| (self.show_hidden || !e.name.starts_with('.')) && (q.is_empty() || e.name.to_lowercase().contains(&q)))
            .collect()
    }

    fn current(&self) -> Option<(String, bool)> {
        self.visible().get(self.selected).map(|e| (e.name.clone(), e.is_dir))
    }

    fn move_by(&mut self, delta: isize) {
        let last = self.visible().len().saturating_sub(1) as isize;
        self.selected = (self.selected as isize + delta).clamp(0, last) as usize;
    }

    pub fn handle(&mut self, code: KeyCode, ctrl: bool) -> Action {
        match code {
            KeyCode::Esc => return Action::Close,
            KeyCode::Up => self.move_by(-1),
            KeyCode::Down => self.move_by(1),
            KeyCode::PageUp => self.move_by(-10),
            KeyCode::PageDown => self.move_by(10),
            KeyCode::Left => self.go_up(),
            KeyCode::Backspace => {
                if self.filter.pop().is_some() {
                    self.selected = 0;
                } else {
                    self.go_up();
                }
            }
            KeyCode::Enter | KeyCode::Right => {
                if let Some((name, is_dir)) = self.current() {
                    if is_dir {
                        let next = self.dir.join(name);
                        self.go_to(next);
                    } else if code == KeyCode::Enter {
                        return Action::Attach { path: self.dir.join(name), stay: false };
                    }
                }
            }
            KeyCode::Tab => {
                if let Some((name, _)) = self.current() {
                    return Action::Attach { path: self.dir.join(name), stay: true };
                }
            }
            KeyCode::Char('a') if ctrl => return Action::Attach { path: self.dir.clone(), stay: true },
            KeyCode::Char('t') if ctrl => {
                self.show_hidden = !self.show_hidden;
                self.selected = 0;
            }
            KeyCode::Char('g') if ctrl => {
                let home = env::var_os("HOME").map(PathBuf::from).unwrap_or_else(|| PathBuf::from("/"));
                self.go_to(home);
            }
            KeyCode::Char('r') if ctrl => self.go_to(PathBuf::from("/")),
            _ if ctrl => {}
            KeyCode::Char(c) => {
                self.filter.push(c);
                self.selected = 0;
            }
            _ => {}
        }
        Action::None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn human_size_picks_a_unit() {
        assert_eq!(human_size(512), "512 B");
        assert_eq!(human_size(2048), "2.0 KB");
        assert_eq!(human_size(5 * 1024 * 1024), "5.0 MB");
    }

    #[test]
    fn relative_paths_are_made_absolute() {
        assert!(expand("catatan.txt").is_absolute());
        assert_eq!(expand("/etc/hosts"), PathBuf::from("/etc/hosts"));
    }
}
