# Tasks: Price Intelligence — Phase 3

## Review Workload Forecast

| Field | Value |
|-------|-------|
| Estimated changed lines | ~1,200–1,400 |
| 400-line budget risk | High |
| Chained PRs recommended | Yes |
| Suggested split | PR 1: Groups 0+1 (~350) → PR 2: Groups 2+3 (~450) → PR 3: Groups 4+5+6 (~500) |
| Delivery strategy | ask-on-risk |
| Chain strategy | pending |

Decision needed before apply: Yes
Chained PRs recommended: Yes
Chain strategy: pending
400-line budget risk: High

### Suggested Work Units

| Unit | Goal | Likely PR | Notes |
|------|------|-----------|-------|
| 1 | Infrastructure + App Settings (Groups 0+1) | PR 1 | Base to main. Migrations, deps, settings IPC, Settings.svelte shell. |
| 2 | Price History + Price Insight (Groups 2+3) | PR 2 | Dependent on PR 1. PriceHistoryRepo, price IPC, SVG chart, badge. |
| 3 | Alert Delivery + Data Export + Integration (Groups 4+5+6) | PR 3 | Dependent on PR 2. AlertService, ExportService, final wiring. |

---

## Group 0: Infrastructure (shared)

- [x] **T-0.1** — Add `zip = "2"` and `tauri-plugin-dialog = "2"` to `Cargo.toml`
  - Deps: none
  - TDD: Write compile-check test asserting deps resolve
  - Files: `Cargo.toml`
  - Time: 15min
  - AC: `cargo check` passes with new deps

- [x] **T-0.2** — Create migration `004_add_price_source.sql`
  - `ALTER TABLE price_history ADD COLUMN source_id TEXT NOT NULL DEFAULT '';` + `CREATE INDEX IF NOT EXISTS idx_price_history_sku_recorded ON price_history(sku, recorded_at);`
  - Deps: none
  - TDD: Seed old-format data, apply migration, verify source_id is `''` and index exists via `EXPLAIN QUERY PLAN`
  - Files: `src-tauri/src/repository/sqlite/migrations/004_add_price_source.sql`
  - Time: 30min
  - AC: Migration applies idempotently, existing rows get `source_id = ''`

- [x] **T-0.3** — Create migration `005_add_settings.sql`
  - `CREATE TABLE IF NOT EXISTS settings (key TEXT PRIMARY KEY, value TEXT NOT NULL);`
  - Deps: none
  - TDD: Apply migration, INSERT + SELECT round-trip, verify idempotent re-apply
  - Files: `src-tauri/src/repository/sqlite/migrations/005_add_settings.sql`
  - Time: 20min
  - AC: Table created, UPSERT works, re-apply is no-op

## Group 1: app-settings (shared by F3, F4)

- [x] **T-1.1** — Create `settings_command.rs` with `get_setting` + `save_setting` IPC
  - Inline sqlx queries per design ADR (no repository layer — pure UPSERT/SELECT on `settings` table)
  - Wire into `commands/mod.rs`
  - Deps: T-0.3 (settings migration)
  - TDD RED: Test `save_setting("key","val")` then `get_setting("key")` returns `"val"`, unknown key returns `""`, empty key returns error
  - Files: `commands/settings_command.rs`, `commands/mod.rs`
  - Time: 1h
  - AC: Both IPC commands work, round-trip passes, no panic on missing key

- [x] **T-1.2** — Create `Settings.svelte` with alert channel form shell
  - Radio group (App / Ntfy.sh / Webhook), config text input, "Test Notification" button stub
  - Calls `get_setting`/`save_setting` on mount/change
  - Deps: T-1.1 (settings IPC)
  - TDD: Manual — load page, verify radio renders, mount triggers get_setting call
  - Files: `src/lib/components/Settings.svelte`
  - Time: 1.5h
  - AC: Form renders, settings persist across reload, radio change triggers save_setting

- [x] **T-1.3** — Write settings unit tests
  - Deps: T-1.1
  - TDD RED: Already done in T-1.1. Now full coverage: empty key, round-trip, unknown key, concurrent save+read, structured JSON value survival
  - Files: `commands/settings_command.rs` (add `#[cfg(test)] mod tests`)
  - Time: 1h
  - AC: `cargo test` passes, all three spec scenarios covered, clippy clean

