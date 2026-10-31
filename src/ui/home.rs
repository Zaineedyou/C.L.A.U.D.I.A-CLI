//! Layar awal: logo, judul yang diketik, dan tip yang berganti.

use ratatui::{prelude::*, widgets::Paragraph};

use crate::{app::App, provider};

use super::{
    anim,
    theme::{bold, faded, fg, flowing, ACCENT_B, DIM},
    Screen,
};

const TIPS: [&str; 5] = [
    "Ctrl+P ganti provider, Ctrl+O ganti model",
    "Ketik / buat lihat daftar perintah",
    "/key <kunci> set API key buat sesi ini",
    "Esc membatalkan balasan yang lagi jalan",
    "PgUp / PgDn buat gulir riwayat",
];

pub fn draw(f: &mut Frame, s: &Screen, app: &App, alpha: f32) {
    let area = s.main;
    let logo_x = area.x + area.width.saturating_sub(anim::LOGO_W) / 2;
    anim::logo(f, logo_x, s.home_top, app.tick, alpha);

    let title = "C.L.A.U.D.I.A-CLI";
    let typed = (app.tick / 2).min(title.chars().count());
    let spans = flowing(title, app.tick).into_iter().take(typed).collect();
    centered(f, area, s.home_top + anim::LOGO_H + 1, title.chars().count(), faded(spans, alpha));

    let version = format!("v{} · {} provider", env!("CARGO_PKG_VERSION"), provider::ALL.len());
    let width = version.chars().count();
    centered(f, area, s.home_top + anim::LOGO_H + 2, width, faded(vec![Span::styled(version, fg(DIM))], alpha));

    let tip = TIPS[app.tick / 120 % TIPS.len()];
    let spans = vec![Span::styled("● Tip  ", bold(ACCENT_B)), Span::styled(tip, fg(DIM))];
    centered(f, area, s.prompt.bottom() + 1, 7 + tip.chars().count(), faded(spans, alpha));
}

/// Taruh teks di tengah berdasarkan lebar penuhnya, jadi tidak bergeser saat diketik.
fn centered(f: &mut Frame, area: Rect, y: u16, width: usize, spans: Vec<Span<'static>>) {
    if y >= area.bottom() {
        return;
    }
    let w = (width as u16).min(area.width);
    let x = area.x + (area.width - w) / 2;
    f.render_widget(Paragraph::new(Line::from(spans)), Rect::new(x, y, w, 1));
}
