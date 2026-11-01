use ratatui::{
    prelude::*,
    widgets::{Block, BorderType, Clear, Padding, Paragraph},
};

use crate::{
    app::App,
    picker::{Item, Kind, Picker, POPUP_FRAMES},
    provider,
};

use super::{
    anim,
    theme::{bold, ease, fg, flow, flowing, lerp, mix, spinner, DIM, ERR, FG, HEAD, PANEL},
};

pub fn draw(f: &mut Frame, app: &App) {
    editor_popup(f, app);
    key_popup(f, app);
    picker_popup(f, app);
}

/// Dialog isi API key: input bertopeng (•), Enter simpan, Esc balik ke daftar provider.
fn key_popup(f: &mut Frame, app: &App) {
    let Some(entry) = &app.keyentry else { return };
    let p = &provider::ALL[entry.provider];
    let height = lerp(5, 9, ease(entry.age, POPUP_FRAMES));
    let area = centered(f.area(), 64, height);

    let mut head = vec![Span::raw(" ")];
    head.extend(flowing(&format!("Hubungkan {}", p.name), app.tick));
    head.push(Span::raw(" "));
    head.extend(anim::slashes(8, app.tick));
    head.push(Span::raw(" "));
    let block = Block::bordered()
        .border_type(BorderType::Rounded)
        .border_style(fg(flow(0.3, app.tick)))
        .style(Style::new().fg(FG).bg(PANEL))
        .padding(Padding::horizontal(1))
        .title(Line::from(head))
        .title_bottom(Span::styled(" enter simpan · ctrl+u hapus · esc balik ", fg(DIM)));
    let inner = block.inner(area);
    f.render_widget(Clear, area);
    f.render_widget(block, area);
    if inner.width < 6 || inner.height < 3 {
        return;
    }

    let hint = format!("atau set {} di environment", p.requirement());
    f.render_widget(Paragraph::new(Span::styled(hint, fg(DIM))), Rect::new(inner.x, inner.y, inner.width, 1));

    let input_area = Rect::new(inner.x, inner.y + 2, inner.width, 1);
    anim::fill(f, input_area, mix(PANEL, FG, 0.06));
    let (visible, col) = entry.input.window((inner.width as usize).saturating_sub(2));
    let shown = if entry.input.is_empty() {
        Span::styled("API key", fg(DIM))
    } else {
        Span::styled("•".repeat(visible.chars().count()), fg(FG))
    };
    f.render_widget(Paragraph::new(Line::from(vec![Span::raw(" "), shown])), input_area);

    if inner.height >= 5 {
        if let Some(error) = &entry.error {
            let rect = Rect::new(inner.x, inner.y + 4, inner.width, 1);
            f.render_widget(Paragraph::new(Span::styled(error.clone(), fg(ERR))), rect);
        }
    }
    f.set_cursor_position((input_area.x + 1 + col as u16, input_area.y));
}

/// Popup edit system prompt: teks banyak baris dengan kursor.
fn editor_popup(f: &mut Frame, app: &App) {
    let Some(editor) = &app.editor else { return };
    let height = lerp(5, 20, ease(editor.age, POPUP_FRAMES));
    let area = centered(f.area(), 76, height);

    let mut head = vec![Span::raw(" ")];
    head.extend(flowing("System prompt", app.tick));
    head.push(Span::raw(" "));
    head.extend(anim::slashes(8, app.tick));
    head.push(Span::raw(" "));
    let block = Block::bordered()
        .border_type(BorderType::Rounded)
        .border_style(fg(flow(0.3, app.tick)))
        .style(Style::new().fg(FG).bg(PANEL))
        .padding(Padding::horizontal(1))
        .title(Line::from(head))
        .title_bottom(Span::styled(" ctrl+s simpan · ctrl+r bawaan · ctrl+u kosong · esc batal ", fg(DIM)));
    let inner = block.inner(area);
    f.render_widget(Clear, area);
    f.render_widget(block, area);
    if inner.width < 2 || inner.height == 0 {
        return;
    }

    let (rows, cx, cy) = editor.layout(inner.width as usize);
    let visible = inner.height as usize;
    let top = (cy + 1).saturating_sub(visible);
    let lines: Vec<Line> = rows
        .into_iter()
        .skip(top)
        .take(visible)
        .map(|row| Line::from(Span::styled(row, fg(FG))))
        .collect();
    f.render_widget(Paragraph::new(lines), inner);
    f.set_cursor_position((inner.x + cx as u16, inner.y + (cy - top) as u16));
}

