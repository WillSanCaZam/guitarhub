# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## [Unreleased]

## [0.6.0] - 2026-06-15

### Added

- **"Amp Glow" Design System** — Complete redesign with void palette (#07070C → #252538), glow accents (#FF7A3D, #FFB84D, #FF4D4D), Space Grotesk display font, and 9 animations (amp-glow pulse, string-vibe, power-on, shimmer warm-toned).
- **ProductCard "The Stage" v2** — Full rewrite with store logo badge, category pill, deal badge (-30%), star rating, social proof ("X people viewing now"), price with strikethrough original, hover glow effects (scale 1.03, glow border), and quick actions (wishlist, alert).
- **Discovery Feed** — Replaced static bento grid with immersive discovery experience: HeroSection (70vh gradient mesh with animated search bar), TrendingPill (horizontal scrollable trending searches), FeaturedRig (full-width artist rig banner), FeedSection wrappers with "See All" links.
- **Product Detail "The Deep Dive"** — New route at `/product/[sku]` with image gallery (thumbnails), reviews section with upvote/downvote, price history sparkline, store comparison table, specs grid, and action buttons (wishlist, alert, compare).
- **FilterBar v2** — Category pills with icons, dual-thumb price range slider, condition filter checkboxes, styled sort dropdown, active filter pills with × to remove.
- **New Components** — StarRating (SVG stars with partial fill), PriceDisplay (Intl currency + discount badge), PriceRangeSlider (dual-thumb), StoreComparison, PriceHistory.

### Changed

- All design tokens replaced with "Amp Glow" palette (obsidian/graphite/amber from v0.5.0 superseded).
- Space Grotesk replaces Syne as display font.
- ProductCard skeleton shimmer now uses warm glow gradient (--glow-soft) instead of grey.
- FilterBar sort uses custom styled dropdown instead of native <select>.
- Version bumped to 0.6.0 across Cargo.toml, tauri.conf.json, and package.json.

### Fixed

- Reviews section added to ProductDetail (was missing, CRITICAL spec violation).
- Image thumbnails added to ProductDetail (5 placeholder thumbnails).
- PriceRangeSlider track color corrected to --void-raised.
- Product rating now uses dynamic product.rating instead of hardcoded 4.5.
- 25 stale tests updated for new components (192/192 passing).

## [0.5.0] - 2026-06-15

### Added

- **Design System** — New design tokens (obsidian/graphite/amber palette), Google Fonts (Syne, Inter, JetBrains Mono), global CSS with skeleton shimmer and card animations.
- **Navbar Redesign** — SVG icons, /collection link, wishlist badge with count, sync button with amber hover.
- **ProductCard Rewrite** — Store link button (opens external URL), wishlist toggle, shimmer skeleton during image load, condition badge, JetBrains Mono price font.
- **FilterBar Always Visible** — Removed collapse toggle, category chips, active filter pills with × to remove.
- **SearchPanel Improvements** — Recent searches as chips below search bar, skeleton cards during loading (not spinner), stagger animation for card entrance.
- **Wishlist Page Rewrite** — ProductCard grid layout, header with count and estimated value, empty state SVG with CTA.
- **Dashboard Bento Redesign** — 5-cell layout with clear hierarchy (Search, Sync Status, Stats KPIs, Featured Deal, Collection).
- **Collection Page Improvements** — Stats header with count and value, grid/list toggle, prominent CSV export button.
- **Tauri Opener Plugin** — Replaced deprecated tauri-plugin-shell with tauri-plugin-opener for opening URLs in browser.

### Changed

- All hardcoded colors replaced with CSS custom properties from design tokens.
- DashboardCell uses design tokens with amber glow hover effect.
- Svelte 5 runes migration completed — all stores use `$state` (zero `writable()` imports).
- Version bumped to 0.5.0 across Cargo.toml, tauri.conf.json, and package.json.

### Fixed

- `$wishlistState` Svelte 4 store syntax removed from +layout.svelte (now uses direct rune access).
- FilterBar toggle removed — filters are now always visible as per design spec.

## [0.4.0] - 2026-06-13

### Added

- **Community Hub** — Auth (OAuth/JWT), user profiles, practice streaks, lessons, riffs, feed, comments, follows, challenges, and leaderboards.
- **Navigation Shell** — Adaptive sidebar/bottom nav with AppShell, Sidebar, and BottomNav components.
- **Design System** — Acoustic Dark Modern design tokens, shared UI atoms (Button, Card, Avatar, Badge, Chip, Input, ProgressBar).
- **Community Routes** — Explore, Feed, Lessons, My Gear, Profile, and Saved Riffs pages.
- **Community Backend** — Auth, community, and profile Tauri commands/services; SQLite migration 010 for community schema.
- **Svelte 5 Store Migration** — All 8 stores now use `$state` runes (zero `writable()` imports).

### Changed

- README version badge updated to v0.4.0.
- Removed unused `beautifulsoup4` dependency from scraper.
- Migrated `dashboard.ts`, `sync.ts`, `wishlist.ts` to Svelte 5 runes.

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

[Unreleased]: https://github.com/WillSanCaZam/guitarhub/compare/v0.6.0...HEAD
[0.6.0]: https://github.com/WillSanCaZam/guitarhub/compare/v0.5.0...v0.6.0
[0.5.0]: https://github.com/WillSanCaZam/guitarhub/compare/v0.4.0...v0.5.0
[0.4.0]: https://github.com/WillSanCaZam/guitarhub/releases/tag/v0.4.0
[0.3.0]: https://github.com/WillSanCaZam/guitarhub/releases/tag/v0.3.0
[0.2.0]: https://github.com/WillSanCaZam/guitarhub/releases/tag/v0.2.0
[0.1.0]: https://github.com/WillSanCaZam/guitarhub/releases/tag/v0.1.0
