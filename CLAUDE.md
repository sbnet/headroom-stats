# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

@AGENTS.md

## Available skills

The skills below are located in `.agents/skills/`. Read the corresponding `SKILL.md` before working in the related domain.

## Commands

```bash
cargo build                    # debug build
cargo build --release          # release build
cargo run -- --url http://localhost:8787 --interval 5
cargo clippy -- -D warnings    # lint
cargo fmt                      # format
```

No test suite exists yet; correctness is verified by running against a live Headroom proxy.

## Architecture

A single-binary async TUI that polls a [Headroom](https://github.com/chopratejas/headroom) proxy and renders live stats.

**Data flow:**

```
HeadroomClient ──tokio::join!──> /stats + /stats-history + /health
        │
        └─ mpsc::UnboundedSender<AppEvent>
                │
                ▼
        main loop: drain events → App::handle_event → ui::render
```

The background poller (`app::spawn_poller`) runs on a `tokio::select!` between a sleep interval and a `force_rx` channel. Sending on `force_rx` (triggered by `r` key) causes an immediate re-poll.

**Module responsibilities:**

| Module | Role |
|---|---|
| `main.rs` | CLI args (`clap`), terminal init/restore, event loop |
| `app.rs` | `App` state struct, `spawn_poller`, `AppEvent`/`AppData` types |
| `client.rs` | `HeadroomClient` — wraps `reqwest`, fetches the three endpoints |
| `model.rs` | `serde::Deserialize` structs for all API responses; `jval_f64`/`jval_u64` helpers for navigating untyped `serde_json::Value` fields |
| `ui.rs` | All `ratatui` rendering — one `render_*` fn per dashboard section |

**Key design decisions:**
- `/stats` and `/stats-history` carry overlapping data; `display_session` in history is the persisted session view (matches `headroom perf`) while `/stats` has live per-request counters.
- `cost.per_model.*` and `prefix_cache.*` are kept as `serde_json::Value` because their schemas vary by proxy version; `jval_f64`/`jval_u64` extract values by dot-path.
- Model pricing in `ui::model_price_per_mtok` is hardcoded by model name substring — update when new Claude models ship.
