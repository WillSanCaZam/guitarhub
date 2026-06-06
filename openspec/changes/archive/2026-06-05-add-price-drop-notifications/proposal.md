# Proposal: Add Price Drop Notifications

> **Change**: `add-price-drop-notifications`
> **Date**: 2026-06-05
> **Phase**: sdd-propose
> **Mode**: hybrid (engram + openspec)

## Intent

Guitar buyers track gear to catch price drops. Today the workflow is manual: click **Sync**, eyeball the UI, and hope you noticed a deal. The infrastructure for proactive alerts is **already built** — `AlertDispatcher` trait, `NtfyAlert` / `WebhookAlert` / `AppNotificationAlert` impls, `tauri-plugin-notification`, settings UI for `alert_channel` — but **nothing wires it up**. A real `price_history` writer does not exist; sync is the only place that knows "old price" vs "new price" and it overwrites both. This change closes the loop: detect drops during sync, gate them with anti-spam rules, and dispatch via the user's configured channel. Outcome: a buyer sees a native notification the moment a tracked product drops ≥10% or ≥$50.

## Scope

### In Scope

- Migration **007** adding `price_drop_notifications` (sku PK, last_notified, last_price, channel).
- `CatalogSyncService::upsert_products` writes each upserted price to `price_history` (today, only tests do).
- New pure function `is_price_drop(new, prev, thresholds) -> Option<PriceDrop>` in `services/price_drop.rs`.
- 3-layer anti-spam: materiality (10% OR $50), 24h cooldown per SKU, first-observation suppression.
- `sync_command` dispatches collected drops via the configured `AlertDispatcher`.
- Frontend toast: "X price drops, Y sent" after sync.
- ~460 LoC, ~30 new tests, **strict TDD** (RED → GREEN → REFACTOR per task).

### Out of Scope

- User-tunable threshold UI (constants only; settings keys exist but not exposed).
- Dedicated notifications history/log view.
- Background scheduler (sync is on-demand only).
- Per-SKU opt-out column (column reserved for v2).
- Including drops in the export ZIP.

## Capabilities

### New Capabilities

- `price-drop-notifications`: full spec covering detection algorithm, cooldown rules, dispatcher integration, anti-spam guarantees, and the toast counter.

### Modified Capabilities

- `sync-service`: delta requiring `upsert_products` to write each upserted price to `price_history` and `SyncResult` to carry a `drops: Vec<PriceDrop>` field.

## Approach

**Detect-on-sync** in a single pass. Inside `upsert_products`, for each product: (1) read previous price from `price_history` (new `get_last_price` helper), (2) `INSERT OR REPLACE` into `products_meta` (unchanged), (3) `INSERT` into `price_history` (new), (4) run `is_price_drop` against the baseline, (5) if a drop fires, check cooldown via `price_drop_notifications`; if expired, push onto a `Vec<PriceDrop>` and continue. After the loop, `sync_command` reads `alert_channel` from settings, builds the dispatcher, and fires each drop. On send success, upsert the cooldown row; on failure, log and leave the cooldown unchanged so the next sync retries.

## Trigger Rules

| Layer | Rule | Default | Configurable |
|---|---|---|---|
| Strict drop | `new_price < previous_price` | always | n/a |
| Materiality | `(prev - new) / prev ≥ 0.10` **OR** `prev - new ≥ $50` | yes | `drop_threshold_pct`, `drop_threshold_abs` settings keys (read, no UI yet) |
| Cooldown | `last_notified + 24h > now` → skip | 24h | `cooldown_hours` setting key |
| First-obs | `get_last_price == None` → skip (baseline) | always | n/a |

## Affected Areas

| Area | Impact | Description |
|---|---|---|
| `src-tauri/src/repository/sqlite/migrations/007_price_drop_notifications.sql` (new) | New | `price_drop_notifications` table + index |
| `src-tauri/src/repository/sqlite/migrations/mod.rs` | Modified | Register 007 + 1–2 tests |
| `src-tauri/src/repository/price_history.rs` | Modified | New `get_last_price(sku) -> Option<f64>` + 3 tests |
| `src-tauri/src/repository/sqlite/price_drop_notifications.rs` (new) | New | `get_last_notified`, `upsert` (~50 LoC) |
| `src-tauri/src/services/price_drop.rs` (new) | New | `is_price_drop` + `PriceDrop` + 8–10 unit tests (~120 LoC) |
| `src-tauri/src/services/sync.rs` | Modified | Write `price_history` per upsert; collect drops; return `drops` on `SyncResult` (~80 LoC) |
| `src-tauri/src/services/alert_service.rs` | Modified | Factory `build_dispatcher(channel, config)` + threshold `pub const`s (~40 LoC) |
| `src-tauri/src/commands/sync_command.rs` | Modified | Dispatch loop; `AppHandle` bridge for "app" channel (~50 LoC) |
| `src/lib/components/SyncButton.svelte` / `+page.svelte` | Modified | Show "X drops, Y sent" toast (~20 LoC) |
| `src/lib/types/sync.ts` (new) | New | Mirror `SyncResult.drops` |