## Group 2: price-history (F1)

- [x] **T-2.1** — Create `PriceHistoryRepo` in `repository/price_history.rs`
  - Concrete struct `PriceHistoryRepo { pool: SqlitePool }` (no trait — matches existing ImageCacheRepo pattern)
  - `get_history(sku, window_days)` with 5σ outlier filter per source via CTE
  - `get_insight(sku)` returning rolling stats (min_30d, avg_90d, count_30d, current_price)
  - Wire into `repository/mod.rs`
  - Deps: T-0.2 (migration 004 adds source_id column)
  - TDD RED: Write test seeding 2 sources with outliers → assert outlier excluded per source; write test with <30 points → all returned unfiltered; empty table → empty vec; `get_insight` returns None on empty data
  - Files: `repository/price_history.rs`, `repository/mod.rs`
  - Time: 2h
  - AC: All 8 test cases from design §8.1 pass, queries complete <50ms with 10k rows

- [x] **T-2.2** — Create `price_command.rs` with `get_price_history` IPC
  - Validates sku non-empty, delegates to `PriceHistoryRepo::get_history`
  - Wire into `commands/mod.rs`
  - Deps: T-2.1 (PriceHistoryRepo exists)
  - TDD RED: Test empty SKU returns `"sku_required"`, valid SKU returns `Vec<PricePoint>`
  - Files: `commands/price_command.rs`, `commands/mod.rs`
  - Time: 1h
  - AC: IPC contract matches spec §4.1, error strings correct

- [x] **T-2.3** — Create `PriceChart.svelte` SVG chart component
  - `<svg viewBox>` with `<polyline>` per source, 5-color palette, downsampling at >500 pts
  - `computeSvgData` derived: groups by source_id, computes extents, flags insufficient data (<30 pts per source)
  - Deps: T-2.2 (IPC command exists)
  - TDD: Manual — mount with mock data, verify DOM contains correct number of `<polyline>` elements, verify `role="img"` + `<title>`
  - Files: `src/lib/components/PriceChart.svelte`
  - Time: 2h
  - AC: Chart renders 1–5 source lines, responsive via viewBox, "Insufficient data" shown per source when <30 pts

- [x] **T-2.4** — Create `ProductDetail.svelte` and wire PriceChart
  - New page component showing product name, price, and `<PriceChart {sku}>`
  - Accepts `sku` prop, renders chart on mount
  - Deps: T-2.3 (PriceChart exists)
  - TDD: Manual — verify chart renders on detail view with known SKU
  - Files: `src/lib/components/ProductDetail.svelte`
  - Time: 1h
  - AC: Detail page shows chart with correct SKU, empty state shown when no data

- [x] **T-2.5** — Write price history backend tests
  - Deps: T-2.1, T-2.2
  - TDD: Already covered. Now add benchmark assertion (<50ms), multi-source edge case, single-point scenario (spec scenario 4)
  - Files: `repository/price_history.rs` + `commands/price_command.rs` (add test modules)
  - Time: 1h
  - AC: All 8+ repo tests + 4 command tests pass, benchmark assertion included

## Group 3: price-insight (F2)

- [x] **T-3.1** — Add `get_price_insight` to `price_command.rs`
  - Calls `PriceHistoryRepo::get_insight`, classifies level (green/amber/hidden) per spec thresholds
  - Returns `PriceInsight { level, pct, current_price, min_30d, avg_90d }`
  - Deps: T-2.2 (price_command.rs exists)
  - TDD RED: Test all 3 classification paths: green (price ≤ min_30d×1.05), amber (price ≥ avg_90d×1.20), hidden (<30 rows)
  - Files: `commands/price_command.rs`
  - Time: 1h
  - AC: All 3 badge levels returned correctly, hidden suppresses badge data

- [x] **T-3.2** — Create `PriceBadge.svelte`
  - Props: `level`, `pct`. Renders green badge "✓ Good price" or amber badge "↑ Above average"
  - Hidden when level is "hidden" (parent controls visibility)
  - Deps: none
  - TDD: Manual — mount with each level, inspect DOM classes and text
  - Files: `src/lib/components/PriceBadge.svelte`
  - Time: 30min
  - AC: Green renders with class `badge--green`, amber with `badge--amber`, no badge when hidden

