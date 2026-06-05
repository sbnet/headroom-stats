use std::time::Duration;

use anyhow::{Context, Result};
use reqwest::Client;
use serde::de::DeserializeOwned;

use crate::model::{HealthResponse, StatsHistoryResponse, StatsResponse};

const CONNECT_TIMEOUT: Duration = Duration::from_secs(5);
const REQUEST_TIMEOUT: Duration = Duration::from_secs(10);

pub struct HeadroomClient {
    client: Client,
    base_url: String,
}

impl HeadroomClient {
    pub fn new(base_url: String) -> Self {
        let client = Client::builder()
            .connect_timeout(CONNECT_TIMEOUT)
            .timeout(REQUEST_TIMEOUT)
            .build()
            .expect("failed to build HTTP client");

        Self {
            client,
            base_url: base_url.trim_end_matches('/').to_string(),
        }
    }

    async fn fetch<T: DeserializeOwned>(&self, path: &str) -> Result<T> {
        let url = format!("{}{path}", self.base_url);
        let resp = self
            .client
            .get(&url)
            .send()
            .await
            .with_context(|| format!("failed to connect to {url}"))?;

        let status = resp.status();
        if !status.is_success() {
            anyhow::bail!("server returned {status} for {url}");
        }

        resp.json::<T>()
            .await
            .with_context(|| format!("failed to parse response from {url}"))
    }

    pub async fn fetch_stats(&self) -> Result<StatsResponse> {
        self.fetch("/stats").await
    }

    pub async fn fetch_stats_history(&self) -> Result<StatsHistoryResponse> {
        self.fetch("/stats-history").await
    }

    pub async fn fetch_health(&self) -> Result<HealthResponse> {
        self.fetch("/health").await
    }
}
