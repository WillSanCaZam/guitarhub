# GuitarHub

[![v0.4.0](https://img.shields.io/badge/release-v0.4.0-blue)](https://github.com/WillSanCaZam/guitarhub/releases)

A native cross-platform desktop app that aggregates guitar, amp, pedal, and accessory listings from multiple online stores into a single unified catalog.

## What is GuitarHub?

GuitarHub is the **Mihon of guitars**: free forever, open source, offline-first, zero server costs, and community-extensible. Install the app, sync the catalog, and browse or search without ads, registration, or trackers. When you find a product, you are redirected to the original store to buy. GuitarHub never holds inventory, never processes payments, and never touches money.

## Tech Stack

| Layer | Technology |
|-------|------------|
| App framework | Tauri 2.x |
| UI | Svelte 5 |
| App backend | Rust |
| Local DB | SQLite + FTS5 |
| Frontend tooling | Vite + TypeScript |

## Quick Start

```bash
# Install frontend dependencies
npm install

# Run the frontend dev server
npm run dev

# Run the Tauri app in dev mode
cargo tauri dev
```

## Architecture Overview

The app follows a clean architecture:

- **Frontend (Svelte 5)** — UI layer with stores, routes, and components.
- **Backend (Rust / Tauri)** — Commands act as IPC glue, services hold business logic, and repositories abstract data access.
- **SQLite** — Local database with FTS5 for full-text search, WAL mode enabled, and a custom migration runner.

## Key Features

- **Search** — Full-text search across the local catalog (FTS5).
- **Price History** — Track price changes over time per SKU.
- **Alerts** — App, Ntfy, and Webhook alert channels for price drops.
- **Collection** — Manage your personal gear collection with estimated values.
- **Export** — Export wishlist and price history to a ZIP of CSV files.
- **Community Hub** — Auth, user profiles, practice streaks, lessons, riffs, feed, comments, follows, and challenges.

## Contributing

See [docs/CONTRIBUTING.md](docs/CONTRIBUTING.md) for contribution guidelines.

## License

This project is licensed under the **GNU General Public License v3.0** (GPL-3.0). See [LICENSE](LICENSE) for details.

## Security

For security policies and vulnerability reporting, see [SECURITY.md](SECURITY.md).
