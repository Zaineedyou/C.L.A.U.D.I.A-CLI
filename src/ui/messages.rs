use ratatui::{
    style::Style,
    text::{Line, Span},
};

use crate::{
    app::App,
    chat::{Message, Role},
};

use super::{
    anim, markdown,
    theme::{fg, mix, pulse, shimmer, ACCENT_B, AI, DIM, PANEL, USER},
};

/// Seluruh percakapan sebagai baris yang sudah di-wrap untuk lebar `width`.
pub fn lines(app: &App, width: usize) -> Vec<Line<'static>> {
    let mut out = Vec::new();
    let last = app.messages.len().saturating_sub(1);
    for (i, msg) in app.messages.iter().enumerate() {
        match msg.role {
            Role::User => user(msg, width, &mut out),
            _ => assistant(app, msg, i == last, width, &mut out),
        }
        out.push(Line::default());
    }
    out
}

/// Pesan user: panel dengan garis aksen di kiri, seperti di OpenCode.
fn user(msg: &Message, width: usize, out: &mut Vec<Line<'static>>) {
    let bar = Span::styled("┃ ", fg(USER));
    let mut rows = vec![Line::from(bar.clone())];
    markdown::render(&msg.content, width.saturating_sub(2), &bar, &mut rows);
    rows.push(Line::from(bar));
    out.extend(rows.into_iter().map(|row| panel(row, width)));
}

/// Beri latar panel dan isi sisa lebar baris.
fn panel(mut line: Line<'static>, width: usize) -> Line<'static> {
    let used = line.width();
    for span in &mut line.spans {
        if span.style.bg.is_none() {
            span.style = span.style.bg(PANEL);
        }
    }
    line.spans.push(Span::styled(" ".repeat(width.saturating_sub(used)), Style::new().bg(PANEL)));
    line
}

fn assistant(app: &App, msg: &Message, last: bool, width: usize, out: &mut Vec<Line<'static>>) {
    let indent = Span::raw("  ");
    let streaming = app.streaming && last;

    if streaming && msg.content.is_empty() {
        let mut spans = vec![indent];
        spans.extend(anim::cycle(app.tick, 10));
        spans.push(Span::raw(" "));
        spans.extend(shimmer(&format!("{} lagi mikir", app.cfg.name), app.tick, DIM));
        spans.push(Span::styled(anim::dots(app.tick), fg(DIM)));
        out.push(Line::from(spans));
        return;
    }

    let start = out.len();
    markdown::render(&msg.content, width.saturating_sub(2), &indent, out);
    if streaming && out.len() > start {
        let cursor = Span::styled("▌", fg(mix(DIM, ACCENT_B, pulse(app.tick, 12.0))));
        if let Some(line) = out.last_mut() {
            line.spans.push(cursor);
        }
    }
    if !streaming {
        out.push(Line::from(vec![indent, Span::styled("▣ ", fg(AI)), Span::styled(msg.tag.clone(), fg(DIM))]));
    }
}
