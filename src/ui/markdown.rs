//! Markdown ringan untuk terminal: heading, bullet, **tebal**, `kode`, dan blok kode.

use ratatui::{
    style::{Modifier, Style},
    text::{Line, Span},
};
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

use super::theme::{bold, fg, mix, BG, CODE_BG, CODE_FG, DIM, FG, HEAD};

pub fn render(text: &str, width: usize, bar: &Span<'static>, out: &mut Vec<Line<'static>>) {
    let row = |spans: Vec<Span<'static>>| {
        Line::from(std::iter::once(bar.clone()).chain(spans).collect::<Vec<_>>())
    };
    let mut in_code = false;
    let lines: Vec<&str> = text.split('\n').collect();
    let mut i = 0;

    while i < lines.len() {
        let raw = lines[i];
        let trimmed = raw.trim_start();
        if let Some(lang) = trimmed.strip_prefix("```") {
            in_code = !in_code;
            let label = match (in_code, lang.trim()) {
                (true, "") => "┌─ kode".to_string(),
                (true, lang) => format!("┌─ {lang}"),
                (false, _) => "└─".to_string(),
            };
            out.push(row(vec![Span::styled(label, fg(DIM))]));
        } else if in_code {
            let code_style = Style::new().fg(CODE_FG).bg(CODE_BG);
            for piece in hard_wrap(&raw.replace('\t', "    "), width.saturating_sub(2).max(4)) {
                out.push(row(vec![Span::styled(format!(" {piece} "), code_style)]));
            }
        } else if is_table_line(raw) {
            let mut end = i;
            while end < lines.len() && is_table_line(lines[end]) {
                end += 1;
            }
            let parsed: Vec<Vec<String>> = lines[i..end].iter().map(|l| split_row(l)).collect();
            let has_header = parsed.len() > 1 && is_separator(&parsed[1]);
            let body: Vec<Vec<String>> = parsed.into_iter().filter(|r| !is_separator(r)).collect();
            if !body.is_empty() && render_table(&body, has_header, width, bar, out) {
                i = end;
                continue;
            }
            // Tidak muat sebagai tabel: tampilkan sebagai teks biasa.
            for piece in wrap(raw, width) {
                out.push(row(inline(&piece, fg(FG))));
            }
        } else {
            let (body, base) = classify(raw, trimmed);
            for piece in wrap(&body, width) {
                out.push(row(inline(&piece, base)));
            }
        }
        i += 1;
    }
}

fn is_table_line(line: &str) -> bool {
    let t = line.trim();
    t.len() > 1 && t.starts_with('|')
}

fn split_row(line: &str) -> Vec<String> {
    let t = line.trim();
    let t = t.strip_prefix('|').unwrap_or(t);
    let t = t.strip_suffix('|').unwrap_or(t);
    t.split('|').map(|cell| cell.trim().to_string()).collect()
}

/// Baris pemisah header, mis. `| --- | :---: |`.
fn is_separator(cells: &[String]) -> bool {
    !cells.is_empty() && cells.iter().all(|c| !c.is_empty() && c.chars().all(|ch| ch == '-' || ch == ':'))
}

/// Lebar tampil teks tanpa penanda `**` dan `` ` ``.
fn plain_width(text: &str) -> usize {
    UnicodeWidthStr::width(text.replace("**", "").replace('`', "").as_str())
}

/// Gambar tabel berbingkai; `false` kalau lebarnya tidak cukup.
fn render_table(
    rows: &[Vec<String>],
    has_header: bool,
    width: usize,
    bar: &Span<'static>,
    out: &mut Vec<Line<'static>>,
) -> bool {
    let cols = rows.iter().map(|r| r.len()).max().unwrap_or(0);
    if cols == 0 || width < 3 * cols + 1 + 3 * cols {
        return false;
    }
    let avail = width - (3 * cols + 1);
    let mut widths: Vec<usize> = (0..cols)
        .map(|c| rows.iter().filter_map(|r| r.get(c)).map(|cell| plain_width(cell)).max().unwrap_or(1).max(1))
        .collect();
    while widths.iter().sum::<usize>() > avail {
        let widest = (0..cols).max_by_key(|&c| widths[c]).unwrap_or(0);
        if widths[widest] <= 3 {
            return false;
        }
        widths[widest] -= 1;
    }

    let line_style = fg(mix(BG, DIM, 0.55));
    let rule = |left: &str, mid: &str, right: &str| -> Line<'static> {
        let mut text = String::from(left);
        for (c, w) in widths.iter().enumerate() {
            text.push_str(&"─".repeat(w + 2));
            text.push_str(if c + 1 == widths.len() { right } else { mid });
        }
        Line::from(vec![bar.clone(), Span::styled(text, line_style)])
    };

    out.push(rule("┌", "┬", "┐"));
    for (r, cells_row) in rows.iter().enumerate() {
        let base = if has_header && r == 0 { bold(FG) } else { fg(FG) };
        let cells: Vec<Vec<String>> =
            (0..cols).map(|c| wrap(cells_row.get(c).map(String::as_str).unwrap_or(""), widths[c])).collect();
        let height = cells.iter().map(|c| c.len()).max().unwrap_or(1);
        for line_no in 0..height {
            let mut spans = vec![bar.clone(), Span::styled("│", line_style)];
            for c in 0..cols {
                let piece = cells[c].get(line_no).map(String::as_str).unwrap_or("");
                spans.push(Span::raw(" "));
                spans.extend(inline(piece, base));
                let pad = widths[c].saturating_sub(plain_width(piece));
                spans.push(Span::raw(" ".repeat(pad + 1)));
                spans.push(Span::styled("│", line_style));
            }
            out.push(Line::from(spans));
        }
        out.push(if r + 1 == rows.len() { rule("└", "┴", "┘") } else { rule("├", "┼", "┤") });
    }
    true
}

