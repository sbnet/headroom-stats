#![allow(dead_code)]

use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    symbols,
    text::{Line, Span},
    widgets::{Axis, Block, Borders, Cell, Chart, Dataset, GraphType, Paragraph, Row, Table},
};

use crate::app::App;
use crate::model::{jval_f64, jval_u64};

pub fn render(frame: &mut Frame, app: &App) {
    let area = frame.area();

    let rows = Layout::vertical([
        Constraint::Length(1), // title
        Constraint::Length(5), // summary: display_session + lifetime + live session
        Constraint::Min(4),    // per-model table
        Constraint::Length(6), // sparkline
        Constraint::Length(5), // cache | overhead | proxy status
        Constraint::Length(5), // content router | TOIN
        Constraint::Length(1), // help
    ])
    .split(area);

    render_title(frame, app, rows[0]);
    render_summary(frame, app, rows[1]);
    render_models(frame, app, rows[2]);
    render_sparkline(frame, app, rows[3]);
    render_cache_overhead_status(frame, app, rows[4]);
    render_router_toin(frame, app, rows[5]);
    render_help(frame, rows[6]);
}

// ── Helpers ──────────────────────────────────────────────────────────────────

fn label(s: &str) -> Span<'static> {
    Span::styled(s.to_string(), Style::default().fg(Color::Cyan))
}

fn val(s: impl Into<String>) -> Span<'static> {
    Span::styled(s.into(), Style::default().fg(Color::White))
}

fn good(s: impl Into<String>) -> Span<'static> {
    Span::styled(s.into(), Style::default().fg(Color::Green))
}

fn warn(s: impl Into<String>) -> Span<'static> {
    Span::styled(s.into(), Style::default().fg(Color::Yellow))
}

fn dim(s: impl Into<String>) -> Span<'static> {
    Span::styled(s.into(), Style::default().fg(Color::DarkGray))
}

fn fmt_num(n: u64) -> String {
    let s = n.to_string();
    let mut result = String::new();
    for (i, c) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.push(',');
        }
        result.push(c);
    }
    result.chars().rev().collect()
}

fn fmt_uptime(secs: f64) -> String {
    let s = secs as u64;
    if s < 60 {
        format!("{}s", s)
    } else if s < 3600 {
        format!("{}m{}s", s / 60, s % 60)
    } else {
        format!("{}h{}m", s / 3600, (s % 3600) / 60)
    }
}

/// "2026-06-02T21:26:04Z" → "02/06 21:26"
fn fmt_iso_short(iso: &str) -> String {
    let day = iso.get(8..10).unwrap_or("??");
    let month = iso.get(5..7).unwrap_or("??");
    let time = iso.get(11..16).unwrap_or("??:??");
    format!("{}/{} {}", day, month, time)
}

// ── Sections ─────────────────────────────────────────────────────────────────

