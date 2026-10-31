use unicode_width::UnicodeWidthChar;

/// Satu baris input dengan kursor (indeks dalam karakter).
#[derive(Default)]
pub struct Input {
    chars: Vec<char>,
    cursor: usize,
}

impl Input {
    pub fn is_empty(&self) -> bool {
        self.chars.is_empty()
    }

    pub fn text(&self) -> String {
        self.chars.iter().collect()
    }

    pub fn clear(&mut self) {
        self.chars.clear();
        self.cursor = 0;
    }

    pub fn insert(&mut self, c: char) {
        self.chars.insert(self.cursor, c);
        self.cursor += 1;
    }

    pub fn backspace(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
            self.chars.remove(self.cursor);
        }
    }

    pub fn delete(&mut self) {
        if self.cursor < self.chars.len() {
            self.chars.remove(self.cursor);
        }
    }

    pub fn left(&mut self) {
        self.cursor = self.cursor.saturating_sub(1);
    }

    pub fn right(&mut self) {
        self.cursor = (self.cursor + 1).min(self.chars.len());
    }

    pub fn home(&mut self) {
        self.cursor = 0;
    }

    pub fn end(&mut self) {
        self.cursor = self.chars.len();
    }

    /// Potongan teks yang muat di `width` kolom (minimal 1) dan kolom kursor di dalamnya.
    pub fn window(&self, width: usize) -> (String, usize) {
        let width = width.max(1);
        let first = (self.cursor + 1).saturating_sub(width);
        let end = (first + width).min(self.chars.len());
        let visible = self.chars[first..end].iter().collect();
        let col = self.chars[first..self.cursor]
            .iter()
            .map(|c| c.width().unwrap_or(0))
            .sum();
        (visible, col)
    }
}