fn picker_popup(f: &mut Frame, app: &App) {
    let Some(picker) = &app.picker else { return };
    // Popup "tumbuh" dari kotak kecil ke ukuran penuh.
    let height = lerp(5, 24, ease(picker.age, POPUP_FRAMES));
    let area = centered(f.area(), 76, height);

    let title = match picker.kind {
        Kind::Provider => format!("Provider ({})", picker.items.len()),
        Kind::Model => format!("Model · {} ({})", app.cfg.current().name, picker.items.len()),
        Kind::Connect => format!("Hubungkan provider ({})", picker.items.len()),
        Kind::Command => format!("Perintah ({})", picker.items.len()),
    };
    // Judul ala Crush: teks gradien lalu ╱╱╱╱.
    let mut head = vec![Span::raw(" ")];
    head.extend(flowing(&title, app.tick));
    head.push(Span::raw(" "));
    head.extend(anim::slashes(8, app.tick));
    head.push(Span::raw(" "));
    let block = Block::bordered()
        .border_type(BorderType::Rounded)
        .border_style(fg(flow(0.3, app.tick)))
        .style(Style::new().fg(FG).bg(PANEL))
        .padding(Padding::horizontal(1))
        .title(Line::from(head))
        .title_bottom(Span::styled(" ↑↓ pilih · Enter ok · Esc tutup ", fg(DIM)));
    let inner = block.inner(area);
    f.render_widget(Clear, area);
    f.render_widget(block, area);

    let visible = picker.visible();
    let rows = (inner.height as usize).saturating_sub(2);
    let top = (picker.selected + 1).saturating_sub(rows);

    let mut lines = vec![
        search_line(picker, app.tick),
        Line::from(Span::styled("─".repeat(inner.width as usize), fg(DIM))),
    ];
    if visible.is_empty() {
        lines.push(empty_line(picker));
    }
    lines.extend(
        visible
            .iter()
            .enumerate()
            .skip(top)
            .take(rows)
            .map(|(i, item)| item_line(item, i == picker.selected)),
    );
    f.render_widget(Paragraph::new(lines), inner);
}

fn centered(screen: Rect, width: u16, height: u16) -> Rect {
    let w = width.min(screen.width.saturating_sub(2));
    let h = height.min(screen.height.saturating_sub(2));
    Rect::new(screen.x + (screen.width - w) / 2, screen.y + (screen.height - h) / 2, w, h)
}

fn search_line(picker: &Picker, tick: usize) -> Line<'static> {
    let query = if picker.filter.is_empty() {
        Span::styled("ketik buat nyaring…", fg(DIM))
    } else {
        Span::styled(picker.filter.clone(), fg(FG))
    };
    let mut spans = vec![Span::styled("⌕ ", fg(HEAD)), query];
    if picker.loading {
        spans.push(Span::styled(format!("  {} ngambil daftar…", spinner(tick)), fg(DIM)));
    }
    Line::from(spans)
}

fn empty_line(picker: &Picker) -> Line<'static> {
    let query = picker.filter.trim();
    let text = if picker.kind == Kind::Model && !query.is_empty() {
        format!("Enter: pakai \"{query}\" secara manual")
    } else {
        "nggak ada yang cocok".to_string()
    };
    Line::from(Span::styled(text, fg(DIM)))
}

fn item_line(item: &Item, selected: bool) -> Line<'static> {
    let style = if selected { bold(HEAD).bg(mix(PANEL, FG, 0.08)) } else { fg(FG) };
    let mut spans = vec![
        Span::styled(if selected { "▸ " } else { "  " }, style),
        Span::styled(item.label.clone(), style),
    ];
    if !item.hint.is_empty() {
        spans.push(Span::styled(format!("  {}", item.hint), fg(DIM)));
    }
    if item.current {
        spans.push(Span::styled("  ✓", fg(HEAD)));
    }
    Line::from(spans)
}
