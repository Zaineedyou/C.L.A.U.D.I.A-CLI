//! Sidebar sesi ala OpenCode: judul, konteks, dan folder kerja di dasar.

use ratatui::{prelude::*, widgets::Paragraph};

use crate::{app::App, chat::Role};

use super::{
    anim, approx_tokens,
    theme::{bold, fg, DIM, ERR, FG, OK, RAISED},
};

/// Judul sesi: baris pertama pesan user pertama, dipotong sesuai lebar.
fn title(app: &App, width: usize) -> String {
    let first = app
        .messages
        .iter()
        .find(|m| m.role == Role::User)
        .and_then(|m| m.content.lines().next())
        .unwrap_or("Sesi baru");
    let mut out: String = first.chars().take(width).collect();
    if first.chars().count() > width && width > 1 {
        out.pop();
        out.push('…');
    }
    out
}

pub fn draw(f: &mut Frame, area: Rect, app: &App) {
    if area.width < 14 || area.height < 5 {
        return;
    }
    // Kartu bulat yang melayang di atas latar pastel.
    let card = Rect::new(area.x + 1, area.y, area.width - 2, area.height - 1);
    anim::rounded_card(f, card, RAISED, app.tick);
    let inner = Rect::new(card.x + 2, card.y + 1, card.width - 4, card.height.saturating_sub(2));
    let w = inner.width as usize;

    let p = app.cfg.current();
    let dot = fg(if app.cfg.is_ready(p) { OK } else { ERR });
    let dim = |text: String| Line::from(Span::styled(text, fg(DIM)));

    let lines = vec![
        Line::from(Span::styled(title(app, w), bold(FG))),
        Line::from(anim::slashes(w, app.tick)),
        Line::default(),
        Line::from(Span::styled("Context", bold(FG))),
        dim(format!("{} tokens", approx_tokens(app))),
        dim(format!("{} pesan", app.messages.len())),
        Line::default(),
        Line::from(Span::styled("Provider", bold(FG))),
        Line::from(vec![Span::styled("● ", dot), Span::styled(p.name, fg(FG))]),
        dim(app.cfg.model()),
    ];
    f.render_widget(Paragraph::new(lines), inner);

    if inner.height > 1 {
        let foot = Rect::new(inner.x, inner.bottom() - 1, inner.width, 1);
        f.render_widget(Paragraph::new(Span::styled(app.cwd.clone(), fg(DIM))), foot);
    }
}