## Key Decisions

| # | Decision | Rationale |
|---|---|---|
| D1 | Detect inside `upsert_products` (Option A from exploration) | Single pass; has access to new price + history baseline; no second full table scan |
| D2 | Dedicated `price_drop_notifications` table (not settings-only) | Bounded growth (~1 row per SKU that ever dropped); atomic upsert; queryable for future analytics |
| D3 | Keep `AppNotificationAlert` as service-layer stub; `sync_command` bridges to `tauri_plugin_notification` via `AppHandle` | Preserves trait purity (no `AppHandle` in dispatcher); command layer is the only place `AppHandle` exists |
| D4 | Thresholds as `pub const` + settings-keyed overrides (read but not UI-exposed) | Single source of truth; future UI surface is a 1-line change; avoids scattered magic numbers |

## Chained PR Strategy

Total is ~460 LoC, **above the 400-line review budget**. Ship as two stacked PRs:

- **PR 1 — Data layer (≤200 LoC, stacked to main)**: migration 007, `get_last_price` helper, `price_drop_notifications` repo, `is_price_drop` pure fn + tests. **Zero user-visible change.** Landable and revertable in isolation.
- **PR 2 — Wiring (≤250 LoC, stacked to main)**: modify `upsert_products` to write `price_history` + emit `drops`; modify `sync_command` to dispatch; add toast; add type file. End-to-end behavior lands here.

Both target `main` once their review is green (not stacked on each other). PR 1 is the foundation; PR 2 is the visible payoff.

## Risks

| Risk | Likelihood | Mitigation |
|---|---|---|
| Spurious drop on first sync after ship (no baseline) | High (by design) | First-observation suppression: `get_last_price == None` → skip. **Tested explicitly.** |
| `price_history` unbounded growth | Low | ~one row per (sku, sync). At 6h cadence × 10k SKUs × 1y ≈ 14.6M rows ≈ 300MB. Future: retention job. |
| Cooldown race on 2 syncs in same hour | Med | `last_notified` upsert is atomic; 2nd sync re-reads it → still inside 24h → skip. **Tested.** |
| Dispatcher HTTP failure | Med | `last_notified` NOT updated on failure → next sync retries the same drop. Bounded retry, no silent loss. |
| `AppHandle` injected into service layer by mistake | Low | Code review + the `AppNotificationAlert` stub stays a stub; bridge lives in `sync_command` only. |
| Threshold defaults feel wrong in production | Med | Defaults are named `pub const`; `drop_threshold_pct`, `drop_threshold_abs`, `cooldown_hours` settings keys are read; future UI is a tiny PR. |

## Rollback Plan

PRs are isolated:
1. **Revert PR 2 first** (no schema change). Sync returns to "no drops field, no dispatch". `price_history` writer is also reverted, so history stops accumulating. Existing data untouched.
2. **Revert PR 1** (drops table 007). Migration 007 is a single new table; forward path's `down` migration drops it. Existing `price_history` rows are unaffected.
3. **Worst case**: keep PR 1, revert only PR 2. The new table sits empty; no functional change visible to the user.

## Dependencies

- `tauri-plugin-notification` (already registered, permissions granted) — no new crate.
- `sqlx` (already in use) — no new migration runner changes.
- `AlertDispatcher` trait (already exists) — reused as-is.

## Success Criteria

- [ ] `make test` is green; all new tests pass.
- [ ] After one sync of N products, `price_history` has N rows with non-null `recorded_at`.
- [ ] After a second sync with a simulated 15% drop on one product, `SyncResult.drops` has exactly 1 entry and `price_drop_notifications` has 1 row.
- [ ] A third sync within 24h with the same drop returns `drops: []` (cooldown holds).
- [ ] `AlertDispatcher::send` is invoked exactly once per cooldown window per SKU per channel (assertion in tests with `httpmock`).
- [ ] All three channels covered: `NtfyAlert`, `WebhookAlert`, `AppNotificationAlert` (stub path).
- [ ] First sync after install produces `drops: []` for every product (first-observation suppression verified).
- [ ] Strict TDD: every change has a failing test written first (RED), then implementation (GREEN), then refactor.
