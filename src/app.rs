use std::time::{Duration, Instant};

use tokio::sync::mpsc;

use crate::client::HeadroomClient;
use crate::model::{HealthResponse, StatsHistoryResponse, StatsResponse};

pub struct AppData {
    pub stats: Option<StatsResponse>,
    pub history: Option<StatsHistoryResponse>,
    pub health: Option<HealthResponse>,
    pub error: Option<String>,
}

pub enum AppEvent {
    Data(AppData),
}

pub struct App {
    pub base_url: String,
    pub interval: Duration,
    pub stats: Option<StatsResponse>,
    pub stats_history: Option<StatsHistoryResponse>,
    pub health: Option<HealthResponse>,
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
            stats_history: None,
            health: None,
            last_error: None,
            last_refresh: None,
            should_quit: false,
        }
    }

    pub fn handle_event(&mut self, event: AppEvent) {
        let AppEvent::Data(data) = event;
        if let Some(s) = data.stats {
            self.stats = Some(s);
        }
        if let Some(h) = data.history {
            self.stats_history = Some(h);
        }
        if let Some(h) = data.health {
            self.health = Some(h);
        }
        self.last_error = data.error;
        self.last_refresh = Some(Instant::now());
    }

    pub fn since_refresh(&self) -> String {
        match self.last_refresh {
            Some(t) => {
                let secs = t.elapsed().as_secs();
                if secs < 60 { format!("{secs}s ago") } else { format!("{}m ago", secs / 60) }
            }
            None => "loading…".to_string(),
        }
    }
}

/// Spawns a background task that polls /stats, /stats-history and /health in parallel.
/// Sending on `force_rx` triggers an immediate refresh.
pub fn spawn_poller(
    client: HeadroomClient,
    interval: Duration,
    tx: mpsc::UnboundedSender<AppEvent>,
    mut force_rx: mpsc::UnboundedReceiver<()>,
) {
    tokio::spawn(async move {
        loop {
            let (stats_res, history_res, health_res) = tokio::join!(
                client.fetch_stats(),
                client.fetch_stats_history(),
                client.fetch_health(),
            );

            // Collect the first error encountered, keep whatever succeeded.
            let error = stats_res.as_ref().err()
                .or(history_res.as_ref().err())
                .or(health_res.as_ref().err())
                .map(|e| e.to_string());

            let data = AppData {
                stats: stats_res.ok(),
                history: history_res.ok(),
                health: health_res.ok(),
                error,
            };

            if tx.send(AppEvent::Data(data)).is_err() {
                break;
            }

            tokio::select! {
                _ = tokio::time::sleep(interval) => {}
                _ = force_rx.recv() => {}
            }
        }
    });
}
