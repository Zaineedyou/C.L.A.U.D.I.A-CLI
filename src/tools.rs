//! Alat baca file untuk model (read-only). Alat cuma ditawarkan ke model pada giliran yang user minta
//! secara eksplisit (lihat `Access` dan `asks_for_files`); model tidak bisa menulis atau menghapus apa pun.

use std::{fs, io::Read, path::Path};

use serde_json::{json, Value};

use crate::files;

const READ_CAP: u64 = 2_000_000;
const MAX_OUT: usize = 60_000;
const MAX_LIST: usize = 300;
const MAX_HITS: usize = 100;
const MAX_VISITED: usize = 20_000;
const MAX_DEPTH: usize = 8;
const SKIP_DIRS: [&str; 2] = [".git", "node_modules"];

/// Ditambahkan ke system prompt hanya pada giliran yang alat file ditawarkan.
pub const TOOL_PROMPT: &str = "\n\nKamu punya akses baca (read-only) ke file di perangkat user lewat alat list_dir, read_file, \
find_files, dan grep. Pakai alat itu HANYA kalau user secara eksplisit menyuruhmu melihat, membaca, atau \
mencari file/folder di pesan terbarunya. Jangan menjelajah sendiri, jangan membuka file yang tidak disebut atau \
tidak perlu, dan kalau user belum menyebut lokasinya, tanya dulu. Baca seperlunya saja.";

/// Seberapa bebas model boleh membaca file.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Access {
    /// Alat ditawarkan hanya kalau pesan user jelas meminta soal file.
    Auto,
    /// Alat selalu ditawarkan (model tetap diminta menunggu perintah user).
    Always,
    /// Alat tidak pernah ditawarkan.
    Never,
}

impl Access {
    pub fn parse(text: &str) -> Option<Self> {
        match text.trim().to_lowercase().as_str() {
            "auto" | "otomatis" => Some(Self::Auto),
            "on" | "selalu" | "always" => Some(Self::Always),
            "off" | "mati" | "never" => Some(Self::Never),
            _ => None,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Auto => "otomatis (kalau diminta)",
            Self::Always => "selalu tersedia",
            Self::Never => "mati",
        }
    }
}

const ACTIONS: [&str; 22] = [
    "baca", "bacain", "lihat", "liat", "buka", "cek", "periksa", "cari", "temuin", "temukan", "tampilkan", "list",
    "show", "read", "open", "find", "search", "cat", "ls", "grep", "scan", "telusuri",
];
const OBJECTS: [&str; 18] = [
    "file", "folder", "direktori", "directory", "dir", "isi", "path", "storage", "sdcard", "repo", "kode", "code",
    "proyek", "project", "script", "log", "dokumen", "catatan",
];
const STRONG_OBJECTS: [&str; 5] = ["folder", "direktori", "directory", "sdcard", "storage"];

/// Apakah pesan ini jelas meminta sesuatu soal file/folder? Dipakai buat memutuskan menawarkan alat file.
pub fn asks_for_files(text: &str) -> bool {
    let lower = text.to_lowercase();
    let has_path = lower.split_whitespace().any(|w| {
        let w = w.trim_matches(|c: char| matches!(c, '"' | '\'' | '`' | ',' | '.' | '?' | '!' | ':' | ';' | '(' | ')'));
        (w.starts_with('@') && w.len() > 1)
            || (w.starts_with('/') && w.len() > 1)
            || w.starts_with("~/")
            || w.starts_with("./")
            || w.starts_with("../")
    });
    if has_path {
        return true;
    }
    let words: Vec<&str> = lower.split(|c: char| !c.is_alphanumeric()).filter(|w| !w.is_empty()).collect();
    let has_action = words.iter().any(|w| ACTIONS.contains(w));
    let has_object = words.iter().any(|w| OBJECTS.contains(w));
    let has_strong = words.iter().any(|w| STRONG_OBJECTS.contains(w));
    (has_action && has_object) || has_strong
}

