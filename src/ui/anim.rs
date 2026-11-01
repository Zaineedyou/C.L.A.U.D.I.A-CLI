//! Animasi yang digambar langsung ke buffer: latar pastel, logo, kilau, kartu bulat.

use ratatui::prelude::*;

use super::theme::{backdrop as backdrop_color, fg, flow, mix, BG, FG, WHITE};

/// Huruf CLAUDIA besar: tinggi 5 baris, lebar 6 kolom, sudut dibulatkan pakai setengah blok.
const LETTERS: [[&str; 5]; 7] = [
    ["▄████▄", "██    ", "██    ", "██    ", "▀████▀"], // C
    ["██    ", "██    ", "██    ", "██    ", "██████"], // L
    ["▄████▄", "██  ██", "██████", "██  ██", "██  ██"], // A
    ["██  ██", "██  ██", "██  ██", "██  ██", "▀████▀"], // U
    ["█████▄", "██  ██", "██  ██", "██  ██", "█████▀"], // D
    ["██████", "  ██  ", "  ██  ", "  ██  ", "██████"], // I
    ["▄████▄", "██  ██", "██████", "██  ██", "██  ██"], // A
];

/// 7 huruf x 6 kolom + 6 spasi.
pub const LOGO_W: u16 = 48;
/// 5 baris huruf + 1 baris cadangan untuk gelombang.
pub const LOGO_H: u16 = 6;

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

/// Logo besar: tiap huruf naik-turun satu baris mengikuti gelombang, dan warnanya pelangi yang mengalir.
pub fn logo(f: &mut Frame, x: u16, y: u16, tick: usize, alpha: f32) {
    let buf = f.buffer_mut();
    let t = tick as f32 * 0.08;
    for (i, glyph) in LETTERS.iter().enumerate() {
        let drop = (((t - i as f32 * 0.9).sin() + 1.0) / 2.0).round() as u16;
        for (r, row) in glyph.iter().enumerate() {
            for (c, ch) in row.chars().enumerate() {
                if ch == ' ' {
                    continue;
                }
                let col = i * 7 + c;
                let color = flow(col as f32 / LOGO_W as f32, tick * 2);
                if let Some(cell) = buf.cell_mut((x + col as u16, y + r as u16 + drop)) {
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
            let slow = tick / 4 * 4;
            let twinkle = 0.5 + 0.5 * (slow as f32 * 0.03 + (h % 97) as f32).sin();
            let color = flow((h % 100) as f32 / 100.0, slow);
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

/// Kaca buram: simbol lama dihapus dan latar di bawahnya dipucatkan ke putih, jadi kartu tetap
/// berwarna mengikuti latar pastel tapi teks gelap mudah dibaca.
pub fn frost(f: &mut Frame, area: Rect, amount: f32) {
    let buf = f.buffer_mut();
    for y in area.top()..area.bottom() {
        for x in area.left()..area.right() {
            if let Some(cell) = buf.cell_mut((x, y)) {
                let bg = mix(cell.bg, WHITE, amount);
                cell.set_char(' ').set_bg(bg);
            }
        }
    }
}

/// Hanya menggeser warna latar ke `color`; simbol dan warna teks tidak berubah.
pub fn tint(f: &mut Frame, area: Rect, color: Color, amount: f32) {
    let buf = f.buffer_mut();
    for y in area.top()..area.bottom() {
        for x in area.left()..area.right() {
            if let Some(cell) = buf.cell_mut((x, y)) {
                let bg = mix(cell.bg, color, amount);
                cell.set_bg(bg);
            }
        }
    }
}

/// Kartu bulat: bagian dalam berkaca buram, bingkai ╭─╮ berwarna pelangi mengalir. Sel bingkai
/// tidak diisi, jadi sudutnya terlihat membulat dan bingkai pas di tepi kartu.
pub fn rounded_card(f: &mut Frame, area: Rect, tick: usize) {
    if area.width < 3 || area.height < 3 {
        return;
    }
    frost(f, Rect::new(area.x + 1, area.y + 1, area.width - 2, area.height - 2), 0.6);
    let (w, h) = (area.width, area.height);
    let total = (w + h) as f32;
    let buf = f.buffer_mut();
    let mut put = |x: u16, y: u16, ch: char, pos: f32| {
        if let Some(cell) = buf.cell_mut((x, y)) {
            cell.set_char(ch).set_fg(flow(pos, tick));
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
