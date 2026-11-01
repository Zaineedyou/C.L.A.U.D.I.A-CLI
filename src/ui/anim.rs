//! Animasi yang digambar langsung ke buffer: latar pastel, logo, kilau, kartu bulat.

use ratatui::prelude::*;

use super::theme::{backdrop as backdrop_color, fg, flow, mix, BG, FG};

/// Huruf CLAUDIA, tinggi 3 baris, sudut membulat.
const LETTERS: [[&str; 3]; 7] = [
    ["╭───", "│   ", "╰───"], // C
    ["│   ", "│   ", "╰───"], // L
    ["╭──╮", "├──┤", "╵  ╵"], // A
    ["╷  ╷", "│  │", "╰──╯"], // U
    ["╭─╮ ", "│  │", "╰─╯ "], // D
    ["╶┬╴ ", " │  ", "╶┴╴ "], // I
    ["╭──╮", "├──┤", "╵  ╵"], // A
];

/// 7 huruf x 4 kolom + 6 spasi.
pub const LOGO_W: u16 = 34;
pub const LOGO_H: u16 = 3;

/// Latar gradien pastel (pink, peach, lilac, biru langit, mint) yang bergeser pelan.
pub fn backdrop(f: &mut Frame, area: Rect, tick: usize) {
    let buf = f.buffer_mut();
    for y in area.top()..area.bottom() {
        for x in area.left()..area.right() {
            if let Some(cell) = buf.cell_mut((x, y)) {
                let bg = backdrop_color(x - area.x, y - area.y, area.width, area.height, tick);
                cell.set_char(' ').set_bg(bg).set_fg(FG);
            }
        }
    }
}

/// Logo: tiap huruf berwarna pelangi yang mengalir pelan.
pub fn logo(f: &mut Frame, x: u16, y: u16, tick: usize, alpha: f32) {
    let buf = f.buffer_mut();
    for (i, glyph) in LETTERS.iter().enumerate() {
        for (r, row) in glyph.iter().enumerate() {
            for (c, ch) in row.chars().enumerate() {
                if ch == ' ' {
                    continue;
                }
                let col = i * 5 + c;
                let color = flow(col as f32 / LOGO_W as f32, tick);
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

/// Kilau kecil warna-warni yang berkelip pelan di latar layar awal; posisinya tetap.
pub fn sparkles(f: &mut Frame, area: Rect, tick: usize, alpha: f32) {
    if alpha <= 0.0 {
        return;
    }
    let buf = f.buffer_mut();
    for y in area.top()..area.bottom() {
        for x in area.left()..area.right() {
            let h = hash(x as u32, (y as u32) ^ 0x9e37);
            if h % 47 != 0 {
                continue;
            }
            let glyph = match (h / 47) % 4 {
                0 | 1 => '·',
                2 => '✦',
                _ => '✧',
            };
            let twinkle = 0.5 + 0.5 * (tick as f32 * 0.03 + (h % 97) as f32).sin();
            let color = flow((h % 100) as f32 / 100.0, tick);
            if let Some(cell) = buf.cell_mut((x, y)) {
                let base = cell.bg;
                cell.set_char(glyph).set_fg(mix(base, color, twinkle * 0.85 * alpha));
            }
        }
    }
}

/// Deret huruf acak ala Crush: tiap sel ganti glyph sekitar empat kali per detik, warnanya pelangi pelan.
pub fn cycle(tick: usize, width: usize) -> Vec<Span<'static>> {
    (0..width)
        .map(|i| {
            let glyph = GLYPHS[hash(i as u32, (tick / 5) as u32) as usize % GLYPHS.len()];
            let pos = i as f32 / width.max(1) as f32;
            Span::styled(glyph.to_string(), fg(flow(pos, tick)))
        })
        .collect()
}

/// Titik-titik yang bertambah tiap 500 ms (lebar tetap, jadi teks di sebelahnya tidak goyang).
pub fn dots(tick: usize) -> &'static str {
    ["   ", ".  ", ".. ", "..."][tick / 10 % 4]
}

/// Garis miring ╱╱╱ ala header Crush, warnanya pelangi pudar.
pub fn slashes(width: usize, tick: usize) -> Vec<Span<'static>> {
    (0..width)
        .map(|i| Span::styled("╱", fg(mix(BG, flow(i as f32 / width.max(1) as f32, tick), 0.6))))
        .collect()
}

/// Latar polos satu warna; simbol lama di area itu ikut dihapus.
pub fn fill(f: &mut Frame, area: Rect, bg: Color) {
    let buf = f.buffer_mut();
    for y in area.top()..area.bottom() {
        for x in area.left()..area.right() {
            if let Some(cell) = buf.cell_mut((x, y)) {
                cell.set_char(' ').set_bg(bg);
            }
        }
    }
}

/// Kartu bulat: isi `bg` dengan bingkai ╭─╮ yang warnanya pelangi mengalir.
pub fn rounded_card(f: &mut Frame, area: Rect, bg: Color, tick: usize) {
    if area.width < 3 || area.height < 3 {
        return;
    }
    fill(f, area, bg);
    let (w, h) = (area.width, area.height);
    let total = (w + h) as f32;
    let buf = f.buffer_mut();
    let mut put = |x: u16, y: u16, ch: char, pos: f32| {
        if let Some(cell) = buf.cell_mut((x, y)) {
            cell.set_char(ch).set_fg(flow(pos, tick)).set_bg(bg);
        }
    };
    for dx in 0..w {
        let (top, bottom) = match dx {
            0 => ('╭', '╰'),
            d if d == w - 1 => ('╮', '╯'),
            _ => ('─', '─'),
        };
        put(area.x + dx, area.y, top, dx as f32 / total);
        put(area.x + dx, area.y + h - 1, bottom, (dx + h) as f32 / total);
    }
    for dy in 1..h - 1 {
        put(area.x, area.y + dy, '│', dy as f32 / total);
        put(area.x + w - 1, area.y + dy, '│', (w + dy) as f32 / total);
    }
}
