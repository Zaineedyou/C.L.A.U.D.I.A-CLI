use crate::provider;

use super::App;

const HELP: &str = "Ketik / buat daftar perintah · Ctrl+P palet perintah · Ctrl+O model · /connect buat isi API key";

/// Perintah slash, dipakai dropdown "/" dan palet perintah (Ctrl+P).
pub struct Cmd {
    pub name: &'static str,
    pub desc: &'static str,
    /// Perintah yang butuh argumen: dilengkapi dulu, tidak langsung dijalankan.
    pub arg: bool,
}

pub const COMMANDS: &[Cmd] = &[
    Cmd { name: "connect", desc: "Hubungkan provider dan masukin API key", arg: false },
    Cmd { name: "provider", desc: "Ganti provider", arg: false },
    Cmd { name: "model", desc: "Ganti model", arg: false },
    Cmd { name: "system", desc: "Edit system prompt", arg: false },
    Cmd { name: "new", desc: "Sesi baru", arg: false },
    Cmd { name: "close", desc: "Tutup sesi ini", arg: false },
    Cmd { name: "clear", desc: "Bersihkan sesi ini", arg: false },
    Cmd { name: "tab", desc: "Pindah ke tab nomor ...", arg: true },
    Cmd { name: "key", desc: "Isi API key provider ini", arg: true },
    Cmd { name: "disconnect", desc: "Hapus API key provider ini", arg: false },
    Cmd { name: "help", desc: "Bantuan", arg: false },
    Cmd { name: "exit", desc: "Keluar", arg: false },
];

/// Perintah yang cocok dengan `query` (tanpa "/"): yang diawali query dulu, lalu urut abjad.
pub fn filter(query: &str) -> Vec<&'static Cmd> {
    let q = query.to_lowercase();
    let mut hits: Vec<&'static Cmd> = COMMANDS
        .iter()
        .filter(|c| c.name.contains(q.as_str()) || c.desc.to_lowercase().contains(q.as_str()))
        .collect();
    hits.sort_by_key(|c| (!c.name.starts_with(q.as_str()), c.name));
    hits
}

impl App {
    pub(super) fn run_command(&mut self, line: &str) {
        let (cmd, arg) = line.split_once(' ').map_or((line, ""), |(c, a)| (c, a.trim()));
        match (cmd, arg) {
            ("/exit" | "/quit", _) => self.quit = true,
            ("/clear", _) => self.clear(),
            ("/help", _) => self.info(HELP),
            ("/provider", "") => self.open_providers(),
            ("/provider", name) => match provider::find(name) {
                Some(i) => self.set_provider(i),
                None => self.err(format!("provider '{name}' nggak ada (Ctrl+P buat daftar)")),
            },
            ("/model" | "/models", "") => self.open_models(),
            ("/model" | "/models", name) => self.set_model(name.to_string()),
            ("/connect", "") => self.open_connect(),
            ("/connect", name) => match provider::find(name) {
                Some(i) => self.begin_connect(i),
                None => self.err(format!("provider '{name}' nggak ada (ketik /connect buat daftar)")),
            },
            ("/key", "") => self.err("pakai: /key <api-key> (atau /connect buat dialognya)"),
            ("/key", key) => {
                let p = self.cfg.current();
                self.cfg.set_key(p.id, key.to_string());
                self.info(format!("API key {} disimpan", p.name));
            }
            ("/disconnect", _) => {
                let p = self.cfg.current();
                self.cfg.forget_key(p.id);
                self.info(format!("API key {} dihapus", p.name));
            }
            ("/system" | "/prompt", "") => self.open_system_editor(),
            ("/system" | "/prompt", "reset") => {
                self.cfg.reset_system();
                self.info("system prompt dikembalikan ke bawaan");
            }
            ("/system" | "/prompt", text) => {
                self.cfg.set_system(text.to_string());
                self.info("system prompt disimpan, berlaku mulai pesan berikutnya");
            }
            ("/new", _) => self.new_tab(),
            ("/close", _) => self.close_tab(),
            ("/tab", n) => match n.parse::<usize>() {
                Ok(i) if i >= 1 && i <= self.tabs.len() => self.switch_tab(i - 1),
                _ => self.err("pakai: /tab <nomor>"),
            },
            _ => self.err(format!("perintah nggak dikenal: {cmd}")),
        }
    }
}
