use std::{thread, time::Duration};

use ratatui::crossterm::event::{self, Event, KeyEvent, MouseButton, MouseEvent, MouseEventKind};
use tokio::sync::mpsc::UnboundedSender;

/// Semua yang bisa membangunkan loop utama.
pub enum Ev {
    Key(KeyEvent),
    Mouse(MouseEvent),
    Tick,
    Chunk(String),
    Done,
    Failed(String),
    Models {
        provider: &'static str,
        result: Result<Vec<String>, String>,
    },
}

pub type Tx = UnboundedSender<Ev>;

/// Thread terpisah yang membaca input terminal dan mengirim `Tick` tiap 50 ms (20 fps untuk animasi).
pub fn spawn_input(tx: Tx) {
    thread::spawn(move || loop {
        let ev = match event::poll(Duration::from_millis(50)) {
            Ok(true) => match event::read() {
                Ok(Event::Key(key)) => Ev::Key(key),
                // Cuma klik kiri dan scroll yang dipakai; gerakan mouse lain dibuang tanpa membangunkan loop.
                Ok(Event::Mouse(m)) => match m.kind {
                    MouseEventKind::Down(MouseButton::Left) | MouseEventKind::ScrollUp | MouseEventKind::ScrollDown => {
                        Ev::Mouse(m)
                    }
                    _ => continue,
                },
                Ok(_) => Ev::Tick,
                Err(_) => break,
            },
            Ok(false) => Ev::Tick,
            Err(_) => break,
        };
        if tx.send(ev).is_err() {
            break;
        }
    });
}
