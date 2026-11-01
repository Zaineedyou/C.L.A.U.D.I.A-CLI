//! Animasi yang digambar langsung ke buffer: logo, deret huruf acak, garis miring.

use ratatui::{prelude::*, widgets::Block};

use super::theme::{fg, flow, mix, ACCENT_A, BG, DIM, FG};

/// Huruf CLAUDIA, tinggi 3 baris, gaya blok seperti logo OpenCode.
const LETTERS: [[&str; 3]; 7] = [
    ["█▀▀▀", "█   ", "▀▀▀▀"], // C
    ["█   ", "█   ", "▀▀▀▀"], // L
    ["█▀▀█", "█▀▀█", "▀  ▀"], // A
    ["█  █", "█  █", "▀▀▀▀"], // U
    ["█▀▀▄", "█  █", "▀▀▀ "], // D
    ["▀█▀ ", " █  ", "▀█▀ "], // I
    ["█▀▀█", "█▀▀█", "▀  ▀"], // A
];

/// 7 huruf x 4 kolom + 6 spasi.
pub const LOGO_W: u16 = 34;
pub const LOGO_H: u16 = 3;

/// Logo: "CLAU" abu-abu, "DIA" gelap; kilau ungu menyapu bagian gelapnya.
pub fn logo(f: &mut Frame, x: u16, y: u16, tick: usize, alpha: f32) {
    let sweep = (tick as f32 * 0.02) % 1.6 - 0.3;
    let buf = f.buffer_mut();
    for (i, glyph) in LETTERS.iter().enumerate() {
        for (r, row) in glyph.iter().enumerate() {
            for (c, ch) in row.chars().enumerate() {
                if ch == ' ' {
                    continue;
                }
                let col = i * 5 + c;
                let color = if i < 4 {
                    DIM
                } else {
                    let pos = col as f32 / LOGO_W as f32;
                    let glow = (1.0 - (pos - sweep).abs() / 0.15).max(0.0);
                    mix(FG, ACCENT_A, glow)
                };
                if let Some(cell) = buf.cell_mut((x + col as u16, y + r as u16)) {
                    cell.set_char(ch).set_fg(mix(BG, color, alpha));
                }
            }
        }
    }
}

const GLYPHS: &[char] = &[
    '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'a', 'b', 'c', 'd', 'e', 'f', 'A', 'B', 'C', 'D', 'E', 'F', '~', '!',
    '@', '#', '$', '%', '^', '&', '*', '(', ')', '+', '=', '_',
];

fn hash(a: u32, b: u32) -> u32 {
    let mut h = a.wrapping_mul(374_761_393).wrapping_add(b.wrapping_mul(668_265_263));
    h = (h ^ (h >> 13)).wrapping_mul(1_274_126_177);
    h ^ (h >> 16)
}

/// Deret huruf acak ala Crush: tiap sel ganti glyph sepuluh kali per detik, warnanya gradien yang mengalir.
pub fn cycle(tick: usize, width: usize) -> Vec<Span<'static>> {
    (0..width)
        .map(|i| {
            let glyph = GLYPHS[hash(i as u32, (tick / 2) as u32) as usize % GLYPHS.len()];
            let pos = i as f32 / width.max(1) as f32;
            Span::styled(glyph.to_string(), fg(flow(pos, tick * 2)))
        })
        .collect()
}

/// Titik-titik yang bertambah tiap 400 ms (lebar tetap, jadi teks di sebelahnya tidak goyang).
pub fn dots(tick: usize) -> &'static str {
    ["   ", ".  ", ".. ", "..."][tick / 8 % 4]
}

/// Garis miring ╱╱╱ ala header Crush, gradiennya mengalir.
pub fn slashes(width: usize, tick: usize) -> Vec<Span<'static>> {
    (0..width)
        .map(|i| Span::styled("╱", fg(mix(BG, flow(i as f32 / width.max(1) as f32, tick), 0.55))))
        .collect()
}

/// Latar polos satu warna untuk sebuah area.
pub fn fill(f: &mut Frame, area: Rect, bg: Color) {
    f.render_widget(Block::new().style(Style::new().bg(bg)), area);
}
