//! Kotak prompt ala OpenCode: panel dengan garis aksen di kiri.

use ratatui::{prelude::*, widgets::Paragraph};

use crate::app::App;

use super::{
    anim,
    theme::{bold, fg, flow, mix, pulse, ACCENT_A, ACCENT_B, AI, DIM, ERR, FG, OK, PANEL},
};

pub fn draw(f: &mut Frame, area: Rect, app: &App) {
    if area.width < 6 || area.height < 4 {
        return;
    }
    anim::fill(f, area, PANEL);

    // Garis kiri: berdenyut saat idle, mengalir saat balasan sedang datang.
    let bars: Vec<Line> = (0..area.height)
        .map(|row| {
            let color = if app.streaming {
                flow(row as f32 / area.height as f32, app.tick * 3)
            } else {
                mix(ACCENT_A, ACCENT_B, pulse(app.tick, 80.0))
            };
            Line::from(Span::styled("┃", fg(color)))
        })
        .collect();
    f.render_widget(Paragraph::new(bars), Rect::new(area.x, area.y, 1, area.height));

    let inner = Rect::new(area.x + 2, area.y + 1, area.width - 3, 2);
    let (visible, col) = app.input.window(inner.width as usize);
    let input = if app.input.is_empty() {
        Span::styled("Tulis pesan…  atau ketik / buat perintah", fg(DIM))
    } else {
        Span::styled(visible, fg(FG))
    };

    let p = app.cfg.current();
    let dot = fg(if app.cfg.is_ready(p) { OK } else { ERR });
    let info = Line::from(vec![
        Span::styled(app.cfg.name.clone(), bold(AI)),
        Span::raw("  "),
        Span::styled(app.cfg.model(), fg(FG)),
        Span::styled(format!("  {} ", p.name), fg(DIM)),
        Span::styled("●", dot),
    ]);
    f.render_widget(Paragraph::new(vec![Line::from(input), info]), inner);

    if app.picker.is_none() {
        f.set_cursor_position((inner.x + col as u16, inner.y));
    }
}
