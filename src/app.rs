use std::time::{Duration, Instant};

use tokio::sync::mpsc;

use crate::client::HeadroomClient;
use crate::model::StatsResponse;

pub enum AppEvent {
    Stats(StatsResponse),
    Error(String),
}

pub struct App {
    pub base_url: String,
    pub interval: Duration,
    pub stats: Option<StatsResponse>,
    pub last_error: Option<String>,
    pub last_refresh: Option<Instant>,
    pub should_quit: bool,
}

impl App {
    pub fn new(base_url: String, interval: Duration) -> Self {
        Self {
            base_url,
            interval,
            stats: None,
            last_error: None,
            last_refresh: None,
            should_quit: false,
        }
    }

    pub fn handle_event(&mut self, event: AppEvent) {
        match event {
            AppEvent::Stats(s) => {
                self.stats = Some(s);
                self.last_error = None;
                self.last_refresh = Some(Instant::now());
            }
            AppEvent::Error(e) => {
                self.last_error = Some(e);
                self.last_refresh = Some(Instant::now());
            }
        }
    }

    /// Time since last refresh, formatted as a string.
    pub fn since_refresh(&self) -> String {
        match self.last_refresh {
            Some(t) => {
                let secs = t.elapsed().as_secs();
                if secs < 60 {
                    format!("{secs}s ago")
                } else {
                    format!("{}m ago", secs / 60)
                }
            }
            None => "loading…".to_string(),
        }
    }
}

/// Spawns a background task that polls /stats every `interval` and sends results on `tx`.
/// Sending on `force_rx` triggers an immediate refresh.
pub fn spawn_poller(
    client: HeadroomClient,
    interval: Duration,
    tx: mpsc::UnboundedSender<AppEvent>,
    mut force_rx: mpsc::UnboundedReceiver<()>,
) {
    tokio::spawn(async move {
        loop {
            let event = match client.fetch_stats().await {
                Ok(s) => AppEvent::Stats(s),
                Err(e) => AppEvent::Error(e.to_string()),
            };
            if tx.send(event).is_err() {
                break;
            }

            // Sleep for `interval`, but wake early on force-refresh signal.
            tokio::select! {
                _ = tokio::time::sleep(interval) => {}
                _ = force_rx.recv() => {}
            }
        }
    });
}
