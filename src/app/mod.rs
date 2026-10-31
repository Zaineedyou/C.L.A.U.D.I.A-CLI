mod commands;
mod keys;

use std::collections::HashMap;

use reqwest::Client;
use tokio::task::JoinHandle;

use crate::{
    api::{self, Target},
    chat::{Message, Role},
    config::{display_cwd, Config},
    event::{Ev, Tx},
    input::Input,
    picker::{Item, Kind, Picker, POPUP_FRAMES},
    provider,
};

/// Jumlah pesan terakhir yang dikirim sebagai konteks.
const HISTORY_LIMIT: usize = 40;

/// Jumlah tick transisi dari layar awal ke layar sesi.
pub const SLIDE_FRAMES: u8 = 12;

pub enum Notice {
    Info(String),
    Error(String),
}

pub struct App {
    pub cfg: Config,
    pub messages: Vec<Message>,
    pub input: Input,
    pub picker: Option<Picker>,
    pub notice: Option<Notice>,
    pub scroll: usize,
    pub tick: usize,
    /// 0 = layar awal, SLIDE_FRAMES = layar sesi penuh.
    pub slide: u8,
    pub cwd: String,
    pub streaming: bool,
    pub quit: bool,
    client: Client,
    tx: Tx,
    task: Option<JoinHandle<()>>,
    models_cache: HashMap<&'static str, Vec<String>>,
}

impl App {
    pub fn new(cfg: Config, client: Client, tx: Tx) -> Self {
        Self {
            cfg,
            messages: Vec::new(),
            input: Input::default(),
            picker: None,
            notice: None,
            scroll: 0,
            tick: 0,
            slide: 0,
            cwd: display_cwd(),
            streaming: false,
            quit: false,
            client,
            tx,
            task: None,
            models_cache: HashMap::new(),
        }
    }

    pub fn on_event(&mut self, ev: Ev) {
        match ev {
            Ev::Tick => self.on_tick(),
            Ev::Key(key) => self.on_key(key),
            Ev::Chunk(text) => {
                if let Some(msg) = self.messages.last_mut() {
                    msg.content.push_str(&text);
                }
            }
            Ev::Done => self.finish_stream(),
            Ev::Failed(e) => {
                self.finish_stream();
                self.err(e);
            }
            Ev::Models { provider, result } => self.on_models(provider, result),
        }
    }

    fn on_tick(&mut self) {
        self.tick = self.tick.wrapping_add(1);
        self.slide = if self.messages.is_empty() {
            self.slide.saturating_sub(2)
        } else {
            (self.slide + 1).min(SLIDE_FRAMES)
        };
        if let Some(picker) = self.picker.as_mut() {
            picker.age = (picker.age + 1).min(POPUP_FRAMES);
        }
    }

    fn info(&mut self, text: impl Into<String>) {
        self.notice = Some(Notice::Info(text.into()));
    }

    fn err(&mut self, text: impl Into<String>) {
        self.notice = Some(Notice::Error(text.into()));
    }

    fn target(&self) -> Target {
        let p = self.cfg.current();
        Target {
            id: p.id,
            base_url: self.cfg.base_url(),
            key: self.cfg.keys.get(p.id).cloned(),
            model: self.cfg.model(),
        }
    }

    // ── chat ──────────────────────────────────────────────

    fn submit(&mut self) {
        let text = self.input.text().trim().to_string();
        if text.is_empty() {
            return;
        }
        if text.starts_with('/') {
            self.input.clear();
            self.run_command(&text);
            return;
        }
        if self.streaming {
            return;
        }
        let p = self.cfg.current();
        if !self.cfg.is_ready(p) {
            self.err(format!("{} belum siap: set {} (atau /key <kunci>)", p.name, p.requirement()));
            return;
        }

        self.input.clear();
        self.notice = None;
        self.scroll = 0;
        self.messages.push(Message::new(Role::User, text));

        let mut history = vec![Message::new(Role::System, self.cfg.system.clone())];
        let start = self.messages.len().saturating_sub(HISTORY_LIMIT);
        history.extend_from_slice(&self.messages[start..]);

        let tag = format!("{} · {}", self.cfg.name, self.cfg.model());
        self.messages.push(Message::new(Role::Assistant, "").tagged(tag));
        self.streaming = true;
        self.task = Some(api::spawn_chat(self.client.clone(), self.target(), history, self.tx.clone()));
    }

