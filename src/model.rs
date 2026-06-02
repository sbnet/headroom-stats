#![allow(dead_code)]

use std::collections::HashMap;

use serde::Deserialize;
use serde_json::Value;

#[derive(Debug, Deserialize, Default, Clone)]
pub struct StatsResponse {
    #[serde(default)]
    pub requests: RequestStats,
    #[serde(default)]
    pub tokens: TokenStats,
    #[serde(default)]
    pub overhead: LatencyStats,
    #[serde(default)]
    pub prefix_cache: Value,
    #[serde(default)]
    pub router: RouterStats,
    #[serde(default)]
    pub toin: Value,
    #[serde(default)]
    pub compressions_by_strategy: HashMap<String, u64>,
    #[serde(default)]
    pub tokens_saved_by_strategy: HashMap<String, u64>,
    #[serde(default)]
    pub cost: Value,
}

#[derive(Debug, Deserialize, Default, Clone)]
pub struct RequestStats {
    #[serde(default)]
    pub total: u64,
    #[serde(default)]
    pub cached: u64,
    #[serde(default)]
    pub failed: u64,
    #[serde(default)]
    pub rate_limited: u64,
    #[serde(default)]
    pub by_model: HashMap<String, u64>,
}

#[derive(Debug, Deserialize, Default, Clone)]
pub struct TokenStats {
    #[serde(default)]
    pub input: u64,
    #[serde(default)]
    pub output: u64,
    #[serde(default)]
    pub saved: u64,
    #[serde(default)]
    pub savings_percent: f64,
}

#[derive(Debug, Deserialize, Default, Clone)]
pub struct LatencyStats {
    #[serde(default)]
    pub average_ms: f64,
    #[serde(default)]
    pub min_ms: f64,
    #[serde(default)]
    pub max_ms: f64,
}

#[derive(Debug, Deserialize, Default, Clone)]
pub struct RouterStats {
    #[serde(default)]
    pub route_counts: HashMap<String, u64>,
}

// ── /stats-history ──────────────────────────────────────────────────────────

#[derive(Debug, Deserialize, Default, Clone)]
pub struct StatsHistoryResponse {
    #[serde(default)]
    pub display_session: DisplaySession,
    #[serde(default)]
    pub lifetime: LifetimeStats,
    #[serde(default)]
    pub history: Vec<HistoryPoint>,
}

#[derive(Debug, Deserialize, Default, Clone)]
pub struct DisplaySession {
    #[serde(default)]
    pub requests: u64,
    #[serde(default)]
    pub tokens_saved: u64,
    #[serde(default)]
    pub compression_savings_usd: f64,
    #[serde(default)]
    pub total_input_tokens: u64,
    #[serde(default)]
    pub savings_percent: f64,
    #[serde(default)]
    pub started_at: String,
    #[serde(default)]
    pub last_activity_at: String,
}

#[derive(Debug, Deserialize, Default, Clone)]
pub struct LifetimeStats {
    #[serde(default)]
    pub requests: u64,
    #[serde(default)]
    pub tokens_saved: u64,
    #[serde(default)]
    pub compression_savings_usd: f64,
    #[serde(default)]
    pub total_input_tokens: u64,
    #[serde(default)]
    pub total_input_cost_usd: f64,
}

#[derive(Debug, Deserialize, Default, Clone)]
pub struct HistoryPoint {
    #[serde(default)]
    pub total_tokens_saved: u64,
    #[serde(default)]
    pub compression_savings_usd: f64,
}

// ── /health ─────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize, Default, Clone)]
pub struct HealthResponse {
    #[serde(default)]
    pub version: String,
    #[serde(default)]
    pub uptime_seconds: f64,
    #[serde(default)]
    pub config: Option<ProxyConfig>,
    #[serde(default)]
    pub rust_core: String,
}

#[derive(Debug, Deserialize, Default, Clone)]
pub struct ProxyConfig {
    #[serde(default)]
    pub backend: String,
    #[serde(default)]
    pub optimize: bool,
    #[serde(default)]
    pub cache: bool,
    #[serde(default)]
    pub rate_limit: bool,
    #[serde(default)]
    pub memory: bool,
    #[serde(default)]
    pub learn: bool,
}

/// Extracts a f64 from a nested JSON value by dot-separated path.
pub fn jval_f64(v: &Value, path: &str) -> f64 {
    path.split('.').fold(Some(v), |acc, key| acc?.get(key))
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0)
}

/// Extracts a u64 from a nested JSON value by dot-separated path.
pub fn jval_u64(v: &Value, path: &str) -> u64 {
    path.split('.').fold(Some(v), |acc, key| acc?.get(key))
        .and_then(|v| v.as_u64())
        .unwrap_or(0)
}
