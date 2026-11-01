mod api;
mod app;
mod chat;
mod config;
mod editor;
mod event;
mod input;
mod keyentry;
mod picker;
mod provider;
mod ui;

use std::{io, time::Duration};

use ratatui::DefaultTerminal;
use tokio::sync::mpsc::{self, UnboundedReceiver};

use crate::{app::App, config::Config, event::Ev};

#[tokio::main]
async fn main() -> io::Result<()> {
    let client = reqwest::Client::builder()
        .connect_timeout(Duration::from_secs(15))
        .build()
        .map_err(io::Error::other)?;
    let (tx, rx) = mpsc::unbounded_channel();
    let mut app = App::new(Config::load(), client, tx.clone());

    let mut terminal = ratatui::init();
    // Mouse/sentuhan dipakai buat klik tab sesi dan scroll.
    let _ = ratatui::crossterm::execute!(io::stdout(), ratatui::crossterm::event::EnableMouseCapture);
    event::spawn_input(tx);
    let result = run(&mut terminal, &mut app, rx).await;
    let _ = ratatui::crossterm::execute!(io::stdout(), ratatui::crossterm::event::DisableMouseCapture);
    ratatui::restore();
    result
}

async fn run(
    terminal: &mut DefaultTerminal,
    app: &mut App,
    mut rx: UnboundedReceiver<Ev>,
) -> io::Result<()> {
    while !app.quit {
        // Kursor disembunyikan selama layar digambar ulang supaya tidak terlihat melompat ke mana-mana.
        terminal.hide_cursor()?;
        terminal.draw(|f| ui::draw(f, app))?;
        let Some(ev) = rx.recv().await else { break };
        app.on_event(ev);
        while let Ok(ev) = rx.try_recv() {
            app.on_event(ev);
        }
    }
    Ok(())
}
