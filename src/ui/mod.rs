mod anim;
mod dropdown;
mod home;
mod markdown;
mod messages;
mod popup;
mod prompt;
mod sidebar;
mod tabs;
mod theme;

use ratatui::{prelude::*, widgets::Paragraph};

use crate::app::{App, Notice, SLIDE_FRAMES};

pub use tabs::Click;

use theme::{ease, fg, lerp, shimmer, DIM, ERR, HEAD};

const PROMPT_H: u16 = 4;
/// Prompt + baris petunjuk.
const BLOCK_H: u16 = PROMPT_H + 1;
/// Dari atas konten layar awal sampai kotak prompt: logo lalu dua baris kosong.
const HOME_PROMPT_OFFSET: u16 = anim::LOGO_H + 2;
const HOME_H: u16 = HOME_PROMPT_OFFSET + BLOCK_H;
const SIDEBAR_W: u16 = 42;
const SIDEBAR_MIN_SCREEN: u16 = 106;

/// Pembagian area layar. Dihitung ulang tiap frame dari `app.slide`, jadi
/// prompt meluncur dari tengah ke bawah dan sidebar melebar mulus.
pub struct Screen {
    pub t: f32,
    pub tabs: Rect,
    pub main: Rect,
    pub sidebar: Option<Rect>,
    pub messages: Rect,
    pub status: Rect,
    pub prompt: Rect,
    pub hint: Rect,
    pub home_top: u16,
}

impl Screen {
    fn new(area: Rect, slide: u8) -> Self {
        let t = ease(slide, SLIDE_FRAMES);
        // Baris paling atas dipakai tab sesi; sisanya jadi badan layar.
        let tabs = Rect::new(area.x, area.y, area.width, area.height.min(1));
        let body = Rect::new(area.x, area.y + tabs.height, area.width, area.height - tabs.height);
        let side_w = if body.width >= SIDEBAR_MIN_SCREEN { lerp(0, SIDEBAR_W, t) } else { 0 };
        let main = Rect { width: body.width - side_w, ..body };
        let sidebar = (side_w > 0).then(|| Rect::new(main.right(), body.y, side_w, body.height));

        let home_w = main.width.saturating_sub(4).min(75);
        let session_w = main.width.saturating_sub(5);
        let width = lerp(home_w, session_w, t);
        let x = lerp(main.x + (main.width - home_w) / 2, main.x + 3, t);

        let home_top = main.y + main.height.saturating_sub(HOME_H) / 2;
        let bottom_y = main.bottom().saturating_sub(BLOCK_H);
        let home_y = (home_top + HOME_PROMPT_OFFSET).min(bottom_y);
        let y = lerp(home_y, bottom_y, t).max(main.y);

        let gap = y.saturating_sub(1).max(main.y);
        Self {
            t,
            tabs,
            main,
            sidebar,
            messages: Rect::new(main.x + 3, main.y, main.width.saturating_sub(5), gap - main.y),
            status: Rect::new(main.x + 3, gap, main.width.saturating_sub(5), 1),
            prompt: Rect::new(x, y, width, PROMPT_H.min(main.height)),
            hint: Rect::new(x, y + PROMPT_H, width, 1),
            home_top,
        }
    }
}

/// Perkiraan jumlah token seluruh percakapan (4 karakter per token).
fn approx_tokens(app: &App) -> usize {
    app.messages.iter().map(|m| m.content.chars().count()).sum::<usize>() / 4
}

/// Area baris tab yang bisa diklik (tiap tab dan tombol "+").
pub fn tab_hits(area: Rect, app: &App) -> Vec<(Rect, Click)> {
    tabs::hits(area, app)
}

pub fn draw(f: &mut Frame, app: &mut App) {
    let area = f.area();
    let s = Screen::new(area, app.slide);
    app.tab_area = s.tabs;

    anim::backdrop(f, area, app.tick);
    tabs::draw(f, s.tabs, app);
    if app.messages.is_empty() {
        anim::sparkles(f, area, app.tick, 1.0 - s.t);
        home::draw(f, &s, app, 1.0 - s.t);
    }
    chat(f, s.messages, app);
    status(f, s.status, app);
    prompt::draw(f, &s, app, area);
    dropdown::draw(f, &s, app);
    if let Some(side) = s.sidebar {
        sidebar::draw(f, side, app);
    }
    popup::draw(f, app);
}

fn chat(f: &mut Frame, area: Rect, app: &mut App) {
    if app.messages.is_empty() || area.height == 0 {
        app.scroll = 0;
        return;
    }
    let lines = messages::lines(app, area.width as usize);
    let height = area.height as usize;
    app.scroll = app.scroll.min(lines.len().saturating_sub(height));
    let end = lines.len() - app.scroll;
    f.render_widget(Paragraph::new(lines[end.saturating_sub(height)..end].to_vec()), area);
}

/// Baris tepat di atas prompt: notifikasi, scanner saat menunggu balasan, atau petunjuk gulir.
fn status(f: &mut Frame, area: Rect, app: &App) {
    let spans: Vec<Span<'static>> = match (&app.notice, app.streaming, app.scroll) {
        (Some(Notice::Error(e)), ..) => vec![Span::styled(format!("✖ {e}"), fg(ERR))],
        (Some(Notice::Info(i)), ..) => vec![Span::styled(format!("✦ {i}"), fg(HEAD))],
        (None, true, _) => {
            let mut spans = anim::cycle(app.tick, 12);
            spans.push(Span::raw(" "));
            spans.extend(shimmer(&format!("{} lagi nulis", app.cfg.name), app.tick, DIM));
            spans.push(Span::styled(anim::dots(app.tick), fg(DIM)));
            spans.push(Span::styled(" esc batal", fg(DIM)));
            spans
        }
        (None, false, scroll) if scroll > 0 => vec![Span::styled("↓ End: balik ke bawah", fg(DIM))],
        _ => return,
    };
    f.render_widget(Paragraph::new(Line::from(spans)), area);
}
