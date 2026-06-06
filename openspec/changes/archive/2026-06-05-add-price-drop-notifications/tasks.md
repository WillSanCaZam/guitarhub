# Tasks: Add Price Drop Notifications

> **Change**: `add-price-drop-notifications` · **Mode**: hybrid · **TDD**: strict

## Review Workload Forecast

Estimated changed lines: PR 1 ≈ 200 · PR 2 ≈ 250 · Total ≈ 450.
Work units: PR 1 (data layer) → PR 2 (wiring); both target `main`.
Delivery strategy: force-chained. Chain strategy: stacked-to-main.

Decision needed before apply: No
Chained PRs recommended: Yes
Chain strategy: stacked-to-main
400-line budget risk: Low

## Phase 1: PR 1 — Data Layer (≤200 LoC, → `main`)

- [x] 1.1 **RED** `migrations/mod.rs` — `migration_007_creates_price_drop_notifications_table`: extend `apply_full_migration_chain` w/ 007; assert `PRAGMA table_info(price_drop_notifications)` = 4 cols, `sku` PK. Fails.
- [x] 1.2 **GREEN** Create `007_price_drop_notifications.sql`: `CREATE TABLE price_drop_notifications (sku TEXT PRIMARY KEY, last_notified INTEGER NOT NULL, last_price REAL NOT NULL, channel TEXT NOT NULL)` + `CREATE INDEX idx_price_drop_notifications_notified ON price_drop_notifications(last_notified)`. 1.1 passes.
- [x] 1.3 **RED** `repository/price_history.rs` — 3 tests: `get_last_price_returns_none_when_empty`; `record_price_then_get_last_price_returns_it`; `get_last_price_returns_most_recent_across_sources`. Fails (methods missing).
- [x] 1.4 **GREEN** `get_last_price` = `SELECT price FROM price_history WHERE sku=?1 ORDER BY recorded_at DESC LIMIT 1`. `record_price(sku,price,source_id,recorded_at)` = `INSERT INTO price_history (...) VALUES (?1,?2,?3,?4)`. 1.3 passes.
- [x] 1.5 **RED** New `services/price_drop.rs` + `pub mod price_drop;` in `services/mod.rs`. 10 tests for PDN S1–S10: sig drop, small drop, exact 10%, exact $50, increase, `prev=None`, both `None`, `new=None`, reason=Absolute, custom Thresholds. Fails (compile).
- [x] 1.6 **GREEN** `pub const RELATIVE_DROP_PCT=0.10; ABSOLUTE_DROP_USD=50.0; COOLDOWN_SECS=86_400;`. `Thresholds{pct,abs,cooldown}` w/ Default. `enum DropReason{Relative,Absolute}`. `struct PriceDrop{sku,previous_price,new_price,channel,reason}` (Serialize,Clone,PartialEq). Pure `is_price_drop(sku,new:Option<f64>,prev:Option<f64>,thresholds,channel)->Option<PriceDrop>`. 1.5 passes.
- [x] 1.7 **REFACTOR** ≤200 LoC, clippy clean.

## Phase 2: PR 2 — Wiring (≤250 LoC, → `main`)

- [ ] 2.1 **RED** `services/sync.rs` — `sync_result_has_drops_field_empty_initially`: assert `.drops.is_empty()`. Fails (compile).
- [ ] 2.2 **GREEN** Add `pub drops: Vec<PriceDrop>` to `SyncResult`; import PriceDrop; constructor uses `drops: vec![]`. 2.1 passes.
- [ ] 2.3 **RED** `sync.rs` tests — `upsert_products_writes_price_history_rows` (1 product → 1 history row, `recorded_at≈now`); `upsert_products_detects_15pct_drop` (seed $1000, ingest $850 → 1 drop, `previous_price==1000.0`); `upsert_products_first_observation_no_drop` (empty history → drops empty).
- [ ] 2.4 **GREEN** `upsert_products` returns `(u32,u32,Vec<PriceDrop>)`. Loop: `get_last_price` → products_meta INSERT OR REPLACE → `record_price` (log+continue on fail) → `is_price_drop` w/ `Thresholds::default()` + `channel="app"` → push Some. `sync_catalog` builds `SyncResult{drops,..}`. 2.3 passes.
- [x] 2.5 **RED** New `repository/price_drop_notifications.rs` + `pub mod` in `repository/mod.rs`. Test `price_drop_repo_upsert_and_get_last_notified_roundtrip`. Add `sync_command_dispatches_drops_via_mock_dispatcher` (fixture + mock dispatcher, assert `send` once).
- [x] 2.6 **GREEN** `pub fn try_build_dispatcher(channel, config)` in `commands/sync_command.rs` (Ntfy/Webhook; App handled at command layer). Refactor `commands/sync_command.rs`: sync_catalog → read `settings.alert_channel`+`alert_config` → `try_build_dispatcher` → for each drop `dispatcher.send`; `Ok` → `price_drop_repo.upsert`; `Err` → log+continue; return `SyncResult` w/ new `drops_sent:u32`. Repo: `upsert` (`INSERT OR REPLACE`) + `get_last_notified` (`SELECT last_notified WHERE sku=?1`). 2.5 passes; PDN S9/S10 covered.
- [x] 2.7 **GREEN** Frontend: `+page.svelte` shows toast `"X price drops, Y sent"` from `result.drops.length` + new `drops_sent`. New `lib/stores/sync.ts` mirrors `SyncResult`+`PriceDrop`; shared via Svelte store between layout and page. Smoke test all-succeed and partial-failure texts.
- [x] 2.8 **REFACTOR** ≤250 LoC, no `AppHandle` in `services/`, `make test` green.

## Phase 3: Verification

- [ ] `make test` green.
- [ ] Dev: seed $1000 → sync $800 → "1 price drops, 1 sent"; 3rd sync within 24h → "0 price drops, 0 sent".
- [ ] Ntfy+Webhook via `httpmock`; App via stub. PR 1 & PR 2 each safely revertible.
