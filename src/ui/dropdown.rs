//! Dropdown perintah "/" di atas prompt, seperti OpenCode: baris terpilih berlatar ungu, deskripsi redup.

use ratatui::{
    prelude::*,
    widgets::{Clear, Paragraph},
};

use crate::app::App;

use super::{
    theme::{mix, AI, BG, DIM, FG, PANEL},
    Screen,
};

const MAX_ROWS: usize = 10;

pub fn draw(f: &mut Frame, s: &Screen, app: &App) {
    let matches = app.command_matches();
    if matches.is_empty() || s.prompt.width < 12 {
        return;
    }
    let rows = (matches.len().min(MAX_ROWS) as u16).min(s.prompt.y.saturating_sub(s.main.y));
    if rows == 0 {
        return;
    }
    let area = Rect::new(s.prompt.x, s.prompt.y - rows, s.prompt.width, rows);
    let room = (area.width as usize).saturating_sub(1);
    let sel = app.cmd_sel.min(matches.len() - 1);
    let top = (sel + 1).saturating_sub(rows as usize);
    let name_w = matches.iter().map(|c| c.name.len() + 1).max().unwrap_or(0) + 2;

    let lines: Vec<Line> = matches
        .iter()
        .enumerate()
        .skip(top)
        .take(rows as usize)
        .map(|(i, cmd)| {
            let selected = i == sel;
            let bg = if selected { AI } else { PANEL };
            let (name_color, desc_color) = if selected { (BG, BG) } else { (FG, DIM) };
            let name_part: String = format!(" {:<w$}", format!("/{}", cmd.name), w = name_w).chars().take(room).collect();
            let left = name_part.chars().count();
            let desc_part: String = cmd.desc.chars().take(room.saturating_sub(left)).collect();
            let pad = room.saturating_sub(left + desc_part.chars().count());
            Line::from(vec![
                Span::styled("│", Style::new().fg(mix(BG, DIM, 0.5)).bg(bg)),
                Span::styled(name_part, Style::new().fg(name_color).bg(bg)),
                Span::styled(desc_part, Style::new().fg(desc_color).bg(bg)),
                Span::styled(" ".repeat(pad), Style::new().bg(bg)),
            ])
        })
        .collect();
    f.render_widget(Clear, area);
    f.render_widget(Paragraph::new(lines), area);
}
