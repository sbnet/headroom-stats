use anyhow::{Context, Result};
use reqwest::Client;

use crate::model::{HealthResponse, StatsHistoryResponse, StatsResponse};

pub struct HeadroomClient {
    client: Client,
    base_url: String,
}

impl HeadroomClient {
    pub fn new(base_url: String) -> Self {
        Self {
            client: Client::new(),
            base_url: base_url.trim_end_matches('/').to_string(),
        }
    }

    pub async fn fetch_stats(&self) -> Result<StatsResponse> {
        let url = format!("{}/stats", self.base_url);
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

        resp.json::<StatsResponse>()
            .await
            .context("failed to parse /stats response")
    }

    pub async fn fetch_stats_history(&self) -> Result<StatsHistoryResponse> {
        let url = format!("{}/stats-history", self.base_url);
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

        resp.json::<StatsHistoryResponse>()
            .await
            .context("failed to parse /stats-history response")
    }

    pub async fn fetch_health(&self) -> Result<HealthResponse> {
        let url = format!("{}/health", self.base_url);
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

        resp.json::<HealthResponse>()
            .await
            .context("failed to parse /health response")
    }
}
