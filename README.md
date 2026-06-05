# headroom-stats

A real-time terminal dashboard for [Headroom](https://github.com/chopratejas/headroom) — the token-optimization proxy for Claude. See your token savings, cache performance, and compression stats live, without leaving the terminal.

![Rust](https://img.shields.io/badge/rust-1.75%2B-orange)
![License](https://img.shields.io/badge/license-MIT-blue)

## Requirements

- [Headroom](https://github.com/chopratejas/headroom) proxy running locally

## Installation

```bash
curl -fsSL https://raw.githubusercontent.com/sbnet/headroom-stats/main/install.sh | bash
```

This downloads a pre-built binary for your platform (Linux x86/arm64, macOS Intel/Apple Silicon) to `~/.local/bin`. Re-run to update.

**Custom install dir:**
```bash
INSTALL_DIR=/usr/local/bin curl -fsSL https://raw.githubusercontent.com/sbnet/headroom-stats/main/install.sh | bash
```

**Build from source** (requires Rust 1.75+):
```bash
cargo install --git https://github.com/sbnet/headroom-stats
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

# Release workflow

```
# 1. Bump the version in Cargo.toml
# 2. Commit + tag
git tag vX.Y.Z
git push origin vX.Y.Z
```
## Contribution Guidelines

- Prefer small, focused pull requests.
- Keep changes pragmatic: small diffs, clear manual checks, explicit commit messages.
- Do not commit secrets or credentials in plain text.
- If you change deployment behavior, update this README.

## Security

If you identify a vulnerability or unsafe infra/deployment practice, open a private issue or contact the maintainer before public disclosure.

## License

This project is licensed under MIT. See [LICENSE](LICENSE).