fn render_title(frame: &mut Frame, app: &App, area: Rect) {
    let err_marker = if app.last_error.is_some() { "  [ERR]" } else { "" };
    let title = format!(" Headroom Stats  {}{}", app.base_url, err_marker);

    let (version_str, uptime_str) = app
        .health
        .as_ref()
        .map(|h| (format!("  v{}", h.version), format!("  up {}", fmt_uptime(h.uptime_seconds))))
        .unwrap_or_default();

    let refresh = format!("  ↻{}s  {}", app.interval.as_secs(), app.since_refresh());

    let line = Line::from(vec![
        Span::styled(title, Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        dim(version_str),
        dim(uptime_str),
        dim(refresh),
    ]);
    frame.render_widget(Paragraph::new(line), area);
}

fn render_summary(frame: &mut Frame, app: &App, area: Rect) {
    let block = Block::default().borders(Borders::ALL).title(" Summary ");
    let inner = block.inner(area);
    frame.render_widget(block, area);

    // No data at all yet
    if app.stats.is_none() && app.stats_history.is_none() {
        let msg = if let Some(err) = &app.last_error {
            Line::from(vec![
                Span::styled("Connection error: ", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
                Span::raw(err.clone()),
            ])
        } else {
            Line::from(dim("Connecting…"))
        };
        frame.render_widget(Paragraph::new(msg), inner);
        return;
    }

    let mut lines = vec![];

    // Line 1: display_session (persisted, matches headroom perf)
    if let Some(hist) = &app.stats_history {
        let ds = &hist.display_session;
        let pct_color = if ds.savings_percent >= 15.0 {
            Color::Green
        } else if ds.savings_percent >= 5.0 {
            Color::Yellow
        } else {
            Color::White
        };
        let since = if ds.started_at.is_empty() {
            String::new()
        } else {
            format!("  since {}", fmt_iso_short(&ds.started_at))
        };
        lines.push(Line::from(vec![
            label("Session  "),
            val(fmt_num(ds.requests)),
            dim(" reqs  "),
            val(fmt_num(ds.tokens_saved)),
            dim(" tokens saved  "),
            Span::styled(
                format!("{:.1}%", ds.savings_percent),
                Style::default().fg(pct_color).add_modifier(Modifier::BOLD),
            ),
            dim("  compression "),
            good(format!("~${:.2}", ds.compression_savings_usd)),
            dim(since),
        ]));

        // Line 2: lifetime totals
        let lt = &hist.lifetime;
        lines.push(Line::from(vec![
            label("All-time "),
            val(fmt_num(lt.requests)),
            dim(" reqs  "),
            val(fmt_num(lt.tokens_saved)),
            dim(" tokens saved  "),
            good(format!("~${:.2}", lt.compression_savings_usd)),
            dim(" compression savings"),
        ]));
    }

    // Line 3: live proxy session counters from /stats
    if let Some(stats) = &app.stats {
        let r = &stats.requests;
        lines.push(Line::from(vec![
            label("Live     "),
            val(fmt_num(r.total)),
            dim(" reqs  cached: "),
            val(r.cached.to_string()),
            dim("  failed: "),
            if r.failed > 0 { warn(r.failed.to_string()) } else { val("0".to_string()) },
            dim("  rate-limited: "),
            val(r.rate_limited.to_string()),
        ]));
    }

    frame.render_widget(Paragraph::new(lines), inner);
}

fn render_models(frame: &mut Frame, app: &App, area: Rect) {
    let block = Block::default().borders(Borders::ALL).title(" Per-model breakdown ");
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let Some(stats) = &app.stats else { return };

    let hs = Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD);
    let header = Row::new(vec![
        Cell::from("Model").style(hs),
        Cell::from("Reqs").style(hs),
        Cell::from("Tokens sent").style(hs),
        Cell::from("Saved").style(hs),
        Cell::from("Reduction").style(hs),
        Cell::from("$/MTok").style(hs),
        Cell::from("~Savings").style(hs),
    ])
    .bottom_margin(1);

    let mut model_names: Vec<&String> = stats.requests.by_model.keys().collect();
    model_names.sort();

    let rows: Vec<Row> = model_names
        .iter()
        .map(|model| {
            let req_count = stats.requests.by_model.get(*model).copied().unwrap_or(0);
            let cost_model = stats.cost.get("per_model").and_then(|pm| pm.get(model.as_str()));

            let (tokens_sent, tokens_saved, pct) = if let Some(cm) = cost_model {
                (jval_u64(cm, "tokens_sent"), jval_u64(cm, "tokens_saved"), jval_f64(cm, "reduction_pct"))
            } else {
                (0, 0, 0.0)
            };

            let pct_color = if pct >= 15.0 { Color::Green } else if pct >= 5.0 { Color::Yellow } else { Color::White };

            let (price_str, savings_str) = match model_price_per_mtok(model) {
                Some(price) => (format!("${:.2}", price), format!("~${:.2}", tokens_saved as f64 * price / 1_000_000.0)),
                None => ("n/a".to_string(), "n/a".to_string()),
            };

            Row::new(vec![
                Cell::from(model.as_str().to_string()),
                Cell::from(req_count.to_string()),
                Cell::from(fmt_num(tokens_sent)),
                Cell::from(fmt_num(tokens_saved)),
                Cell::from(format!("{:.1}%", pct)).style(Style::default().fg(pct_color)),
                Cell::from(price_str).style(Style::default().fg(Color::DarkGray)),
                Cell::from(savings_str).style(Style::default().fg(Color::Green)),
            ])
        })
        .collect();

    let widths = [
        Constraint::Min(26),
        Constraint::Length(6),
        Constraint::Length(13),
        Constraint::Length(11),
        Constraint::Length(10),
        Constraint::Length(7),
        Constraint::Length(9),
    ];

    let table = Table::new(rows, widths).header(header).column_spacing(1);
    frame.render_widget(table, inner);
}

fn render_sparkline(frame: &mut Frame, app: &App, area: Rect) {
    let block = Block::default().borders(Borders::ALL).title(" Savings trend (tokens saved, cumulative) ");
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let Some(hist) = &app.stats_history else { return };
    if hist.history.is_empty() { return }

    // Keep last 60 points to stay readable at any terminal width
    let points: Vec<(f64, f64)> = hist.history.iter().rev().take(60).rev()
        .enumerate()
        .map(|(i, p)| (i as f64, p.total_tokens_saved as f64))
        .collect();

    let max_val = points.iter().map(|p| p.1).fold(0.0_f64, f64::max);
    if max_val == 0.0 { return }

    let dataset = Dataset::default()
        .marker(symbols::Marker::Braille)
        .graph_type(GraphType::Line)
        .style(Style::default().fg(Color::Green))
        .data(&points);

    let chart = Chart::new(vec![dataset])
        .x_axis(Axis::default().bounds([0.0, points.len() as f64]))
        .y_axis(
            Axis::default()
                .bounds([0.0, max_val * 1.05])
                .labels(vec![
                    Span::styled("0", Style::default().fg(Color::DarkGray)),
                    Span::styled(fmt_num(max_val as u64), Style::default().fg(Color::DarkGray)),
                ]),
        );

    frame.render_widget(chart, inner);
}

fn render_cache_overhead_status(frame: &mut Frame, app: &App, area: Rect) {
    let cols = Layout::horizontal([
        Constraint::Percentage(33),
        Constraint::Percentage(33),
        Constraint::Percentage(34),
    ])
    .split(area);

    render_cache(frame, app, cols[0]);
    render_overhead(frame, app, cols[1]);
    render_proxy_status(frame, app, cols[2]);
}

fn render_cache(frame: &mut Frame, app: &App, area: Rect) {
    let block = Block::default().borders(Borders::ALL).title(" Prefix cache ");
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let Some(stats) = &app.stats else { return };
    let pc = &stats.prefix_cache;

    let hit_rate = jval_f64(pc, "totals.hit_rate");
    let read_tokens = jval_u64(pc, "totals.cache_read_tokens");
    let write_tokens = jval_u64(pc, "totals.cache_write_tokens");

    let hit_color = if hit_rate >= 70.0 { Color::Green } else if hit_rate >= 30.0 { Color::Yellow } else { Color::Red };

    frame.render_widget(
        Paragraph::new(vec![
            Line::from(vec![label("Read:  "), val(fmt_num(read_tokens))]),
            Line::from(vec![label("Write: "), val(fmt_num(write_tokens))]),
            Line::from(vec![
                label("Hit:   "),
                Span::styled(format!("{:.1}%", hit_rate), Style::default().fg(hit_color).add_modifier(Modifier::BOLD)),
            ]),
        ]),
        inner,
    );
}

fn render_overhead(frame: &mut Frame, app: &App, area: Rect) {
    let block = Block::default().borders(Borders::ALL).title(" Overhead ");
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let Some(stats) = &app.stats else { return };
    let oh = &stats.overhead;

    let avg_color = if oh.average_ms < 300.0 { Color::Green } else if oh.average_ms < 700.0 { Color::Yellow } else { Color::Red };

    frame.render_widget(
        Paragraph::new(vec![
            Line::from(vec![label("Avg: "), Span::styled(format!("{:.0}ms", oh.average_ms), Style::default().fg(avg_color))]),
            Line::from(vec![label("Min: "), val(format!("{:.0}ms", oh.min_ms))]),
            Line::from(vec![label("Max: "), warn(format!("{:.0}ms", oh.max_ms))]),
        ]),
        inner,
    );
}

fn render_proxy_status(frame: &mut Frame, app: &App, area: Rect) {
    let block = Block::default().borders(Borders::ALL).title(" Proxy ");
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let Some(health) = &app.health else { return };

    let pill = |enabled: bool, name: &str| -> Span<'static> {
        if enabled {
            Span::styled(format!("{} ● ", name), Style::default().fg(Color::Green))
        } else {
            Span::styled(format!("{} ○ ", name), Style::default().fg(Color::DarkGray))
        }
    };

    let mut lines = vec![
        Line::from(vec![
            Span::styled(format!("v{}", health.version), Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            dim(format!("  up {}", fmt_uptime(health.uptime_seconds))),
        ]),
    ];

    if let Some(cfg) = &health.config {
        lines.push(Line::from(vec![
            pill(cfg.optimize, "opt"),
            pill(cfg.cache, "cache"),
            pill(cfg.rate_limit, "rl"),
        ]));
        lines.push(Line::from(vec![
            pill(cfg.memory, "mem"),
            pill(cfg.learn, "learn"),
        ]));
    }

    frame.render_widget(Paragraph::new(lines), inner);
}

fn render_router_toin(frame: &mut Frame, app: &App, area: Rect) {
    let cols = Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)]).split(area);
    render_router(frame, app, cols[0]);
    render_toin(frame, app, cols[1]);
}

