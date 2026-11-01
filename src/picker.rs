use ratatui::crossterm::event::KeyCode;

/// Jumlah tick animasi membuka popup.
pub const POPUP_FRAMES: u8 = 6;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Provider,
    Model,
    /// Pilih provider yang mau dihubungkan (lanjut ke dialog API key).
    Connect,
    /// Palet perintah (Ctrl+P).
    Command,
}

pub struct Item {
    pub id: String,
    pub label: String,
    pub hint: String,
    pub current: bool,
}

/// Popup pilihan dengan filter ketik-langsung.
pub struct Picker {
    pub kind: Kind,
    pub items: Vec<Item>,
    pub filter: String,
    pub selected: usize,
    pub loading: bool,
    /// Umur popup dalam tick, untuk animasi buka.
    pub age: u8,
}

impl Picker {
    pub fn new(kind: Kind, items: Vec<Item>, loading: bool) -> Self {
        let selected = items.iter().position(|i| i.current).unwrap_or(0);
        Self { kind, items, filter: String::new(), selected, loading, age: 0 }
    }

    /// Item yang lolos filter.
    pub fn visible(&self) -> Vec<&Item> {
        let q = self.filter.to_lowercase();
        self.items
            .iter()
            .filter(|i| i.label.to_lowercase().contains(&q) || i.id.to_lowercase().contains(&q))
            .collect()
    }

    pub fn selected_item(&self) -> Option<&Item> {
        self.visible().get(self.selected).copied()
    }

    pub fn set_items(&mut self, items: Vec<Item>) {
        self.items = items;
        self.loading = false;
        self.selected = if self.filter.is_empty() {
            self.items.iter().position(|i| i.current).unwrap_or(0)
        } else {
            0
        };
    }

    pub fn handle(&mut self, code: KeyCode) {
        match code {
            KeyCode::Up => self.move_by(-1),
            KeyCode::Down => self.move_by(1),
            KeyCode::PageUp => self.move_by(-10),
            KeyCode::PageDown => self.move_by(10),
            KeyCode::Backspace => {
                self.filter.pop();
                self.selected = 0;
            }
            KeyCode::Char(c) => {
                self.filter.push(c);
                self.selected = 0;
            }
            _ => {}
        }
    }

    fn move_by(&mut self, delta: isize) {
        let last = self.visible().len().saturating_sub(1) as isize;
        self.selected = (self.selected as isize + delta).clamp(0, last) as usize;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn item(name: &str) -> Item {
        Item { id: name.into(), label: name.into(), hint: String::new(), current: false }
    }

    #[test]
    fn filter_is_case_insensitive() {
        let mut p = Picker::new(Kind::Provider, vec![item("Groq"), item("OpenAI")], false);
        p.handle(KeyCode::Char('O'));
        p.handle(KeyCode::Char('p'));
        assert_eq!(p.visible().len(), 1);
        assert_eq!(p.selected_item().map(|i| i.id.as_str()), Some("OpenAI"));
    }
}
