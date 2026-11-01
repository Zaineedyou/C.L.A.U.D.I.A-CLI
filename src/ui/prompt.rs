//! Kartu prompt: bingkai bulat berwarna pelangi, baris model, dan petunjuk di bawahnya.

use ratatui::{prelude::*, widgets::Paragraph};

use crate::app::App;

use super::{
    anim, approx_tokens,
    theme::{bold, fg, AI, DIM, ERR, FG},
    Screen,
};

const PLACEHOLDERS: [&str; 4] = [
    "Perbaiki TODO di kode ini",
    "Apa tech stack proyek ini?",
    "Jelasin kode ini",
    "Cari bug di fungsi ini",
];

pub fn draw(f: &mut Frame, s: &Screen, app: &App, screen: Rect) {
    let area = s.prompt;
    if area.width < 10 || area.height < 4 {
        return;
    }
    // Kartu bulat dengan bingkai pelangi; geraknya sedikit lebih cepat saat balasan datang.
    anim::rounded_card(f, area, if app.streaming { app.tick * 2 } else { app.tick });

    let text_area = Rect::new(area.x + 2, area.y + 1, area.width - 4, 1);
    let meta_area = Rect::new(area.x + 2, area.y + 2, area.width - 4, 1);

    let (visible, col) = app.input.window(text_area.width as usize);
    let input = if app.input.is_empty() {
        let hint = format!("Tanya apa aja… \"{}\"", PLACEHOLDERS[app.tick / 200 % PLACEHOLDERS.len()]);
        Span::styled(hint.chars().take(text_area.width as usize).collect::<String>(), fg(DIM))
    } else {
        Span::styled(visible, fg(FG))
    };
    f.render_widget(Paragraph::new(Line::from(input)), text_area);

    let p = app.cfg.current();
    let mut meta = vec![
        Span::styled(app.cfg.name.clone(), fg(AI)),
        Span::styled(" · ", fg(DIM)),
        Span::styled(app.cfg.model(), fg(FG)),
        Span::styled(format!(" {}", p.name), fg(DIM)),
    ];
    if !app.cfg.is_ready(p) {
        meta.push(Span::styled(format!("  belum ada key ({})", p.requirement()), bold(ERR)));
    }
    f.render_widget(Paragraph::new(Line::from(meta)), meta_area);

    if app.picker.is_none() {
        f.set_cursor_position((text_area.x + col as u16, text_area.y));
    }

    if s.hint.bottom() > screen.bottom() {
        return;
    }
    // Petunjuk di bawah prompt: folder kerja di kiri, pintasan di kanan.
    let left = Span::styled(app.cwd.clone(), fg(DIM));
    f.render_widget(Paragraph::new(Line::from(left.clone())), s.hint);

    let mut right: Vec<Span<'static>> = Vec::new();
    if !app.messages.is_empty() {
        right.push(Span::styled(format!("{} token  ", approx_tokens(app)), fg(DIM)));
    }
    right.push(Span::styled("ctrl+p ", fg(FG)));
    right.push(Span::styled("commands  ", fg(DIM)));
    right.push(Span::styled("ctrl+o ", fg(FG)));
    right.push(Span::styled("model", fg(DIM)));
    let right_w: usize = right.iter().map(|sp| sp.content.chars().count()).sum();
    if left.content.chars().count() + right_w + 2 < s.hint.width as usize {
        f.render_widget(Paragraph::new(Line::from(right)).alignment(Alignment::Right), s.hint);
    }
}
