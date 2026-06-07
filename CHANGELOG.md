# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## [0.1.0] - 2026-06-05

### Added

- **Search** — Full-text search across the local catalog via SQLite FTS5.
- **Sync Service** — Background catalog sync with state-machine tracking (idle → downloading → validating → inserting → done).
- **Price Drops & Alerts** — Detect price drops with configurable relative/absolute thresholds; dispatch alerts via in-app, Ntfy, or webhook channels.
- **Collection Tracking** — Manage personal gear collection with purchase details, condition, and estimated market value.
- **Dashboard Bento Grid** — 9-cell dashboard showing catalog stats, wishlist count, recent searches, collection value, and price insights.
- **Local Image Cache** — LRU image cache with SQLite BLOB storage, request coalescing, stale fallback, and 7-day TTL.
- **Export** — Export wishlist, price history, settings, and collection as a ZIP of JSON files.
- **Scraper** — Reverb.com marketplace adapter with retry, structured errors (FetchError, ParseError), and JSON API mapping.
- **Structured Errors** — Unified AppError enum with typed variants (NotFound, InvalidInput, Database, Network, Internal, SyncInProgress).
- **Offline-First** — Full catalog and image caching for offline browsing, search, and collection management.

### Changed

- Initial project scaffold with Tauri 2, Svelte 5, and Rust backend.

### Fixed

- CI workflows for scraper-less builds.

### Security

- URL validation for images and webhooks with SSRF prevention (IP literal rejection, HTTPS enforcement, domain allowlist).

## [Unreleased]

### Added

- In-app updater via tauri-plugin-updater (checks latest.json on gh-pages)
- Tag-scoped release concurrency (groups by tag name)
- Automatic Linux .deb generation via CI release pipeline
- docs/ARCHITECTURE.md — architecture overview and design decisions
- docs/RELEASE.md — release process documentation

### Changed

- Release CI matrix reduced to Linux-only (macOS/Windows deferred)
- httpmock upgraded from 0.7 to 0.8.3

### Fixed

- macOS codesigning bypassed in CI with TAURI_SKIP_SIGNING
- SvelteKit test file routing conflict (+page.test.ts → page.test.ts)
- release.yml: tauri-action replaced with npx tauri build
- release.yml: asset discovery excludes inner tar.gz files

### Security

[0.1.0]: https://github.com/WillSanCaZam/guitarhub/releases/tag/v0.1.0
