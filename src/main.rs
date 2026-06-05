mod app;
mod client;
mod model;
mod ui;

use std::time::Duration;

use anyhow::Result;
use clap::Parser;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use tokio::sync::mpsc;

use app::{App, spawn_poller};
use client::HeadroomClient;

#[derive(Parser)]
#[command(name = "headroom-stats", about = "Real-time TUI dashboard for the Headroom proxy", version)]
struct Args {
    /// Headroom proxy URL (default: $ANTHROPIC_BASE_URL or http://localhost:8787)
    #[arg(long)]
    url: Option<String>,

    /// Refresh interval in seconds
    #[arg(long, short, default_value = "5")]
    interval: u64,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    let base_url = args
        .url
        .or_else(|| std::env::var("ANTHROPIC_BASE_URL").ok())
        .unwrap_or_else(|| "http://localhost:8787".to_string());

    let interval = Duration::from_secs(args.interval);

    let client = HeadroomClient::new(base_url.clone());
    let (tx, mut rx) = mpsc::unbounded_channel();
    let (force_tx, force_rx) = mpsc::unbounded_channel::<()>();

    spawn_poller(client, interval, tx, force_rx);

    let mut terminal = ratatui::init();
    let mut app = App::new(base_url, interval);

    let result = run_loop(&mut terminal, &mut app, &mut rx, &force_tx).await;

    ratatui::restore();
    result
}

async fn run_loop(
    terminal: &mut ratatui::DefaultTerminal,
    app: &mut App,
    rx: &mut mpsc::UnboundedReceiver<app::AppEvent>,
    force_tx: &mpsc::UnboundedSender<()>,
) -> Result<()> {
    loop {
        // Drain all pending stats updates before drawing
        while let Ok(event) = rx.try_recv() {
            app.handle_event(event);
        }

        terminal.draw(|f| ui::render(f, app))?;

        // Poll for keyboard events with a short timeout to stay responsive
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc => {
                            app.should_quit = true;
                        }
                        KeyCode::Char('r') | KeyCode::Char('R') => {
                            let _ = force_tx.send(());
                        }
                        _ => {}
                    }
                }
            }
        }

        if app.should_quit {
            break;
        }

        // Yield to let tokio run the poller task
        tokio::task::yield_now().await;
    }

    Ok(())
}