- [x] **T-3.3** — Wire PriceBadge into `ProductCard.svelte`
  - Add `priceInsight` state, fetch via `invoke('get_price_insight')` in `onMount` (after image load)
  - Render `{#if priceInsight && priceInsight.level !== 'hidden'}` → `<PriceBadge>`
  - Deps: T-3.1, T-3.2
  - TDD: Manual — load product list, verify badge appears green/amber per data
  - Files: `src/lib/components/ProductCard.svelte`
  - Time: 1h
  - AC: Badge renders conditionally, no cascading request (fetched after product data)

- [x] **T-3.4** — Write price insight tests
  - Deps: T-3.1
  - TDD: RED done in T-3.1. Add edge case: all prices equal → stddev=0 → hidden? No, green. Boundary tests: price exactly at threshold boundaries
  - Files: `commands/price_command.rs` (add test cases)
  - Time: 1h
  - AC: All 3 spec scenarios pass, boundary thresholds verified, `cargo test` green

## Group 4: alert-delivery (F3)

- [x] **T-4.1** — Create `alert_service.rs` with `AlertDispatcher` trait + 3 impls
  - Trait: `AlertDispatcher: Send + Sync { async fn send(); async fn test() }`
  - `AppNotificationAlert` — uses Tauri notification API (stub for now, full impl after Tauri 2 API confirmed)
  - `NtfyAlert` — HTTP POST to `https://ntfy.sh/{topic}` with JSON body (title, message, tags)
  - `WebhookAlert` — HTTP POST with JSON body `{title, message}`
  - Shared `validate_webhook_url()` — rejects non-http(s), empty, IP literals
  - Wire into `services/mod.rs`. Add `http_client: reqwest::Client` to `AppState` in `lib.rs`.
  - Deps: T-0.1 (zip + tauri-plugin-dialog in Cargo.toml), existing `reqwest` dep
  - TDD RED: Write tests for each dispatcher using `httpmock` (Ntfy POSTs correct URL, Webhook POSTs valid JSON, validation rejects bad URLs)
  - Files: `services/alert_service.rs`, `services/mod.rs`, `lib.rs`
  - Time: 2.5h
  - AC: All 3 dispatchers implemented, URL validation passes, no unsafe blocks

- [x] **T-4.2** — Add `test_alert_channel` to `settings_command.rs`
  - Reads channel + config, instantiates correct `AlertDispatcher`, calls `test()`, returns `AlertTestResult`
  - Validates channel string ("app" / "ntfy" / "webhook"), returns `"invalid_channel"` otherwise
  - Deps: T-4.1 (AlertDispatcher implementations), T-1.1 (settings_command.rs exists)
  - TDD RED: Test invalid channel returns error, valid channel dispatches test
  - Files: `commands/settings_command.rs`
  - Time: 1h
  - AC: test_alert_channel IPC returns success/failure, timeout at 5s, no crash on network error

- [x] **T-4.3** — Wire "Test Notification" button in `Settings.svelte`
  - Calls `invoke('test_alert_channel', {channel, config})` on button click
  - Shows result inline (✓ Sent! / ✗ Failed: message) with 5s timeout UX
  - Deps: T-4.2 (IPC exists), T-1.2 (Settings.svelte exists)
  - TDD: Manual — click test with each channel, verify feedback shown
  - Files: `src/lib/components/Settings.svelte`
  - Time: 1h
  - AC: Test button works for all 3 channels, feedback within 5s, error handled gracefully

- [x] **T-4.4** — Write alert service tests
  - Deps: T-4.1
  - TDD: All httpmock tests for NtfyAlert (correct URL, headers, retry-once), WebhookAlert (JSON body, 4xx handling, network failure), AppNotificationAlert (no-op returns Ok). URL validation: empty, non-http, IP literal, valid URL
  - Files: `services/alert_service.rs` (add `#[cfg(test)] mod tests`)
  - Time: 1.5h
  - AC: All 9 test cases from design §8.2 pass, httpmock assertions verify HTTP behavior

## Group 5: data-export (F4)