fn render_router(frame: &mut Frame, app: &App, area: Rect) {
    let block = Block::default().borders(Borders::ALL).title(" Content Router ");
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let Some(stats) = &app.stats else { return };
    let rc = &stats.router.route_counts;

    let cache_hit = rc.get("cache_hit").copied().unwrap_or(0);
    let cache_miss = rc.get("cache_miss").copied().unwrap_or(0);
    let excluded = rc.get("excluded_tool").copied().unwrap_or(0);
    let small = rc.get("small").copied().unwrap_or(0);
    let unchanged = rc.get("ratio_too_high").copied().unwrap_or(0);

    let compressed = cache_hit + cache_miss;
    let total = compressed + excluded + small + unchanged;
    let pct = |n: u64| if total > 0 { format!(" ({:.0}%)", n as f64 / total as f64 * 100.0) } else { String::new() };

    frame.render_widget(
        Paragraph::new(vec![
            Line::from(vec![label("Compressed: "), good(format!("{}{}", compressed, pct(compressed)))]),
            Line::from(vec![label("Excluded:   "), val(format!("{}{}", excluded, pct(excluded)))]),
            Line::from(vec![label("Skipped:    "), val(format!("{}{}", small, pct(small)))]),
            Line::from(vec![label("Unchanged:  "), val(format!("{}{}", unchanged, pct(unchanged)))]),
        ]),
        inner,
    );
}

