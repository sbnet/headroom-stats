use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table},
};

use crate::app::App;
use crate::model::{jval_f64, jval_u64};

pub fn render(frame: &mut Frame, app: &App) {
    let area = frame.area();

    let rows = Layout::vertical([
        Constraint::Length(1), // title
        Constraint::Length(4), // summary (2 content lines + 2 borders)
        Constraint::Min(4),    // per-model table
        Constraint::Length(5), // cache | overhead
        Constraint::Length(5), // content router | TOIN
        Constraint::Length(1), // help
    ])
    .split(area);

    render_title(frame, app, rows[0]);
    render_summary(frame, app, rows[1]);
    render_models(frame, app, rows[2]);
    render_cache_overhead(frame, app, rows[3]);
    render_router_toin(frame, app, rows[4]);
    render_help(frame, rows[5]);
}

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

fn render_title(frame: &mut Frame, app: &App, area: Rect) {
    let interval_secs = app.interval.as_secs();
    let err_marker = if app.last_error.is_some() { "  [ERR]" } else { "" };
    let title = format!(" Headroom Stats  {}{}", app.base_url, err_marker);
    let session_note = "  [session]";
    let refresh_info = format!("  ↻{}s  {}", interval_secs, app.since_refresh());

    let line = Line::from(vec![
        Span::styled(title, Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::styled(session_note, Style::default().fg(Color::DarkGray).add_modifier(Modifier::ITALIC)),
        Span::styled(refresh_info, Style::default().fg(Color::DarkGray)),
    ]);
    frame.render_widget(Paragraph::new(line), area);
}

fn render_summary(frame: &mut Frame, app: &App, area: Rect) {
    let block = Block::default().borders(Borders::ALL).title(" Résumé ");

    let inner = block.inner(area);
    frame.render_widget(block, area);

    // No data yet — could be first load or a persistent error.
    let Some(stats) = &app.stats else {
        let msg = if let Some(err) = &app.last_error {
            Line::from(vec![
                Span::styled("Connection error: ", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
                Span::raw(err.clone()),
            ])
        } else {
            Line::from(Span::styled("Connecting…", Style::default().fg(Color::DarkGray)))
        };
        frame.render_widget(Paragraph::new(msg), inner);
        return;
    };

    let t = &stats.tokens;
    let r = &stats.requests;

    let tokens_out = t.input.saturating_sub(t.saved);
    let savings_pct = t.savings_percent;
    let savings_color = if savings_pct >= 20.0 {
        Color::Green
    } else if savings_pct >= 5.0 {
        Color::Yellow
    } else {
        Color::White
    };

    let line1 = Line::from(vec![
        label("Requêtes: "),
        val(format!("{}", r.total)),
        Span::raw("  "),
        label("Mises en cache: "),
        val(format!("{}", r.cached)),
        Span::raw("  "),
        label("Échecs: "),
        val(format!("{}", r.failed)),
        Span::raw("  "),
        label("Limitées: "),
        val(format!("{}", r.rate_limited)),
    ]);

    let line2 = Line::from(vec![
        label("Tokens: "),
        val(fmt_num(t.input)),
        Span::raw(" → "),
        val(fmt_num(tokens_out)),
        Span::raw("  ("),
        Span::styled(
            format!("{:.1}% réduction", savings_pct),
            Style::default().fg(savings_color).add_modifier(Modifier::BOLD),
        ),
        Span::raw(")  "),
        label("Économisés: "),
        good(fmt_num(t.saved)),
    ]);

    let para = Paragraph::new(vec![line1, line2]);
    frame.render_widget(para, inner);
}

/// Returns the list input price in USD per million tokens for known Claude models.
fn model_price_per_mtok(model: &str) -> Option<f64> {
    let m = model.to_lowercase();
    if m.contains("opus-4") || m.contains("opus4") {
        Some(15.0)
    } else if m.contains("sonnet-4") || m.contains("sonnet4") {
        Some(3.0)
    } else if m.contains("haiku-4") || m.contains("haiku4") {
        Some(1.0)
    } else if m.contains("3-5-sonnet") || m.contains("3.5-sonnet") {
        Some(3.0)
    } else if m.contains("3-5-haiku") || m.contains("3.5-haiku") {
        Some(0.80)
    } else if m.contains("opus-3") || m.contains("opus3") {
        Some(15.0)
    } else if m.contains("sonnet-3") || m.contains("sonnet3") {
        Some(3.0)
    } else if m.contains("haiku-3") || m.contains("haiku3") {
        Some(0.25)
    } else {
        None
    }
}

fn render_models(frame: &mut Frame, app: &App, area: Rect) {
    let block = Block::default().borders(Borders::ALL).title(" Par modèle ");
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let Some(stats) = &app.stats else { return };

    let header_style = Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD);
    let header = Row::new(vec![
        Cell::from("Model").style(header_style),
        Cell::from("Reqs").style(header_style),
        Cell::from("Tokens sent").style(header_style),
        Cell::from("Saved").style(header_style),
        Cell::from("Reduction").style(header_style),
        Cell::from("$/MTok").style(header_style),
        Cell::from("~Savings").style(header_style),
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
                (
                    jval_u64(cm, "tokens_sent"),
                    jval_u64(cm, "tokens_saved"),
                    jval_f64(cm, "reduction_pct"),
                )
            } else {
                (0, 0, 0.0)
            };

            let pct_color = if pct >= 15.0 {
                Color::Green
            } else if pct >= 5.0 {
                Color::Yellow
            } else {
                Color::White
            };

            let (price_str, savings_str) = match model_price_per_mtok(model) {
                Some(price) => {
                    let savings_usd = tokens_saved as f64 * price / 1_000_000.0;
                    (
                        format!("${:.2}", price),
                        format!("~${:.2}", savings_usd),
                    )
                }
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

fn render_cache_overhead(frame: &mut Frame, app: &App, area: Rect) {
    let cols = Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)]).split(area);

    render_cache(frame, app, cols[0]);
    render_overhead(frame, app, cols[1]);
}

fn render_cache(frame: &mut Frame, app: &App, area: Rect) {
    let block = Block::default().borders(Borders::ALL).title(" Cache prefix ");
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let Some(stats) = &app.stats else { return };
    let pc = &stats.prefix_cache;

    let hit_rate = jval_f64(pc, "totals.hit_rate");
    let read_tokens = jval_u64(pc, "totals.cache_read_tokens");
    let write_tokens = jval_u64(pc, "totals.cache_write_tokens");

    let hit_color = if hit_rate >= 70.0 {
        Color::Green
    } else if hit_rate >= 30.0 {
        Color::Yellow
    } else {
        Color::Red
    };

    let lines = vec![
        Line::from(vec![label("Lecture:  "), val(fmt_num(read_tokens))]),
        Line::from(vec![label("Écriture: "), val(fmt_num(write_tokens))]),
        Line::from(vec![
            label("Hit rate: "),
            Span::styled(
                format!("{:.1}%", hit_rate),
                Style::default().fg(hit_color).add_modifier(Modifier::BOLD),
            ),
        ]),
    ];
    frame.render_widget(Paragraph::new(lines), inner);
}

fn render_overhead(frame: &mut Frame, app: &App, area: Rect) {
    let block = Block::default().borders(Borders::ALL).title(" Overhead ");
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let Some(stats) = &app.stats else { return };
    let oh = &stats.overhead;

    let avg_color = if oh.average_ms < 300.0 {
        Color::Green
    } else if oh.average_ms < 700.0 {
        Color::Yellow
    } else {
        Color::Red
    };

    let lines = vec![
        Line::from(vec![
            label("Moy: "),
            Span::styled(
                format!("{:.0}ms", oh.average_ms),
                Style::default().fg(avg_color),
            ),
        ]),
        Line::from(vec![label("Min: "), val(format!("{:.0}ms", oh.min_ms))]),
        Line::from(vec![label("Max: "), warn(format!("{:.0}ms", oh.max_ms))]),
    ];
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

    // Sum only the meaningful routing categories (not meta-counts like content_blocks)
    let rc = &stats.router.route_counts;
    let cache_hit = rc.get("cache_hit").copied().unwrap_or(0);
    let cache_miss = rc.get("cache_miss").copied().unwrap_or(0);
    let excluded = rc.get("excluded_tool").copied().unwrap_or(0);
    let small = rc.get("small").copied().unwrap_or(0);
    let unchanged = rc.get("ratio_too_high").copied().unwrap_or(0);

    let total = cache_hit + cache_miss + excluded + small + unchanged;
    let pct = |n: u64| {
        if total > 0 {
            format!(" ({:.0}%)", n as f64 / total as f64 * 100.0)
        } else {
            String::new()
        }
    };

    let compressed = cache_hit + cache_miss;
    let lines = vec![
        Line::from(vec![label("Compressé:"), good(format!("  {}{}", compressed, pct(compressed)))]),
        Line::from(vec![label("Exclu:    "), val(format!("  {}{}", excluded, pct(excluded)))]),
        Line::from(vec![label("Ignoré:   "), val(format!("  {}{}", small, pct(small)))]),
        Line::from(vec![label("Inchangé: "), val(format!("  {}{}", unchanged, pct(unchanged)))]),
    ];
    frame.render_widget(Paragraph::new(lines), inner);
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
    let retrieval_rate = jval_f64(toin, "global_retrieval_rate");

    let rate_str = if retrieval_rate > 0.0 {
        format!(" ({:.1}%)", retrieval_rate * 100.0)
    } else {
        String::new()
    };

    let lines = vec![
        Line::from(vec![label("Patterns:     "), val(patterns.to_string())]),
        Line::from(vec![label("Compressions: "), val(compressions.to_string())]),
        Line::from(vec![
            label("Récupérations:"),
            Span::raw(" "),
            good(format!("{}{}", retrievals, rate_str)),
        ]),
    ];
    frame.render_widget(Paragraph::new(lines), inner);
}

fn render_help(frame: &mut Frame, area: Rect) {
    let line = Line::from(vec![
        Span::styled(" [q] ", Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
        Span::styled("Quitter  ", Style::default().fg(Color::DarkGray)),
        Span::styled("[r] ", Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
        Span::styled("Rafraîchir maintenant", Style::default().fg(Color::DarkGray)),
    ]);
    frame.render_widget(Paragraph::new(line), area);
}

/// Format a large number with thousands separator.
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
