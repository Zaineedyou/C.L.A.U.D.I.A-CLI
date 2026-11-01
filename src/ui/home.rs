//! Layar awal ala OpenCode: logo di tengah, prompt di bawahnya, versi di pojok kanan bawah.

use ratatui::{prelude::*, widgets::Paragraph};

use crate::app::App;

use super::{
    anim,
    theme::{faded, fg, DIM},
    Screen,
};

pub fn draw(f: &mut Frame, s: &Screen, app: &App, alpha: f32) {
    let area = s.main;
    let logo_x = area.x + area.width.saturating_sub(anim::LOGO_W) / 2;
    anim::logo(f, logo_x, s.home_top, app.tick, alpha);

    let full = f.area();
    if full.width < 44 || full.height < 8 {
        return;
    }
    let version = env!("CARGO_PKG_VERSION");
    let width = version.chars().count() as u16;
    let rect = Rect::new(full.right().saturating_sub(width + 2), full.bottom() - 1, width, 1);
    let spans = faded(vec![Span::styled(version, fg(DIM))], alpha);
    f.render_widget(Paragraph::new(Line::from(spans)), rect);
}