fn render_toin(frame: &mut Frame, app: &App, area: Rect) {
    let block = Block::default().borders(Borders::ALL).title(" TOIN Learning ");
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let Some(stats) = &app.stats else { return };
    let toin = &stats.toin;

    let patterns = jval_u64(toin, "patterns_tracked");
    let compressions = jval_u64(toin, "total_compressions");
    let retrievals = jval_u64(toin, "total_retrievals");
    let rate = jval_f64(toin, "global_retrieval_rate");
    let rate_str = if rate > 0.0 { format!(" ({:.1}%)", rate * 100.0) } else { String::new() };

    frame.render_widget(
        Paragraph::new(vec![
            Line::from(vec![label("Patterns:     "), val(patterns.to_string())]),
            Line::from(vec![label("Compressions: "), val(compressions.to_string())]),
            Line::from(vec![label("Retrievals:   "), good(format!("{}{}", retrievals, rate_str))]),
        ]),
        inner,
    );
}

fn render_help(frame: &mut Frame, area: Rect) {
    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(" [q] ", Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
            dim("Quit  "),
            Span::styled("[r] ", Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
            dim("Refresh now"),
        ])),
        area,
    );
}

// ── Per-model pricing ─────────────────────────────────────────────────────────

fn model_price_per_mtok(model: &str) -> Option<f64> {
    let m = model.to_lowercase();
    if m.contains("opus-4") || m.contains("opus4") { Some(15.0) }
    else if m.contains("sonnet-4") || m.contains("sonnet4") { Some(3.0) }
    else if m.contains("haiku-4") || m.contains("haiku4") { Some(1.0) }
    else if m.contains("3-5-sonnet") || m.contains("3.5-sonnet") { Some(3.0) }
    else if m.contains("3-5-haiku") || m.contains("3.5-haiku") { Some(0.80) }
    else if m.contains("opus-3") || m.contains("opus3") { Some(15.0) }
    else if m.contains("sonnet-3") || m.contains("sonnet3") { Some(3.0) }
    else if m.contains("haiku-3") || m.contains("haiku3") { Some(0.25) }
    else { None }
}
