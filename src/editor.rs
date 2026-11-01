use ratatui::crossterm::event::KeyCode;
use unicode_width::UnicodeWidthChar;

/// Editor teks banyak baris untuk popup (dipakai edit system prompt).
pub struct Editor {
    lines: Vec<Vec<char>>,
    row: usize,
    col: usize,
    /// Umur popup dalam tick, untuk animasi buka.
    pub age: u8,
}

impl Editor {
    pub fn new(text: &str) -> Self {
        let mut lines: Vec<Vec<char>> = text.split('\n').map(|l| l.chars().collect()).collect();
        if lines.is_empty() {
            lines.push(Vec::new());
        }
        let row = lines.len() - 1;
        let col = lines[row].len();
        Self { lines, row, col, age: 0 }
    }

    pub fn text(&self) -> String {
        self.lines.iter().map(|l| l.iter().collect::<String>()).collect::<Vec<_>>().join("\n")
    }

    pub fn handle(&mut self, code: KeyCode) {
        match code {
            KeyCode::Char(c) => {
                self.lines[self.row].insert(self.col, c);
                self.col += 1;
            }
            KeyCode::Enter => {
                let rest = self.lines[self.row].split_off(self.col);
                self.row += 1;
                self.lines.insert(self.row, rest);
                self.col = 0;
            }
            KeyCode::Backspace => {
                if self.col > 0 {
                    self.col -= 1;
                    self.lines[self.row].remove(self.col);
                } else if self.row > 0 {
                    let cur = self.lines.remove(self.row);
                    self.row -= 1;
                    self.col = self.lines[self.row].len();
                    self.lines[self.row].extend(cur);
                }
            }
            KeyCode::Delete => {
                if self.col < self.lines[self.row].len() {
                    self.lines[self.row].remove(self.col);
                } else if self.row + 1 < self.lines.len() {
                    let next = self.lines.remove(self.row + 1);
                    self.lines[self.row].extend(next);
                }
            }
            KeyCode::Left => {
                if self.col > 0 {
                    self.col -= 1;
                } else if self.row > 0 {
                    self.row -= 1;
                    self.col = self.lines[self.row].len();
                }
            }
            KeyCode::Right => {
                if self.col < self.lines[self.row].len() {
                    self.col += 1;
                } else if self.row + 1 < self.lines.len() {
                    self.row += 1;
                    self.col = 0;
                }
            }
            KeyCode::Up => {
                if self.row > 0 {
                    self.row -= 1;
                    self.col = self.col.min(self.lines[self.row].len());
                }
            }
            KeyCode::Down => {
                if self.row + 1 < self.lines.len() {
                    self.row += 1;
                    self.col = self.col.min(self.lines[self.row].len());
                }
            }
            KeyCode::Home => self.col = 0,
            KeyCode::End => self.col = self.lines[self.row].len(),
            _ => {}
        }
    }

    /// Baris tampilan (di-wrap per karakter sesuai `width`) dan posisi kursor (x, y) di dalamnya.
    pub fn layout(&self, width: usize) -> (Vec<String>, usize, usize) {
        let width = width.max(2);
        let mut rows: Vec<String> = Vec::new();
        let (mut cx, mut cy) = (0, 0);
        for (r, line) in self.lines.iter().enumerate() {
            let mut cur = String::new();
            let mut cur_w = 0;
            for (i, ch) in line.iter().enumerate() {
                let w = ch.width().unwrap_or(0);
                if cur_w + w > width && cur_w > 0 {
                    rows.push(std::mem::take(&mut cur));
                    cur_w = 0;
                }
                if r == self.row && i == self.col {
                    cx = cur_w;
                    cy = rows.len();
                }
                cur.push(*ch);
                cur_w += w;
            }
            if r == self.row && self.col >= line.len() {
                if cur_w >= width {
                    rows.push(std::mem::take(&mut cur));
                    cur_w = 0;
                }
                cx = cur_w;
                cy = rows.len();
            }
            rows.push(cur);
        }
        (rows, cx, cy)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn enter_and_backspace_round_trip() {
        let mut e = Editor::new("ab");
        e.handle(KeyCode::Left);
        e.handle(KeyCode::Enter);
        assert_eq!(e.text(), "a\nb");
        e.handle(KeyCode::Backspace);
        assert_eq!(e.text(), "ab");
    }

    #[test]
    fn cursor_wraps_with_long_lines() {
        let e = Editor::new("abcdef");
        let (rows, cx, cy) = e.layout(4);
        assert_eq!(rows, ["abcd", "ef"]);
        assert_eq!((cx, cy), (2, 1));
    }
}
