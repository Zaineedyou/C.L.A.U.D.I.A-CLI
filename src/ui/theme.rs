use std::f32::consts::TAU;

use ratatui::{
    style::{Color, Modifier, Style},
    text::Span,
};

/// Pink pastel dasar; latar sebenarnya gradien (lihat `backdrop`), ini dipakai buat memudarkan teks.
pub const BG: Color = Color::Rgb(255, 228, 240);
/// Latar kartu: prompt, pesan user, blok kode, popup.
pub const PANEL: Color = Color::Rgb(255, 247, 251);
/// Latar kartu sidebar.
pub const RAISED: Color = Color::Rgb(255, 243, 249);
pub const FG: Color = Color::Rgb(62, 42, 72);
pub const DIM: Color = Color::Rgb(138, 108, 142);
pub const USER: Color = Color::Rgb(214, 84, 156);
pub const AI: Color = Color::Rgb(150, 100, 222);
pub const HEAD: Color = Color::Rgb(226, 118, 40);
pub const OK: Color = Color::Rgb(56, 160, 112);
pub const ERR: Color = Color::Rgb(214, 66, 92);
pub const CODE_FG: Color = Color::Rgb(40, 150, 156);
pub const CODE_BG: Color = Color::Rgb(246, 238, 255);
pub const ACCENT_A: Color = Color::Rgb(214, 84, 156);
pub const ACCENT_B: Color = Color::Rgb(90, 150, 232);

/// Warna-warni cerah buat teks dan garis yang bergerak pelan.
const RAINBOW: [Color; 6] = [
    Color::Rgb(236, 92, 150),
    Color::Rgb(242, 140, 96),
    Color::Rgb(226, 172, 48),
    Color::Rgb(64, 182, 132),
    Color::Rgb(78, 160, 228),
    Color::Rgb(150, 108, 236),
];

/// Warna pastel buat latar: dominan pink, diselingi peach, lilac, biru langit, dan mint.
const PASTEL: [Color; 8] = [
    Color::Rgb(255, 210, 230),
    Color::Rgb(255, 224, 214),
    Color::Rgb(255, 204, 226),
    Color::Rgb(236, 214, 255),
    Color::Rgb(210, 232, 255),
    Color::Rgb(214, 246, 232),
    Color::Rgb(240, 214, 250),
    Color::Rgb(255, 214, 236),
];

const SPINNER: [&str; 10] = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

pub fn fg(color: Color) -> Style {
    Style::new().fg(color)
}

pub fn bold(color: Color) -> Style {
    fg(color).add_modifier(Modifier::BOLD)
}

pub fn spinner(tick: usize) -> &'static str {
    SPINNER[tick / 3 % SPINNER.len()]
}

fn rgb(c: Color) -> (u8, u8, u8) {
    match c {
        Color::Rgb(r, g, b) => (r, g, b),
        _ => (255, 255, 255),
    }
}

/// Campur dua warna; `t` = 0 → `a`, 1 → `b`.
pub fn mix(a: Color, b: Color, t: f32) -> Color {
    let (a, b, t) = (rgb(a), rgb(b), t.clamp(0.0, 1.0));
    let ch = |x: u8, y: u8| (x as f32 + (y as f32 - x as f32) * t).round() as u8;
    Color::Rgb(ch(a.0, b.0), ch(a.1, b.1), ch(a.2, b.2))
}

/// Gelombang 0..1 yang naik-turun, satu siklus tiap `period` tick.
pub fn pulse(tick: usize, period: f32) -> f32 {
    0.5 + 0.5 * (TAU * tick as f32 / period).sin()
}

/// Warna di sepanjang palet melingkar; `pos` 0..1 (lewat dari itu berputar lagi).
fn palette(colors: &[Color], pos: f32) -> Color {
    let n = colors.len();
    let p = pos.rem_euclid(1.0) * n as f32;
    let i = (p.floor() as usize).min(n - 1);
    mix(colors[i], colors[(i + 1) % n], p - p.floor())
}

/// Pelangi yang mengalir pelan; `pos` 0..1 di sepanjang objek.
pub fn flow(pos: f32, tick: usize) -> Color {
    palette(&RAINBOW, pos * 0.6 - tick as f32 * 0.004)
}

/// Warna latar pastel di sel (x, y) dari layar `w` x `h`; bergeser sangat pelan.
pub fn backdrop(x: u16, y: u16, w: u16, h: u16, tick: usize) -> Color {
    let fx = x as f32 / w.max(1) as f32;
    let fy = y as f32 / h.max(1) as f32;
    palette(&PASTEL, fx * 0.35 + fy * 0.65 + tick as f32 * 0.0007)
}

/// Posisi animasi 0..1 dengan easing smoothstep.
pub fn ease(step: u8, max: u8) -> f32 {
    let t = step as f32 / max as f32;
    t * t * (3.0 - 2.0 * t)
}

pub fn lerp(a: u16, b: u16, t: f32) -> u16 {
    (a as f32 + (b as f32 - a as f32) * t).round() as u16
}

/// Teks dengan gradien yang bergerak.
pub fn flowing(text: &str, tick: usize) -> Vec<Span<'static>> {
    let n = text.chars().count().max(1) as f32;
    text.chars()
        .enumerate()
        .map(|(i, c)| Span::styled(c.to_string(), bold(flow(i as f32 / n, tick))))
        .collect()
}

/// Teks redup dengan kilau yang menyapu dari kiri ke kanan.
pub fn shimmer(text: &str, tick: usize, base: Color) -> Vec<Span<'static>> {
    let n = text.chars().count().max(1) as f32;
    let center = (tick as f32 * 0.012) % 1.6 - 0.3;
    text.chars()
        .enumerate()
        .map(|(i, c)| {
            let glow = (1.0 - (i as f32 / n - center).abs() / 0.25).max(0.0);
            Span::styled(c.to_string(), fg(mix(base, FG, glow)))
        })
        .collect()
}

/// Pudarkan warna teks ke latar; `alpha` 1 = utuh, 0 = hilang.
pub fn faded(spans: Vec<Span<'static>>, alpha: f32) -> Vec<Span<'static>> {
    spans
        .into_iter()
        .map(|mut s| {
            let color = s.style.fg.unwrap_or(FG);
            s.style = s.style.fg(mix(BG, color, alpha));
            s
        })
        .collect()
}
