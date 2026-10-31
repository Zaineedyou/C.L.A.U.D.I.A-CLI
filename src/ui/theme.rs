use std::f32::consts::TAU;

use ratatui::{
    style::{Color, Modifier, Style},
    text::Span,
};

pub const BG: Color = Color::Rgb(12, 12, 22);
pub const PANEL: Color = Color::Rgb(22, 22, 38);
pub const FG: Color = Color::Rgb(220, 222, 240);
pub const DIM: Color = Color::Rgb(110, 114, 145);
pub const USER: Color = Color::Rgb(255, 190, 110);
pub const AI: Color = Color::Rgb(180, 140, 255);
pub const HEAD: Color = Color::Rgb(120, 220, 255);
pub const OK: Color = Color::Rgb(120, 230, 160);
pub const ERR: Color = Color::Rgb(255, 100, 120);
pub const CODE_FG: Color = Color::Rgb(130, 230, 190);
pub const CODE_BG: Color = Color::Rgb(30, 32, 52);
pub const ACCENT_A: Color = Color::Rgb(150, 110, 255);
pub const ACCENT_B: Color = Color::Rgb(80, 220, 255);
pub const WHITE: Color = Color::Rgb(255, 255, 255);

const SPINNER: [&str; 10] = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

pub fn fg(color: Color) -> Style {
    Style::new().fg(color)
}

pub fn bold(color: Color) -> Style {
    fg(color).add_modifier(Modifier::BOLD)
}

pub fn spinner(tick: usize) -> &'static str {
    SPINNER[tick / 2 % SPINNER.len()]
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

/// Gradien ungu → cyan yang mengalir; `pos` 0..1 di sepanjang objek.
pub fn flow(pos: f32, tick: usize) -> Color {
    mix(ACCENT_A, ACCENT_B, 0.5 + 0.5 * (TAU * (pos - tick as f32 * 0.015)).sin())
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
    let center = (tick as f32 * 0.03) % 1.6 - 0.3;
    text.chars()
        .enumerate()
        .map(|(i, c)| {
            let glow = (1.0 - (i as f32 / n - center).abs() / 0.25).max(0.0);
            Span::styled(c.to_string(), fg(mix(base, WHITE, glow)))
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
