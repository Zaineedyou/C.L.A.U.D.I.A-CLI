use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

use super::App;

impl App {
    pub(super) fn on_key(&mut self, key: KeyEvent) {
        if key.kind != KeyEventKind::Press {
            return;
        }
        let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
        if ctrl && key.code == KeyCode::Char('c') {
            self.quit = true;
        } else if self.picker.is_some() {
            self.picker_key(key.code, ctrl);
        } else {
            self.chat_key(key.code, ctrl);
        }
    }

    fn picker_key(&mut self, code: KeyCode, ctrl: bool) {
        match code {
            KeyCode::Esc => self.picker = None,
            KeyCode::Enter => self.confirm_pick(),
            _ if ctrl => {}
            _ => {
                if let Some(picker) = self.picker.as_mut() {
                    picker.handle(code);
                }
            }
        }
    }

    fn chat_key(&mut self, code: KeyCode, ctrl: bool) {
        match code {
            KeyCode::Char('d') if ctrl && self.input.is_empty() => self.quit = true,
            KeyCode::Char('l') if ctrl => self.clear(),
            KeyCode::Char('u') if ctrl => self.input.clear(),
            KeyCode::Char('p') if ctrl => self.open_providers(),
            KeyCode::Char('o') if ctrl => self.open_models(),
            KeyCode::Char('a') if ctrl => self.input.home(),
            KeyCode::Char('e') if ctrl => self.input.end(),
            KeyCode::Char(c) if !ctrl => self.input.insert(c),
            KeyCode::Enter => self.submit(),
            KeyCode::Backspace => self.input.backspace(),
            KeyCode::Delete => self.input.delete(),
            KeyCode::Left => self.input.left(),
            KeyCode::Right => self.input.right(),
            KeyCode::Home => self.input.home(),
            KeyCode::End => {
                self.input.end();
                self.scroll = 0;
            }
            KeyCode::Up => self.scroll += 1,
            KeyCode::Down => self.scroll = self.scroll.saturating_sub(1),
            KeyCode::PageUp => self.scroll += 10,
            KeyCode::PageDown => self.scroll = self.scroll.saturating_sub(10),
            KeyCode::Esc => self.cancel(),
            _ => {}
        }
    }
}