/// Definisi alat dalam format function-calling OpenAI.
pub fn definitions() -> Value {
    json!([
        {
            "type": "function",
            "function": {
                "name": "list_dir",
                "description": "Daftar isi sebuah folder di perangkat user (nama, tipe, ukuran). Hanya dipakai kalau user menyuruh.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "path": { "type": "string", "description": "Path folder; mendukung ~ dan path relatif." },
                        "show_hidden": { "type": "boolean", "description": "Sertakan file tersembunyi (diawali titik)." }
                    },
                    "required": ["path"]
                }
            }
        },
        {
            "type": "function",
            "function": {
                "name": "read_file",
                "description": "Baca file teks. Untuk file besar pakai start_line dan max_lines. Hanya dipakai kalau user menyuruh.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "path": { "type": "string", "description": "Path file." },
                        "start_line": { "type": "integer", "description": "Mulai dari baris ke-n (default 1)." },
                        "max_lines": { "type": "integer", "description": "Jumlah baris maksimal (default 400)." }
                    },
                    "required": ["path"]
                }
            }
        },
        {
            "type": "function",
            "function": {
                "name": "find_files",
                "description": "Cari file atau folder yang namanya mengandung teks tertentu di bawah sebuah folder (.git dan node_modules dilewati).",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "path": { "type": "string", "description": "Folder awal pencarian." },
                        "name": { "type": "string", "description": "Teks yang harus ada di nama (tidak peka huruf besar/kecil)." }
                    },
                    "required": ["path", "name"]
                }
            }
        },
        {
            "type": "function",
            "function": {
                "name": "grep",
                "description": "Cari teks di dalam file teks, di satu file atau di bawah sebuah folder (.git dan node_modules dilewati).",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "path": { "type": "string", "description": "File atau folder yang dicari." },
                        "query": { "type": "string", "description": "Teks yang dicari (tidak peka huruf besar/kecil)." }
                    },
                    "required": ["path", "query"]
                }
            }
        }
    ])
}

/// Teks pendek buat ditampilkan ke user setiap kali model memakai alat.
pub fn label(name: &str, args: &Value) -> String {
    let path = args["path"].as_str().unwrap_or("?");
    match name {
        "list_dir" => format!("lihat folder {path}"),
        "read_file" => format!("baca {path}"),
        "find_files" => format!("cari nama '{}' di {path}", args["name"].as_str().unwrap_or("?")),
        "grep" => format!("cari teks '{}' di {path}", args["query"].as_str().unwrap_or("?")),
        other => other.to_string(),
    }
}

/// Jalankan alat. Kesalahan dikembalikan sebagai teks "Error: ..." supaya model bisa menyesuaikan diri.
pub fn run(name: &str, args: &Value) -> String {
    let result = match name {
        "list_dir" => list_dir(args),
        "read_file" => read_file(args),
        "find_files" => find_files(args),
        "grep" => grep(args),
        other => format!("Error: alat '{other}' nggak dikenal"),
    };
    cap(result)
}

fn cap(mut text: String) -> String {
    if text.len() > MAX_OUT {
        let cut = text.char_indices().map(|(i, _)| i).take_while(|&i| i <= MAX_OUT).last().unwrap_or(0);
        text.truncate(cut);
        text.push_str("\n... (dipotong)");
    }
    text
}

fn list_dir(args: &Value) -> String {
    let Some(raw) = args["path"].as_str() else { return "Error: parameter path wajib".to_string() };
    let path = files::expand(raw);
    let hidden = args["show_hidden"].as_bool().unwrap_or(false);
    let read = match fs::read_dir(&path) {
        Ok(read) => read,
        Err(e) => return format!("Error: {}: {e}", path.display()),
    };
    let mut rows: Vec<(bool, String, u64)> = read
        .filter_map(|e| e.ok())
        .filter_map(|e| {
            let name = e.file_name().to_string_lossy().to_string();
            if !hidden && name.starts_with('.') {
                return None;
            }
            let meta = fs::metadata(e.path()).ok();
            let is_dir = meta.as_ref().map(|m| m.is_dir()).unwrap_or(false);
            Some((is_dir, name, meta.map(|m| m.len()).unwrap_or(0)))
        })
        .collect();
    rows.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| a.1.to_lowercase().cmp(&b.1.to_lowercase())));
    let total = rows.len();
    let mut out = format!("{} ({total} item):\n", path.display());
    for (is_dir, name, size) in rows.iter().take(MAX_LIST) {
        if *is_dir {
            out.push_str(&format!("{name}/\n"));
        } else {
            out.push_str(&format!("{name}  ({})\n", files::human_size(*size)));
        }
    }
    if total > MAX_LIST {
        out.push_str(&format!("... ({} lainnya)\n", total - MAX_LIST));
    }
    out
}

fn read_file(args: &Value) -> String {
    let Some(raw) = args["path"].as_str() else { return "Error: parameter path wajib".to_string() };
    let path = files::expand(raw);
    let meta = match fs::metadata(&path) {
        Ok(meta) => meta,
        Err(e) => return format!("Error: {}: {e}", path.display()),
    };
    if meta.is_dir() {
        return format!("Error: {} adalah folder, pakai list_dir", path.display());
    }
    let mut bytes = Vec::new();
    if let Err(e) = fs::File::open(&path).and_then(|file| file.take(READ_CAP).read_to_end(&mut bytes)) {
        return format!("Error: {}: {e}", path.display());
    }
    if bytes[..bytes.len().min(8000)].contains(&0) {
        return format!("Error: {} bukan file teks (biner)", path.display());
    }
    let text = String::from_utf8_lossy(&bytes);
    let total = text.lines().count();
    let start = args["start_line"].as_u64().unwrap_or(1).max(1) as usize;
    let max_lines = args["max_lines"].as_u64().unwrap_or(400).clamp(1, 2000) as usize;

    let mut body = String::new();
    let mut last = start.saturating_sub(1);
    for (i, line) in text.lines().enumerate().skip(start - 1).take(max_lines) {
        if body.len() + line.len() > MAX_OUT {
            break;
        }
        body.push_str(&format!("{}: {line}\n", i + 1));
        last = i + 1;
    }
    let mut out = format!("{} ({}, {total} baris, tampil {start}-{last}):\n{body}", path.display(), files::human_size(meta.len()));
    if last < total {
        out.push_str(&format!("... (masih ada; lanjut dengan start_line={})\n", last + 1));
    }
    if meta.len() > READ_CAP {
        out.push_str("(file lebih besar dari 2 MB, hanya bagian awal yang dibaca)\n");
    }
    out
}

