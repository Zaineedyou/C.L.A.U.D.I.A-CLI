//! Animasi yang digambar langsung ke buffer: logo, bintang, scanner.

use ratatui::{prelude::*, widgets::Block};

use super::theme::{fg, flow, mix, ACCENT_A, ACCENT_B, BG, WHITE};

const LETTERS: [[&str; 5]; 7] = [
    [" ████", "█    ", "█    ", "█    ", " ████"], // C
    ["█    ", "█    ", "█    ", "█    ", "█████"], // L
    [" ███ ", "█   █", "█████", "█   █", "█   █"], // A
    ["█   █", "█   █", "█   █", "█   █", " ███ "], // U
    ["████ ", "█   █", "█   █", "█   █", "████ "], // D
    ["█████", "  █  ", "  █  ", "  █  ", "█████"], // I
    [" ███ ", "█   █", "█████", "█   █", "█   █"], // A
];

pub const LOGO_W: u16 = 42;
/// 5 baris huruf + 1 baris bayangan.
pub const LOGO_H: u16 = 6;

fn grid() -> Vec<Vec<bool>> {
    (0..5)
        .map(|row| {
            let mut cells: Vec<bool> = LETTERS
                .iter()
                .flat_map(|l| l[row].chars().map(|c| c == '█').chain(std::iter::once(false)))
                .collect();
            cells.pop();
            cells
        })
        .collect()
}

/// Logo CLAUDIA: gradien mengalir, kilau menyapu, dan bayangan lembut.
pub fn logo(f: &mut Frame, x: u16, y: u16, tick: usize, alpha: f32) {
    let g = grid();
    let width = g[0].len();
    let on = |r: usize, c: usize| g.get(r).and_then(|row| row.get(c)).copied().unwrap_or(false);
    let sweep = (tick as f32 * 0.02) % 1.5 - 0.25;
    let buf = f.buffer_mut();

    for r in 0..LOGO_H as usize {
        for c in 0..=width {
            let (ch, color) = if on(r, c) {
                let pos = c as f32 / width as f32;
                let glow = (1.0 - (pos - sweep).abs() / 0.12).max(0.0);
                ('█', mix(flow(pos, tick), WHITE, glow * 0.8))
            } else if r > 0 && c > 0 && on(r - 1, c - 1) {
                ('░', mix(BG, ACCENT_A, 0.3))
            } else {
                continue;
            };
            if let Some(cell) = buf.cell_mut((x + c as u16, y + r as u16)) {
                cell.set_char(ch).set_fg(mix(BG, color, alpha));
            }
        }
    }
}

/// Bintang berkelap-kelip di latar; posisinya tetap (hash dari koordinat).
pub fn stars(f: &mut Frame, area: Rect, tick: usize, alpha: f32) {
    if alpha <= 0.0 {
        return;
    }
    let buf = f.buffer_mut();
    for y in area.top()..area.bottom() {
        for x in area.left()..area.right() {
            let mut h = (x as u32).wrapping_mul(374_761_393).wrapping_add((y as u32).wrapping_mul(668_265_263));
            h = (h ^ (h >> 13)).wrapping_mul(1_274_126_177);
            h ^= h >> 16;
            if h % 41 != 0 {
                continue;
            }
            let glyph = match (h / 41) % 6 {
                0..=2 => '·',
                3 | 4 => '⋆',
                _ => '✦',
            };
            let twinkle = 0.5 + 0.5 * (tick as f32 * 0.08 + (h % 97) as f32).sin();
            let color = mix(BG, Color::Rgb(190, 200, 255), twinkle * 0.7 * alpha);
            if let Some(cell) = buf.cell_mut((x, y)) {
                cell.set_char(glyph).set_fg(color);
            }
        }
    }
}

/// Scanner ala Knight Rider: kepala terang dengan ekor memudar, bolak-balik.
pub fn scanner(tick: usize, width: usize) -> Vec<Span<'static>> {
    let cycle = 2 * (width - 1);
    let p = tick / 2 % cycle;
    let pos = if p < width { p } else { cycle - p };
    (0..width)
        .map(|i| match i.abs_diff(pos) {
            0 => Span::styled("■", fg(flow(i as f32 / width as f32, tick))),
            1 => Span::styled("■", fg(mix(ACCENT_A, BG, 0.4))),
            2 => Span::styled("■", fg(mix(ACCENT_A, BG, 0.7))),
            _ => Span::styled("·", fg(mix(ACCENT_A, BG, 0.85))),
        })
        .collect()
}

/// Latar polos satu warna untuk sebuah area.
pub fn fill(f: &mut Frame, area: Rect, bg: Color) {
    f.render_widget(Block::new().style(Style::new().bg(bg)), area);
}
