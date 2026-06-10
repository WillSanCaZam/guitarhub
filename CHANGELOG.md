# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## [Unreleased]

### Added
- macOS ARM64 (aarch64-apple-darwin) release builds via CI

## [0.3.0] - 2026-06-10

### Added

- **ETag cache for catalog sync** — `CatalogSyncService` now uses `If-None-Match`/`304 Not Modified` to skip download when the remote catalog is unchanged. ETag and source_id are persisted in the settings table (key-value). (#3)
- **Down migrations** — `MigrationRunner` supports `rollback(steps)` via `.down.sql` files. 9 down migration files created for the full migration chain (001→009). (#4)
- **FilterBar E2E spec** — 6 new E2E tests covering toggle, category, price range, condition, currency, sort, and clear-all. (#5)

### Changed

- **Scraper category mapping** — `ReverbAdapter` now extracts `category` from the `product_type` parameter via `PRODUCT_TYPE_CATEGORIES` mapping. Products will no longer have empty categories. (#1)
- **Coverage enforcement** — `openspec/config.yaml` `coverage_threshold` increased from 0 to 70, and `verify.test_command` includes `npm run test:coverage`. (#2)
- **Docs hygiene** — `.env.example` comment updated from `"user/guitarhub"` to `"WillSanCaZam/guitarhub"`.

### Fixed

- `httpmock` test ordering in `sync_etag_200_saves_etag_then_304_uses_it` — mocks deleted before re-registration to avoid first-match conflict in httpmock 0.8.

## [0.2.0] - 2026-06-10

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

[Unreleased]: https://github.com/WillSanCaZam/guitarhub/compare/v0.3.0...HEAD
[0.3.0]: https://github.com/WillSanCaZam/guitarhub/releases/tag/v0.3.0
[0.2.0]: https://github.com/WillSanCaZam/guitarhub/releases/tag/v0.2.0
[0.1.0]: https://github.com/WillSanCaZam/guitarhub/releases/tag/v0.1.0