/// Tentukan teks dan gaya dasar sebuah baris biasa.
fn classify(raw: &str, trimmed: &str) -> (String, Style) {
    if trimmed.starts_with('#') {
        (trimmed.trim_start_matches('#').trim_start().to_string(), bold(HEAD))
    } else if let Some(item) = trimmed.strip_prefix("- ").or_else(|| trimmed.strip_prefix("* ")) {
        (format!("• {item}"), fg(FG))
    } else {
        (raw.to_string(), fg(FG))
    }
}

fn inline(piece: &str, base: Style) -> Vec<Span<'static>> {
    let style_of = |bold: bool, code: bool| {
        if code {
            Style::new().fg(CODE_FG)
        } else if bold {
            Style::new().fg(HEAD).add_modifier(Modifier::BOLD)
        } else {
            base
        }
    };
    let flush = |spans: &mut Vec<Span<'static>>, buf: &mut String, style: Style| {
        if !buf.is_empty() {
            spans.push(Span::styled(std::mem::take(buf), style));
        }
    };

    let (mut spans, mut buf) = (Vec::new(), String::new());
    let (mut is_bold, mut is_code) = (false, false);
    let mut chars = piece.chars().peekable();
    while let Some(c) = chars.next() {
        let double_star = c == '*' && !is_code && chars.peek() == Some(&'*');
        if c == '`' || double_star {
            flush(&mut spans, &mut buf, style_of(is_bold, is_code));
            if c == '`' {
                is_code = !is_code;
            } else {
                chars.next();
                is_bold = !is_bold;
            }
        } else {
            buf.push(c);
        }
    }
    flush(&mut spans, &mut buf, style_of(is_bold, is_code));
    spans
}

/// Bungkus per kata; kata yang lebih panjang dari `width` dipotong paksa.
pub fn wrap(text: &str, width: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut cur = String::new();
    let mut cur_w = 0;
    for word in text.split(' ') {
        let w = UnicodeWidthStr::width(word);
        if cur_w > 0 && cur_w + 1 + w > width {
            lines.push(std::mem::take(&mut cur));
            cur_w = 0;
        }
        if w > width {
            for piece in hard_wrap(word, width) {
                if cur_w > 0 {
                    lines.push(std::mem::take(&mut cur));
                }
                cur_w = UnicodeWidthStr::width(piece.as_str());
                cur = piece;
            }
            continue;
        }
        if cur_w > 0 {
            cur.push(' ');
            cur_w += 1;
        }
        cur.push_str(word);
        cur_w += w;
    }
    lines.push(cur);
    lines
}

/// Potong per karakter tanpa menyentuh spasi (untuk blok kode).
fn hard_wrap(text: &str, width: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut cur = String::new();
    let mut cur_w = 0;
    for ch in text.chars() {
        let w = ch.width().unwrap_or(0);
        if cur_w + w > width && cur_w > 0 {
            lines.push(std::mem::take(&mut cur));
            cur_w = 0;
        }
        cur.push(ch);
        cur_w += w;
    }
    lines.push(cur);
    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wraps_on_word_boundaries() {
        assert_eq!(wrap("halo dunia semua", 10), ["halo dunia", "semua"]);
    }

    #[test]
    fn splits_overlong_words() {
        assert_eq!(wrap("abcdefgh", 3), ["abc", "def", "gh"]);
    }

    #[test]
    fn renders_a_table_with_borders() {
        let mut out = Vec::new();
        render("| a | b |\n| --- | --- |\n| 1 | 2 |", 30, &Span::raw(""), &mut out);
        let text: Vec<String> = out.iter().map(|l| l.to_string()).collect();
        assert_eq!(text.first().map(String::as_str), Some("┌───┬───┐"));
        assert_eq!(text.last().map(String::as_str), Some("└───┴───┘"));
    }
}
