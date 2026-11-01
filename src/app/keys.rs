use ratatui::{
    crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers, MouseButton, MouseEvent, MouseEventKind},
    layout::Position,
};

use crate::{
    editor::Editor,
    ui::{tab_hits, Click},
};

use super::App;

impl App {
    pub(super) fn on_key(&mut self, key: KeyEvent) {
        if key.kind != KeyEventKind::Press {
            return;
        }
        let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
        let alt = key.modifiers.contains(KeyModifiers::ALT);
        if ctrl && key.code == KeyCode::Char('c') {
            self.quit = true;
        } else if self.editor.is_some() {
            self.editor_key(key.code, ctrl);
        } else if self.keyentry.is_some() {
            self.keyentry_key(key.code, ctrl);
        } else if self.picker.is_some() {
            self.picker_key(key.code, ctrl);
        } else if alt && self.tab_key(key.code) {
            // Alt+panah / Alt+angka sudah ditangani sebagai pindah tab.
        } else {
            self.chat_key(key.code, ctrl);
        }
    }

    /// Klik kiri pada tab atau "+" di baris sesi; roda/geser menggulir percakapan.
    pub(super) fn on_mouse(&mut self, mouse: MouseEvent) {
        match mouse.kind {
            MouseEventKind::Down(MouseButton::Left) => {
                if self.picker.is_some() || self.editor.is_some() || self.keyentry.is_some() {
                    return;
                }
                let at = Position::new(mouse.column, mouse.row);
                let hit = tab_hits(self.tab_area, self).into_iter().find(|(rect, _)| rect.contains(at));
                match hit {
                    Some((_, Click::Tab(i))) => self.switch_tab(i),
                    Some((_, Click::New)) => self.new_tab(),
                    None => {}
                }
            }
            MouseEventKind::ScrollUp => self.scroll += 3,
            MouseEventKind::ScrollDown => self.scroll = self.scroll.saturating_sub(3),
            _ => {}
        }
    }

    /// Alt+Bawah/Kanan tab berikutnya, Alt+Atas/Kiri sebelumnya, Alt+1..9 langsung ke tab itu.
    fn tab_key(&mut self, code: KeyCode) -> bool {
        match code {
            KeyCode::Down | KeyCode::Right => self.step_tab(1),
            KeyCode::Up | KeyCode::Left => self.step_tab(-1),
            KeyCode::Char(c @ '1'..='9') => self.switch_tab((c as u32 - '1' as u32) as usize),
            _ => return false,
        }
        true
    }

    fn editor_key(&mut self, code: KeyCode, ctrl: bool) {
        match code {
            KeyCode::Esc => self.editor = None,
            KeyCode::Char('s') if ctrl => self.save_system(),
            KeyCode::Char('r') if ctrl => {
                let text = self.cfg.default_system();
                self.editor = Some(Editor::new(&text));
            }
            KeyCode::Char('u') if ctrl => self.editor = Some(Editor::new("")),
            _ if ctrl => {}
            _ => {
                if let Some(editor) = self.editor.as_mut() {
                    editor.handle(code);
                }
            }
        }
    }

    fn keyentry_key(&mut self, code: KeyCode, ctrl: bool) {
        match code {
            KeyCode::Esc => {
                self.keyentry = None;
                self.open_connect();
            }
            KeyCode::Enter => self.save_key(),
            KeyCode::Char('u') if ctrl => {
                if let Some(entry) = self.keyentry.as_mut() {
                    entry.input.clear();
                }
            }
            _ if ctrl => {}
            _ => {
                if let Some(entry) = self.keyentry.as_mut() {
                    entry.error = None;
                    match code {
                        KeyCode::Char(c) => entry.input.insert(c),
                        KeyCode::Backspace => entry.input.backspace(),
                        KeyCode::Delete => entry.input.delete(),
                        KeyCode::Left => entry.input.left(),
                        KeyCode::Right => entry.input.right(),
                        KeyCode::Home => entry.input.home(),
                        KeyCode::End => entry.input.end(),
                        _ => {}
                    }
                }
            }
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
        // Dropdown perintah "/": panah pilih, Tab melengkapi, Enter menjalankan, Esc menutup.
        let matches = self.command_matches();
        if !matches.is_empty() {
            let count = matches.len();
            let sel = self.cmd_sel.min(count - 1);
            match code {
                KeyCode::Up => {
                    self.cmd_sel = (sel + count - 1) % count;
                    return;
                }
                KeyCode::Down => {
                    self.cmd_sel = (sel + 1) % count;
                    return;
                }
                KeyCode::Tab => {
                    self.complete_command(matches[sel]);
                    return;
                }
                KeyCode::Enter => {
                    let cmd = matches[sel];
                    if cmd.arg {
                        self.complete_command(cmd);
                    } else {
                        self.input.clear();
                        self.cmd_sel = 0;
                        self.run_command(&format!("/{}", cmd.name));
                    }
                    return;
                }
                KeyCode::Esc => {
                    self.input.clear();
                    self.cmd_sel = 0;
                    return;
                }
                _ => {}
            }
        }
        if matches!(code, KeyCode::Char(_) | KeyCode::Backspace | KeyCode::Delete) {
            self.cmd_sel = 0;
        }

        match code {
            KeyCode::Char('d') if ctrl && self.input.is_empty() => self.quit = true,
            KeyCode::Char('l') if ctrl => self.clear(),
            KeyCode::Char('n') if ctrl => self.new_tab(),
            KeyCode::Char('w') if ctrl => self.close_tab(),
            KeyCode::Char('u') if ctrl => self.input.clear(),
            KeyCode::Char('p') if ctrl => self.open_commands(),
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
