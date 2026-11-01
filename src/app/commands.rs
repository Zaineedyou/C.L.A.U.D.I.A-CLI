use crate::provider;

use super::App;

const HELP: &str = "/provider · /model · /key <kunci> · /system · /new · /close · /tab <n> · /clear · /exit";

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
            ("/key", "") => self.err("pakai: /key <api-key>"),
            ("/key", key) => {
                let id = self.cfg.current().id;
                self.cfg.keys.insert(id, key.to_string());
                self.info("API key diset (cuma untuk sesi ini)");
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
