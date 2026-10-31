use ratatui::{prelude::*, widgets::Paragraph};

use crate::app::App;

use super::{
    anim,
    theme::{bold, fg, flowing, DIM, ERR, FG, OK, PANEL},
};

const SHORTCUTS: [&str; 4] = ["ctrl+p  provider", "ctrl+o  model", "ctrl+l  bersihkan", "esc     batalkan"];

pub fn draw(f: &mut Frame, area: Rect, app: &App) {
    if area.width < 12 {
        return;
    }
    anim::fill(f, area, PANEL);
    let inner = Rect::new(area.x + 2, area.y + 1, area.width - 4, area.height.saturating_sub(2));

    let p = app.cfg.current();
    let dot = fg(if app.cfg.is_ready(p) { OK } else { ERR });
    let tokens = app.messages.iter().map(|m| m.content.chars().count()).sum::<usize>() / 4;
    let heading = |text: &str| Line::from(Span::styled(text.to_string(), bold(FG)));
    let dim = |text: String| Line::from(Span::styled(text, fg(DIM)));

    let mut lines = vec![
        Line::from(flowing("C.L.A.U.D.I.A", app.tick)),
        dim(format!("CLI v{}", env!("CARGO_PKG_VERSION"))),
        Line::default(),
        heading("Sesi"),
        dim(format!("{} pesan", app.messages.len())),
        dim(format!("~{tokens} token")),
        Line::default(),
        heading("Provider"),
        Line::from(vec![Span::styled("● ", dot), Span::styled(p.name, fg(FG))]),
        dim(app.cfg.model()),
        Line::default(),
        heading("Pintasan"),
    ];
    lines.extend(SHORTCUTS.iter().map(|s| dim(s.to_string())));
    f.render_widget(Paragraph::new(lines), inner);
}