    fn finish_stream(&mut self) {
        self.streaming = false;
        self.task = None;
        self.drop_empty_reply();
    }

    fn cancel(&mut self) {
        if let Some(task) = self.task.take() {
            task.abort();
        }
        self.finish_stream();
    }

    fn clear(&mut self) {
        self.cancel();
        self.messages.clear();
        self.notice = None;
        self.scroll = 0;
    }

    fn drop_empty_reply(&mut self) {
        if self.messages.last().is_some_and(|m| m.role == Role::Assistant && m.content.is_empty()) {
            self.messages.pop();
        }
    }

    // ── provider & model ──────────────────────────────────

    fn set_provider(&mut self, index: usize) {
        self.cancel();
        self.cfg.selected = index;
        self.cfg.save();
        let p = self.cfg.current();
        if self.cfg.is_ready(p) {
            self.info(format!("{} · {}", p.name, self.cfg.model()));
        } else {
            self.err(format!("{} belum siap: set {} (atau /key <kunci>)", p.name, p.requirement()));
        }
    }

    fn set_model(&mut self, model: String) {
        self.cfg.set_model(model);
        self.cfg.save();
        self.info(format!("model: {}", self.cfg.model()));
    }

    fn open_providers(&mut self) {
        let items = provider::ALL
            .iter()
            .enumerate()
            .map(|(i, p)| Item {
                id: p.id.into(),
                label: p.name.into(),
                hint: if self.cfg.is_ready(p) { "siap".into() } else { format!("butuh {}", p.requirement()) },
                current: i == self.cfg.selected,
            })
            .collect();
        self.picker = Some(Picker::new(Kind::Provider, items, false));
    }

    fn open_models(&mut self) {
        let cached = self.models_cache.contains_key(self.cfg.current().id);
        self.picker = Some(Picker::new(Kind::Model, self.model_items(), !cached));
        if !cached {
            api::spawn_models(self.client.clone(), self.target(), self.tx.clone());
        }
    }

    /// Daftar asli dari API kalau sudah ada, kalau belum pakai daftar bawaan.
    fn model_items(&self) -> Vec<Item> {
        let p = self.cfg.current();
        let current = self.cfg.model();
        let mut names: Vec<String> = match self.models_cache.get(p.id) {
            Some(list) => list.clone(),
            None => p.fallback_models.iter().map(|m| m.to_string()).collect(),
        };
        if !names.contains(&current) {
            names.insert(0, current.clone());
        }
        names
            .into_iter()
            .map(|n| Item { current: n == current, id: n.clone(), label: n, hint: String::new() })
            .collect()
    }

    fn on_models(&mut self, provider: &'static str, result: Result<Vec<String>, String>) {
        match result {
            Ok(list) => {
                self.models_cache.insert(provider, list);
            }
            Err(e) => self.err(format!("daftar model: {e}")),
        }
        if self.cfg.current().id == provider {
            let items = self.model_items();
            if let Some(picker) = self.picker.as_mut().filter(|p| p.kind == Kind::Model) {
                picker.set_items(items);
            }
        }
    }

    fn confirm_pick(&mut self) {
        let Some(picker) = &self.picker else { return };
        let choice = match (picker.selected_item(), picker.kind) {
            (Some(item), _) => item.id.clone(),
            (None, Kind::Model) if !picker.filter.trim().is_empty() => picker.filter.trim().to_string(),
            _ => return,
        };
        let kind = picker.kind;
        self.picker = None;
        match kind {
            Kind::Provider => {
                if let Some(i) = provider::find(&choice) {
                    self.set_provider(i);
                }
            }
            Kind::Model => self.set_model(choice),
        }
    }
}
