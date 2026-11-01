//! Baris tab sesi di paling atas, seperti OpenCode: judul memudar di ujung, tab aktif tebal, "+" di akhir.
//! Tiap tab dan "+" bisa diklik/disentuh; tinggi area klik dua baris supaya gampang kena jari.

use ratatui::{prelude::*, widgets::Paragraph};
use unicode_width::UnicodeWidthChar;

use crate::app::App;

use super::{
    anim,
    theme::{fg, mix, spinner, AI, BG, DIM, FG, WHITE},
};

/// Lebar minimum sebuah tab; kalau tidak muat, tab digulir ke sekitar tab aktif.
const MIN_W: usize = 12;
const MAX_W: usize = 28;
/// Judul yang kepanjangan memudar di sekian sel terakhir, bukan dipotong keras.
const FADE: usize = 4;
/// Lebar tombol "+" yang tampak.
const ADD_W: usize = 3;

/// Yang terkena klik di baris tab.
#[derive(Clone, Copy)]
pub enum Click {
    /// Tab ke-n.
    Tab(usize),
    /// Tombol "+" (sesi baru).
    New,
}

/// Susunan baris tab; dipakai bersama oleh penggambaran dan pengenalan klik.
struct Strip {
    titles: Vec<Option<String>>,
    has_empty: bool,
    start: usize,
    fit: usize,
    width: usize,
}

fn layout(area: Rect, app: &App) -> Strip {
    let count = app.tabs.len();
    let titles: Vec<Option<String>> = (0..count).map(|i| app.tab_title(i)).collect();
    // Sesi kosong tampil sebagai "+ New session"; kalau sudah ada, tombol "+" tidak perlu.
    let has_empty = titles.iter().any(|t| t.is_none());
    let add_w = if has_empty { 0 } else { ADD_W };
    let avail = (area.width as usize).saturating_sub(add_w);
    let fit = (avail / MIN_W).clamp(1, count);
    let start = app.active.saturating_sub(fit - 1).min(count - fit);
    let width = (avail / fit).min(MAX_W).max(1);
    Strip { titles, has_empty, start, fit, width }
}

/// Area klik: tiap tab yang terlihat, ditambah "+" kalau ada.
pub fn hits(area: Rect, app: &App) -> Vec<(Rect, Click)> {
    if area.width < 8 || area.height == 0 {
        return Vec::new();
    }
    let l = layout(area, app);
    let zone = Rect::new(area.x, area.y, area.width, 2);
    let mut out: Vec<(Rect, Click)> = (l.start..l.start + l.fit)
        .map(|i| {
            let x = area.x + ((i - l.start) * l.width) as u16;
            (Rect::new(x, area.y, l.width as u16, 2).intersection(zone), Click::Tab(i))
        })
        .collect();
    if !l.has_empty {
        let x = area.x + (l.fit * l.width) as u16;
        out.push((Rect::new(x, area.y, (ADD_W * 2) as u16, 2).intersection(zone), Click::New));
    }
    out
}

/// Potong `text` sampai `width` sel; kembalikan teks, lebar terpakai, dan apakah terpotong.
fn clip(text: &str, width: usize) -> (String, usize, bool) {
    let mut out = String::new();
    let mut used = 0;
    for ch in text.chars() {
        let w = ch.width().unwrap_or(0);
        if used + w > width {
            return (out, used, true);
        }
        out.push(ch);
        used += w;
    }
    (out, used, false)
}

pub fn draw(f: &mut Frame, area: Rect, app: &App) {
    if area.width < 8 || area.height == 0 {
        return;
    }
    let l = layout(area, app);
    let width = l.width;

    let mut spans: Vec<Span<'static>> = Vec::new();
    for i in l.start..l.start + l.fit {
        let active = i == app.active;
        let (label, empty) = match &l.titles[i] {
            Some(t) => (t.clone(), false),
            None => ("New session".to_string(), true),
        };
        let base = if active { FG } else { DIM };
        // Tab aktif tebal dan diberi kaca buram setelah digambar; yang lain transparan.
        let style = |color: Color| {
            let s = Style::new().fg(color);
            if active {
                s.add_modifier(Modifier::BOLD)
            } else {
                s
            }
        };

        let busy = active && app.streaming;
        let marker = if busy {
            format!("{} ", spinner(app.tick))
        } else if empty {
            "+ ".to_string()
        } else if active {
            "✦ ".to_string()
        } else {
            "  ".to_string()
        };
        spans.push(Span::styled(" ", style(base)));
        spans.push(Span::styled(marker, style(if busy { AI } else { base })));

        let room = width.saturating_sub(4);
        let (text, used, cut) = clip(&label, room);
        let chars = text.chars().count();
        for (k, ch) in text.chars().enumerate() {
            let from_end = chars - k;
            let color = if cut && from_end <= FADE {
                mix(base, BG, 0.2 + 0.7 * ((FADE - from_end) as f32 / (FADE - 1) as f32))
            } else {
                base
            };
            spans.push(Span::styled(ch.to_string(), style(color)));
        }
        spans.push(Span::styled(" ".repeat(room.saturating_sub(used) + 1), style(base)));
    }
    if !l.has_empty {
        spans.push(Span::styled(" + ", fg(DIM)));
    }
    f.render_widget(Paragraph::new(Line::from(spans)), area);
    let x = area.x + ((app.active - l.start) * width) as u16;
    anim::tint(f, Rect::new(x, area.y, width as u16, 1).intersection(area), WHITE, 0.6);
}
