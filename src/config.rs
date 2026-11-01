use std::{collections::HashMap, env, fmt::Write, fs, path::PathBuf};

use crate::provider::{self, Provider};

pub struct Config {
    pub selected: usize,
    pub name: String,
    pub system: String,
    /// API key per provider (dari env var atau `/key`), tidak pernah ditulis ke disk.
    pub keys: HashMap<&'static str, String>,
    /// Model pilihan per provider, disimpan di file state.
    models: HashMap<&'static str, String>,
}

fn env_nonempty(key: &str) -> Option<String> {
    env::var(key).ok().filter(|v| !v.is_empty())
}

/// Direktori kerja untuk footer, `$HOME` diringkas jadi `~`.
pub fn display_cwd() -> String {
    let cwd = env::current_dir().unwrap_or_default().display().to_string();
    match env::var("HOME") {
        Ok(home) if !home.is_empty() && cwd.starts_with(&home) => format!("~{}", &cwd[home.len()..]),
        _ => cwd,
    }
}

fn config_dir() -> Option<PathBuf> {
    let base = env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|| env::var_os("HOME").map(|h| PathBuf::from(h).join(".config")))?;
    Some(base.join("claudia"))
}

fn state_path() -> Option<PathBuf> {
    Some(config_dir()?.join("state"))
}

/// System prompt hasil edit, disimpan apa adanya (boleh banyak baris).
fn system_path() -> Option<PathBuf> {
    Some(config_dir()?.join("system"))
}

fn default_system(name: &str) -> String {
    format!(
        "Kamu adalah {name} (C.L.A.U.D.I.A), teman ngobrol yang santai, hangat, dan nyambung. \
         Balas dalam bahasa yang dipakai lawan bicara (default Indonesia santai). \
         Jawab ringkas kecuali diminta detail."
    )
}

impl Config {
    pub fn load() -> Self {
        let keys = provider::ALL
            .iter()
            .filter(|p| !p.key_env.is_empty())
            .filter_map(|p| env_nonempty(p.key_env).map(|key| (p.id, key)))
            .collect();
        let name = env_nonempty("CLAUDIA_NAME").unwrap_or_else(|| "Claudia".into());
        // Prioritas: hasil edit (/system) > env CLAUDIA_SYSTEM > bawaan.
        let saved = system_path().and_then(|p| fs::read_to_string(p).ok()).filter(|t| !t.trim().is_empty());
        let system = saved
            .or_else(|| env_nonempty("CLAUDIA_SYSTEM"))
            .unwrap_or_else(|| default_system(&name));

        let mut cfg = Self { selected: 0, name, system, keys, models: HashMap::new() };
        let saved = cfg.load_state();
        cfg.selected = env_nonempty("CLAUDIA_PROVIDER")
            .and_then(|s| provider::find(&s))
            .or(saved)
            .or_else(|| provider::ALL.iter().position(|p| cfg.keys.contains_key(p.id)))
            .unwrap_or(0);
        if let Some(model) = env_nonempty("CLAUDIA_MODEL") {
            cfg.set_model(model);
        }
        cfg
    }

    pub fn default_system(&self) -> String {
        default_system(&self.name)
    }

    /// Ganti system prompt dan simpan ke disk. Gagal tulis diabaikan.
    pub fn set_system(&mut self, text: String) {
        self.system = text;
        let Some(path) = system_path() else { return };
        if let Some(dir) = path.parent() {
            let _ = fs::create_dir_all(dir);
        }
        let _ = fs::write(path, &self.system);
    }

    /// Kembalikan system prompt ke bawaan dan hapus file hasil edit.
    pub fn reset_system(&mut self) {
        self.system = default_system(&self.name);
        if let Some(path) = system_path() {
            let _ = fs::remove_file(path);
        }
    }

    pub fn current(&self) -> &'static Provider {
        &provider::ALL[self.selected]
    }

    pub fn model(&self) -> String {
        let p = self.current();
        self.models.get(p.id).cloned().unwrap_or_else(|| p.default_model.to_string())
    }

    pub fn set_model(&mut self, model: String) {
        let id = self.current().id;
        self.models.insert(id, model);
    }

    pub fn base_url(&self) -> String {
        let p = self.current();
        if p.is_custom() {
            let url = env_nonempty("CLAUDIA_BASE_URL").unwrap_or_default();
            url.trim_end_matches('/').to_string()
        } else {
            p.base_url.to_string()
        }
    }

    pub fn is_ready(&self, p: &Provider) -> bool {
        if p.is_custom() {
            env_nonempty("CLAUDIA_BASE_URL").is_some()
        } else {
            p.key_env.is_empty() || self.keys.contains_key(p.id)
        }
    }

    fn load_state(&mut self) -> Option<usize> {
        let text = fs::read_to_string(state_path()?).ok()?;
        let mut selected = None;
        for (key, value) in text.lines().filter_map(|l| l.split_once('=')) {
            if key == "provider" {
                selected = provider::find(value);
            } else if let Some(i) = key.strip_prefix("model.").and_then(provider::find) {
                self.models.insert(provider::ALL[i].id, value.trim().to_string());
            }
        }
        selected
    }

    /// Simpan provider terakhir dan model per provider. Gagal tulis diabaikan.
    pub fn save(&self) {
        let Some(path) = state_path() else { return };
        let mut out = format!("provider={}\n", self.current().id);
        for (id, model) in &self.models {
            let _ = writeln!(out, "model.{id}={model}");
        }
        if let Some(dir) = path.parent() {
            let _ = fs::create_dir_all(dir);
        }
        let _ = fs::write(path, out);
    }
}
