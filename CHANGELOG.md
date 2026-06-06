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

### Changed

### Fixed

### Security

[0.1.0]: https://github.com/willbennett/guitarhub/releases/tag/v0.1.0