- [x] **T-5.1** — Create `export_service.rs` with ZIP generation
  - `ExportService { pool: SqlitePool }` — queries wishlist, price_history, settings → builds ZIP in memory via `zip` crate's `ZipWriter`
  - 3 files in ZIP: `wishlist.json`, `price_history.json`, `settings.json` (2-space indent)
  - Wire into `services/mod.rs`
  - Deps: T-0.1 (zip dep), T-0.3 (settings table), existing wishlist table
  - TDD RED: In-memory DB seeded with data → export to temp path → verify ZIP is valid with `zip-rs` read-back, 3 files present, JSON parseable
  - Files: `services/export_service.rs`, `services/mod.rs`
  - Time: 2h
  - AC: ZIP contains 3 valid JSON files, empty tables produce `[]`, errors handled (disk full → ExportError::Write)

- [x] **T-5.2** — Create `export_command.rs` with `export_data` IPC
  - Takes `path: String` (from frontend save dialog), delegates to `ExportService::export_to`
  - Wire into `commands/mod.rs`
  - Deps: T-5.1 (ExportService exists)
  - TDD RED: Test empty path returns write error, valid path returns ExportResult
  - Files: `commands/export_command.rs`, `commands/mod.rs`
  - Time: 45min
  - AC: IPC contract matches spec §4.6, errors mapped correctly

- [x] **T-5.3** — Add "Export All Data" button to `Settings.svelte`
  - Uses `@tauri-apps/plugin-dialog` `save()` to open native dialog (filter `.zip`)
  - Passes returned path to `invoke('export_data')`, shows success/size or error
  - Handle cancelled dialog (empty path → noop)
  - Deps: T-5.2 (IPC exists), T-0.1 (tauri-plugin-dialog registered), T-1.2 (Settings.svelte exists)
  - TDD: Manual — click export, dialog appears, confirm creates ZIP at chosen path
  - Files: `src/lib/components/Settings.svelte`
  - Time: 1.5h
  - AC: Save dialog opens with .zip filter, export creates valid ZIP, cancel is noop, result shown inline

- [x] **T-5.4** — Write export service tests
  - Deps: T-5.1, T-5.2
  - TDD: Full coverage from design §8.3: valid ZIP with data, empty tables, disk full error, permission denied
  - Files: `services/export_service.rs` + `commands/export_command.rs` (add test modules)
  - Time: 1h
  - AC: All 4 test scenarios pass, ZIP verified with `zip-rs` read-back, clippy clean

## Group 6: Integration Wiring

- [x] **T-6.1** — Register `tauri-plugin-dialog` + all IPC commands in `main.rs`
  - Add `.plugin(tauri_plugin_dialog::init())` to builder
  - Add all 6 commands to `invoke_handler`: `get_price_history`, `get_price_insight`, `get_setting`, `save_setting`, `test_alert_channel`, `export_data`
  - Deps: T-0.1 (plugin dep), T-1.1, T-2.2, T-4.2, T-5.2 (all commands exist)
  - TDD: `cargo check` + `cargo test` passes
  - Files: `src-tauri/src/main.rs`
  - Time: 30min
  - AC: All commands registered, `cargo build` succeeds

- [x] **T-6.2** — Add dialog permissions to `tauri.conf.json`
  - Add `"plugins": { "dialog": { "all": true } }` block
  - Deps: T-0.1 (plugin dep), T-6.1 (plugin registered)
  - TDD: `cargo build` succeeds with new permission block
  - Files: `src-tauri/tauri.conf.json`
  - Time: 10min
  - AC: JSON valid, build passes, dialog plugin works at runtime

---

## Phase Summary

| Group | Tasks | Focus | Est. Time |
|-------|-------|-------|-----------|
| 0 — Infrastructure | 3 | Migrations + deps | 1h |
| 1 — app-settings | 3 | Settings IPC + UI shell | 3.5h |
| 2 — price-history | 5 | Query, chart, detail page | 7.5h |
| 3 — price-insight | 4 | Insight SQL, badge, wiring | 3.5h |
| 4 — alert-delivery | 4 | 3 dispatchers, test button | 6h |
| 5 — data-export | 4 | ZIP service, IPC, button | 5h |
| 6 — Wiring | 2 | main.rs + tauri.conf.json | 40min |
| **Total** | **25** | | **~27h** |

## Implementation Order

Groups 0 → 1 → 2 → 3 → 4 → 5 → 6. Groups 2 and 4 can be parallelized after Group 0 is complete. Group 3 depends on Group 2 (same `price_command.rs` file). Group 5 depends on Group 1 (Settings.svelte). Group 6 is final.
