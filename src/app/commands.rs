use crate::provider;

use super::App;

const HELP: &str = "/provider [nama] · /model [nama] · /key <kunci> · /clear · /exit";

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
            _ => self.err(format!("perintah nggak dikenal: {cmd}")),
        }
    }
}
