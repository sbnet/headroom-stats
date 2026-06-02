# headroom-stats

A real-time terminal dashboard for [Headroom](https://github.com/chopratejas/headroom) — the token-optimization proxy for Claude. See your token savings, cache performance, and compression stats live, without leaving the terminal.

![Rust](https://img.shields.io/badge/rust-1.75%2B-orange)
![License](https://img.shields.io/badge/license-MIT-blue)

## What it shows

```
 Headroom Stats  http://localhost:8787  ↻5s  2s ago
┌ Résumé ──────────────────────────────────────────────────────────────┐
│ Requêtes: 77   Mises en cache: 64   Échecs: 0   Limitées: 0         │
│ Tokens: 810,270 → 644,688  (20.4% réduction)  Économisés: 165,582   │
└──────────────────────────────────────────────────────────────────────┘
┌ Par modèle ───────────────────────────────────────────────────────────┐
│ Modèle                     Reqs  Tokens envoyés  Économisés  Réduct. │
│ claude-sonnet-4-6            12      888,091       165,582    20.4%  │
│ claude-haiku-4-5-20251001     1       12,450             0     0.0%  │
└──────────────────────────────────────────────────────────────────────┘
┌ Cache prefix ──────────────┐┌ Overhead ──────────────────────────────┐
│ Lecture:  1,132,138        ││ Moy:  239ms                            │
│ Écriture:   225,206        ││ Min:    7ms                            │
│ Hit rate:     83.4%        ││ Max: 3,711ms                           │
└────────────────────────────┘└────────────────────────────────────────┘
┌ Content Router ────────────┐┌ TOIN Learning ─────────────────────────┐
│ Compressé:   25  (7%)      ││ Patterns:      9                       │
│ Exclu:       79 (21%)      ││ Compressions:  9                       │
│ Ignoré:     259 (68%)      ││ Récupérations: 3 (33.3%)               │
│ Inchangé:    18  (5%)      ││                                        │
└────────────────────────────┘└────────────────────────────────────────┘
 [q] Quit   [r] Refresh now
```

## Requirements

- [Headroom](https://github.com/chopratejas/headroom) proxy running locally
- Rust 1.75+

## Installation

```bash
cargo install --git https://github.com/sbnet/headroom-stats
```

Or build from source:

```bash
git clone https://github.com/sbnet/headroom-stats
cd headroom-stats
cargo build --release
./target/release/headroom-stats
```

## Usage

```bash
# Reads ANTHROPIC_BASE_URL automatically (set by headroom)
headroom-stats

# Explicit proxy URL
headroom-stats --url http://localhost:8787

# Custom refresh interval (default: 5s)
headroom-stats --interval 10
```

| Key | Action |
|-----|--------|
| `q` / `Esc` | Quit |
| `r` | Force immediate refresh |

## How it works

`headroom-stats` polls the `/stats` HTTP endpoint exposed by the Headroom proxy and renders the response as a live TUI dashboard using [ratatui](https://ratatui.rs). The dashboard refreshes automatically every N seconds (configurable) and also responds to force-refresh on demand.

The displayed metrics mirror what `headroom perf` reports from logs, but sourced from the live proxy instead:

| Section | Source |
|---------|--------|
| Request summary | `requests.*` + `tokens.*` |
| Per-model breakdown | `cost.per_model.*` |
| Prefix cache | `prefix_cache.totals.*` |
| Optimization overhead | `overhead.*` |
| Content router | `router.route_counts` |
| TOIN learning | `toin.*` |

## Stack

- **[ratatui](https://ratatui.rs)** — terminal UI framework
- **[tokio](https://tokio.rs)** — async runtime
- **[reqwest](https://docs.rs/reqwest)** — HTTP client
- **[clap](https://docs.rs/clap)** — CLI argument parsing

## License

MIT