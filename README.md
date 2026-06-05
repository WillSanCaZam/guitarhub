# GuitarHub

Electric guitar price tracker — offline-first, multi-store, free and open source.

<!-- ![Screenshot](screenshot.png) -->

## Features

- **Offline-first** — full local catalog with search and sync
- **Multi-store** — track prices across Reverb and other marketplaces
- **Full-text search** — FTS5-powered search with filters, sort, and pagination
- **Free & open source** — GPL-3.0 licensed, no subscriptions, no ads
- **Desktop app** — Tauri 2 native application (Linux, macOS, Windows)
- **Automated scraping** — scheduled CI pipeline keeps catalog up to date

## Prerequisites

- Rust (latest stable via rustup)
- Node.js 20+
- Python 3.12+

## Quick Start

```bash
make setup   # Install frontend, Rust, and Python dependencies
make dev     # Launch Tauri dev server with hot reload
```

## Build

```bash
make build          # Production desktop build
make test           # Run Rust and Python tests
make test-rust      # Rust tests only
make test-python    # Python tests only
```

## Tech Stack

| Layer | Technology |
|-------|-----------|
| Desktop shell | Tauri 2 |
| Frontend | Svelte 5, TypeScript, Vite |
| Backend | Rust, SQLx |
| Database | SQLite with FTS5 |
| Scraper | Python, BeautifulSoup, Pydantic |
| CI | GitHub Actions |

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## License

[GPL-3.0](LICENSE)