/// Jelajah folder tanpa mengikuti symlink; `visit` mengembalikan false untuk berhenti.
fn walk(root: &Path, visit: &mut dyn FnMut(&Path, bool) -> bool) {
    let mut stack = vec![(root.to_path_buf(), 0usize)];
    while let Some((dir, depth)) = stack.pop() {
        let Ok(read) = fs::read_dir(&dir) else { continue };
        let mut entries: Vec<fs::DirEntry> = read.filter_map(|e| e.ok()).collect();
        entries.sort_by_key(|e| e.file_name());
        for entry in entries {
            let Ok(kind) = entry.file_type() else { continue };
            let path = entry.path();
            let is_dir = kind.is_dir();
            if !visit(&path, is_dir) {
                return;
            }
            let name = entry.file_name().to_string_lossy().to_string();
            let skip = SKIP_DIRS.contains(&name.as_str());
            if is_dir && depth < MAX_DEPTH && !skip {
                stack.push((path, depth + 1));
            }
        }
    }
}

fn find_files(args: &Value) -> String {
    let (Some(raw), Some(query)) = (args["path"].as_str(), args["name"].as_str()) else {
        return "Error: parameter path dan name wajib".to_string();
    };
    let root = files::expand(raw);
    let needle = query.to_lowercase();
    let mut hits: Vec<String> = Vec::new();
    let mut visited = 0;
    walk(&root, &mut |path, is_dir| {
        visited += 1;
        let name = path.file_name().map(|n| n.to_string_lossy().to_lowercase()).unwrap_or_default();
        if name.contains(&needle) {
            hits.push(format!("{}{}", path.display(), if is_dir { "/" } else { "" }));
        }
        hits.len() < MAX_HITS && visited < MAX_VISITED
    });
    if hits.is_empty() {
        format!("Tidak ada nama yang mengandung '{query}' di {}", root.display())
    } else {
        hits.join("\n")
    }
}

fn grep_file(path: &Path, needle: &str, hits: &mut Vec<String>) {
    let Ok(meta) = fs::metadata(path) else { return };
    if meta.len() > 1_000_000 {
        return;
    }
    let Ok(bytes) = fs::read(path) else { return };
    if bytes[..bytes.len().min(4000)].contains(&0) {
        return;
    }
    let text = String::from_utf8_lossy(&bytes);
    for (i, line) in text.lines().enumerate() {
        if line.to_lowercase().contains(needle) {
            let shown: String = line.trim().chars().take(200).collect();
            hits.push(format!("{}:{}: {shown}", path.display(), i + 1));
            if hits.len() >= MAX_HITS {
                return;
            }
        }
    }
}

fn grep(args: &Value) -> String {
    let (Some(raw), Some(query)) = (args["path"].as_str(), args["query"].as_str()) else {
        return "Error: parameter path dan query wajib".to_string();
    };
    let root = files::expand(raw);
    let needle = query.to_lowercase();
    let mut hits: Vec<String> = Vec::new();
    match fs::metadata(&root) {
        Ok(meta) if meta.is_file() => grep_file(&root, &needle, &mut hits),
        Ok(_) => {
            let mut scanned = 0;
            walk(&root, &mut |path, is_dir| {
                if !is_dir {
                    scanned += 1;
                    grep_file(path, &needle, &mut hits);
                }
                scanned < 5000 && hits.len() < MAX_HITS
            });
        }
        Err(e) => return format!("Error: {}: {e}", root.display()),
    }
    if hits.is_empty() {
        format!("Tidak ketemu '{query}' di {}", root.display())
    } else {
        hits.join("\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_explicit_file_requests_enable_tools() {
        assert!(asks_for_files("tolong baca file ~/catatan.txt"));
        assert!(asks_for_files("lihat isi folder download"));
        assert!(asks_for_files("jelasin /sdcard/Download/a.txt"));
        assert!(!asks_for_files("halo apa kabar"));
        assert!(!asks_for_files("jelasin cara kerja rust ownership"));
    }

    #[test]
    fn access_modes_parse() {
        assert!(Access::parse("ON") == Some(Access::Always));
        assert!(Access::parse("mati") == Some(Access::Never));
        assert!(Access::parse("entah").is_none());
    }
}
