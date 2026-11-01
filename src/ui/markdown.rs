//! Markdown ringan untuk terminal: heading, bullet, **tebal**, `kode`, dan blok kode.

use ratatui::{
    style::{Modifier, Style},
    text::{Line, Span},
};
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

use super::theme::{bold, fg, CODE_BG, CODE_FG, DIM, FG, HEAD};

pub fn render(text: &str, width: usize, bar: &Span<'static>, out: &mut Vec<Line<'static>>) {
    let row = |spans: Vec<Span<'static>>| {
        Line::from(std::iter::once(bar.clone()).chain(spans).collect::<Vec<_>>())
    };
    let mut in_code = false;

    for raw in text.split('\n') {
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
        } else {
            let (body, base) = classify(raw, trimmed);
            for piece in wrap(&body, width) {
                out.push(row(inline(&piece, base)));
            }
        }
    }
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
}
