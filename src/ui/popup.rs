use ratatui::{
    prelude::*,
    widgets::{Block, BorderType, Clear, Padding, Paragraph},
};

use crate::{
    app::App,
    picker::{Item, Kind, Picker, POPUP_FRAMES},
};

use super::{
    anim,
    theme::{bold, ease, fg, flow, flowing, lerp, spinner, CODE_BG, DIM, FG, HEAD, PANEL},
};

pub fn draw(f: &mut Frame, app: &App) {
    let Some(picker) = &app.picker else { return };
    // Popup "tumbuh" dari kotak kecil ke ukuran penuh.
    let height = lerp(5, 24, ease(picker.age, POPUP_FRAMES));
    let area = centered(f.area(), 76, height);

    let title = match picker.kind {
        Kind::Provider => format!("Provider ({})", picker.items.len()),
        Kind::Model => format!("Model · {} ({})", app.cfg.current().name, picker.items.len()),
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
    let style = if selected { bold(HEAD).bg(CODE_BG) } else { fg(FG) };
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
