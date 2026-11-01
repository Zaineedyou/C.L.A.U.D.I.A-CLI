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
    theme::{fg, mix, pulse, shimmer, ACCENT_B, AI, DIM, USER},
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

/// Pesan user: kartu dengan garis kiri yang membulat di ujungnya (╭ │ ╰).
fn user(msg: &Message, width: usize, out: &mut Vec<Line<'static>>) {
    let mid = Span::styled("│ ", fg(USER));
    let mut rows = vec![Line::from(Span::styled("╭", fg(USER)))];
    markdown::render(&msg.content, width.saturating_sub(2), &mid, &mut rows);
    rows.push(Line::from(Span::styled("╰", fg(USER))));
    out.extend(rows);
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
        // "Claudia · model · 12s · 54.3 tok/s": nama berwarna, sisanya redup (seperti OpenCode).
        let (name, rest) = match msg.tag.split_once(" · ") {
            Some((name, rest)) => (name.to_string(), format!(" · {rest}")),
            None => (msg.tag.clone(), String::new()),
        };
        out.push(Line::from(vec![indent, Span::styled(name, fg(AI)), Span::styled(rest, fg(DIM))]));
    }
}
