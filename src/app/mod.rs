mod commands;
mod keys;

use std::{collections::HashMap, time::Instant};

use reqwest::Client;
use tokio::task::JoinHandle;

use crate::{
    api::{self, Target},
    chat::{Message, Role},
    config::{display_cwd, Config},
    editor::Editor,
    event::{Ev, Tx},
    input::Input,
    picker::{Item, Kind, Picker, POPUP_FRAMES},
    provider,
};

/// Jumlah pesan terakhir yang dikirim sebagai konteks.
const HISTORY_LIMIT: usize = 40;

/// Jumlah tick transisi dari layar awal ke layar sesi.
pub const SLIDE_FRAMES: u8 = 12;

/// Satu sesi. Sesi yang sedang aktif hidup di `App::messages`; isi tab-nya di sini basi.
#[derive(Default)]
pub struct Tab {
    pub messages: Vec<Message>,
    pub scroll: usize,
}

pub enum Notice {
    Info(String),
    Error(String),
}

pub struct App {
    pub cfg: Config,
    pub messages: Vec<Message>,
    pub input: Input,
    pub picker: Option<Picker>,
    pub editor: Option<Editor>,
    pub tabs: Vec<Tab>,
    pub active: usize,
    pub notice: Option<Notice>,
    pub scroll: usize,
    pub tick: usize,
    /// 0 = layar awal, SLIDE_FRAMES = layar sesi penuh.
    pub slide: u8,
    pub cwd: String,
    pub streaming: bool,
    pub quit: bool,
    started: Option<Instant>,
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
            editor: None,
            tabs: vec![Tab::default()],
            active: 0,
            notice: None,
            scroll: 0,
            tick: 0,
            slide: 0,
            cwd: display_cwd(),
            streaming: false,
            quit: false,
            started: None,
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
                // Sisa potongan dari balasan yang sudah dibatalkan atau beda tab dibuang.
                if self.streaming {
                    if let Some(msg) = self.messages.last_mut() {
                        msg.content.push_str(&text);
                    }
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
        if let Some(editor) = self.editor.as_mut() {
            editor.age = (editor.age + 1).min(POPUP_FRAMES);
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
        self.started = Some(Instant::now());
        self.task = Some(api::spawn_chat(self.client.clone(), self.target(), history, self.tx.clone()));
    }

    fn finish_stream(&mut self) {
        self.streaming = false;
        self.task = None;
        if let Some(start) = self.started.take() {
            let secs = start.elapsed().as_secs_f32().max(0.1);
            if let Some(msg) = self.messages.last_mut() {
                if msg.role == Role::Assistant && !msg.content.is_empty() {
                    let tokens = (msg.content.chars().count() as f32 / 4.0).max(1.0);
                    msg.tag = format!("{} · {} · {:.1} tok/s", msg.tag, format_duration(secs), tokens / secs);
                }
            }
        }
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

    // ── tab sesi ──────────────────────────────────────────

    /// Judul tab: baris pertama pesan user pertama; `None` untuk sesi kosong.
    pub fn tab_title(&self, i: usize) -> Option<String> {
        let messages = if i == self.active { &self.messages } else { &self.tabs.get(i)?.messages };
        messages
            .iter()
            .find(|m| m.role == Role::User)
            .and_then(|m| m.content.lines().next())
            .map(|line| line.trim().to_string())
    }

    fn switch_tab(&mut self, target: usize) {
        if target >= self.tabs.len() || target == self.active {
            return;
        }
        self.cancel();
        self.tabs[self.active] = Tab { messages: std::mem::take(&mut self.messages), scroll: self.scroll };
        let next = std::mem::take(&mut self.tabs[target]);
        self.messages = next.messages;
        self.scroll = next.scroll;
        self.active = target;
        self.notice = None;
    }

    fn step_tab(&mut self, delta: isize) {
        let count = self.tabs.len() as isize;
        let target = (self.active as isize + delta).rem_euclid(count) as usize;
        self.switch_tab(target);
    }

    fn new_tab(&mut self) {
        if self.messages.is_empty() {
            return;
        }
        let empty = (0..self.tabs.len()).find(|&i| i != self.active && self.tab_title(i).is_none());
        if let Some(i) = empty {
            self.switch_tab(i);
            return;
        }
        self.cancel();
        self.tabs[self.active] = Tab { messages: std::mem::take(&mut self.messages), scroll: self.scroll };
        self.tabs.push(Tab::default());
        self.active = self.tabs.len() - 1;
        self.scroll = 0;
        self.notice = None;
    }

    fn close_tab(&mut self) {
        self.cancel();
        if self.tabs.len() <= 1 {
            self.messages.clear();
            self.scroll = 0;
            return;
        }
        self.tabs.remove(self.active);
        let next = self.active.min(self.tabs.len() - 1);
        let tab = std::mem::take(&mut self.tabs[next]);
        self.messages = tab.messages;
        self.scroll = tab.scroll;
        self.active = next;
        self.notice = None;
    }

    // ── system prompt ─────────────────────────────────────

    fn open_system_editor(&mut self) {
        self.editor = Some(Editor::new(&self.cfg.system));
    }

    fn save_system(&mut self) {
        let Some(editor) = self.editor.take() else { return };
        let text = editor.text().trim().to_string();
        if text.is_empty() {
            self.cfg.reset_system();
            self.info("system prompt dikembalikan ke bawaan");
        } else {
            self.cfg.set_system(text);
            self.info("system prompt disimpan, berlaku mulai pesan berikutnya");
        }
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

/// "45s" atau "1m 14s".
fn format_duration(secs: f32) -> String {
    let total = secs.round() as u64;
    if total >= 60 {
        format!("{}m {}s", total / 60, total % 60)
    } else {
        format!("{total}s")
    }
}
