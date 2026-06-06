# Design: Add Price Drop Notifications

> **Change**: `add-price-drop-notifications`
> **Date**: 2026-06-05
> **Phase**: sdd-design
> **Mode**: hybrid (engram + openspec)

## Technical Approach

Close the loop on the half-built alerting infrastructure. Sync already has the only point in the app that knows "old price" vs "new price" — that knowledge is currently lost on every `INSERT OR REPLACE`. We make sync: (1) write each upserted price to `price_history` (table exists, no production writer), (2) run a pure `is_price_drop` detector against the row it just wrote, (3) collect drops that clear a 24h cooldown, and (4) hand them to `sync_command` to dispatch via the user's configured `AlertDispatcher`. Anti-spam is layered: first-observation suppression (`price_history` empty) → materiality (10% OR $50, both inclusive) → per-SKU cooldown. No new crates, no new background scheduler, no schema changes outside migration 007.

## Architecture Decisions

### Decision: Detect inside `upsert_products` (single pass)

**Choice**: Run `is_price_drop` per product inside the existing `upsert_products` loop, against the row just written to `price_history`. Collect results into `SyncResult.drops`.
**Alternatives considered**: (A) post-sync batch process via re-query of `products_meta`; (B) per-product background task spawned from sync.
**Rationale**: Co-locates the new write + read in one transaction, avoids a second full table scan, lets the detector run on the in-memory `RawProduct` (no need to re-deserialize). Trade-off: tighter coupling of sync to alerts, mitigated by keeping the detector as a pure function and the cooldown check in its own repo.

### Decision: Dedicated `price_drop_notifications` table

**Choice**: New table keyed by `sku` storing `(last_notified, last_price, channel)`.
**Alternatives considered**: (A) derive cooldown from a "notified" marker column on `price_history`; (B) keep state in `settings` (one row per SKU — doesn't scale).
**Rationale**: Bounded growth (~1 row per SKU that ever dropped), atomic upsert, queryable for future analytics. Index on `(sku, last_notified)` for fast cooldown checks.

### Decision: `AppNotificationAlert` real impl via `AppHandle` in `sync_command`

**Choice**: Keep `AppNotificationAlert` service-layer stub for unit tests. At the command layer, special-case `"app"` to call `app_handle.notification().builder()…show()` directly — the same pattern `test_alert_channel` already uses (`settings_command.rs:108-113`).
**Alternatives considered**: (A) inject `AppHandle` into the trait (breaks the existing design — `AlertDispatcher` deliberately has no Tauri dependency).
**Rationale**: Preserves the trait's testability. `AppHandle` stays out of the service layer (no `tauri` imports in `services/`).

### Decision: Thresholds as `pub const` + settings-keyed overrides

**Choice**: `RELATIVE_DROP_PCT = 0.10`, `ABSOLUTE_DROP_USD = 50.0`, `COOLDOWN_SECS: i64 = 86_400` as `pub const` in `services/price_drop.rs`. `sync_command` reads `drop_threshold_pct`, `drop_threshold_abs`, `cooldown_hours` from `settings` and falls back to the consts.
**Rationale**: Single source of truth; future UI surface is a one-line change. Matches `price_command.rs:11` pattern of named `pub const` thresholds.

## Data Flow

```
sync_catalog(url)
  └─ CatalogSyncService::sync_catalog
       └─ upsert_products(source, products)        [MODIFIED]
            for each product p:
              1. products_meta INSERT OR REPLACE    (unchanged)
              2. price_history INSERT (sku, price, source, now)   [NEW]
              3. is_price_drop(new, prev, thresholds)  [NEW]
                 → None on first-obs / increase / <10% AND <$50
              4. if Some(drop):
                   if cooldown_expired(sku, now):
                     drops.push(drop)                [NEW]
       return SyncResult { ..., drops: Vec<PriceDrop> }
  └─ sync_command                                     [MODIFIED]
       for each drop in result.drops:
         match settings.alert_channel:
           "app"     → app_handle.notification().builder()…show()
           "ntfy"    → NtfyAlert::send(...)
           "webhook" → WebhookAlert::send(...)
         on Ok:  price_drop_notifications.upsert(sku, now, new, channel)
         on Err: log + continue (next sync retries)
       return SyncResult to frontend
  └─ +page.svelte: toast "X drops, Y sent"            [NEW]
```

## File Changes

| File | Action | LoC | Notes |
|------|--------|-----|-------|
| `src-tauri/src/repository/sqlite/migrations/007_price_drop_notifications.sql` | Create | ~10 | New table + index, reuses existing BEGIN/END-aware `split_statements` |
| `src-tauri/src/repository/price_history.rs` | Modify | +25 | `get_last_price(sku)` + `record_price(sku, price, source, now)` + 3 tests |
| `src-tauri/src/services/price_drop.rs` (new) | Create | ~120 | `is_price_drop` pure fn + `Thresholds` + `PriceDrop` + 10 unit tests |
| `src-tauri/src/services/mod.rs` | Modify | +1 | `pub mod price_drop;` |
| `src-tauri/src/services/sync.rs` | Modify | +80 | `SyncResult.drops`; `upsert_products` writes `price_history` + collects drops; integration test |
| `src-tauri/src/commands/sync_command.rs` | Modify | +50 | Dispatch loop via `AlertDispatcher`; `AppHandle` bridge for "app" |
| `src-tauri/src/lib.rs` | Modify | +5 | (no `alert_dispatcher` field — dispatcher is built per-call from `settings.alert_channel`, matches `test_alert_channel_cmd` pattern) |
| `src/routes/+page.svelte` | Modify | +20 | Toast after `sync_catalog` resolves, using existing `SyncResult.drops` |
| `src/lib/types/sync.ts` (new) | Create | ~20 | Mirror `SyncResult.drops: PriceDrop[]` |
| **Total** | | **~330** | Within chained-PR budget (PR 1 ≤200, PR 2 ≤250) |

## Interfaces / Contracts

```rust
// services/price_drop.rs
pub const RELATIVE_DROP_PCT: f64 = 0.10;
pub const ABSOLUTE_DROP_USD: f64 = 50.0;
pub const COOLDOWN_SECS: i64 = 86_400;

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct PriceDrop {
    pub sku: String,
    pub previous_price: f64,
    pub new_price: f64,
    pub channel: String,            // "app" | "ntfy" | "webhook"
    pub reason: DropReason,         // Relative | Absolute
}

pub struct Thresholds { pub pct: f64, pub abs: f64, pub cooldown: i64 }

/// Pure, no I/O. `new_price = None` ⇒ first observation (no baseline).
pub fn is_price_drop(
    new_price: Option<f64>,
    previous_price: Option<f64>,
    thresholds: &Thresholds,
) -> Option<PriceDrop>;

// repository/price_history.rs (additions)
impl PriceHistoryRepo {
    pub async fn get_last_price(&self, sku: &str) -> Result<Option<f64>, sqlx::Error>;
    pub async fn record_price(
        &self, sku: &str, price: f64, source_id: &str, now: i64
    ) -> Result<(), sqlx::Error>;
}

// services/sync.rs (modified)
pub struct SyncResult {
    pub source_id: String,
    pub products_loaded: u32,
    pub products_updated: u32,
    pub state: SyncState,
    pub progress: f32,
    pub drops: Vec<PriceDrop>,   // NEW
}
```

`AppState` is unchanged — dispatcher is built per call from `settings.alert_channel`. `AlertDispatcher` trait stays as-is. No `AppHandle` enters the service layer.

## Testing Strategy

| Layer | What | How |
|-------|------|-----|
| Unit | `is_price_drop` (10 cases) | Pure-function tests: significant drop, small drop, exact-10%, exact-$50, increase, first-obs, prev-None, new-None, both-None, custom thresholds |
| Unit | `PriceHistoryRepo::get_last_price` / `record_price` (3 tests) | In-memory pool, insert + read, empty case, multi-source ordering |
| Integration | `upsert_products` writes `price_history` + detects drop | Mock catalog, assert `price_history` row count = products; assert `SyncResult.drops` populated when price drops 15% |
| Integration | Cooldown gate | Two syncs with same drop: first → 1 drop, second → 0 drops; mutate `last_notified` to simulate past expiry, assert next sync fires again |
| Integration | `AppNotificationAlert` real impl | `httpmock` for ntfy/webhook paths; `app` channel tested by reading from a mock settings (the `AppHandle` path is hard to test without a Tauri runtime, so we extract a `dispatch_native(app, drop) -> Result<(), String>` helper and unit-test it via a thin trait) |
| E2E (manual) | Toast | Smoke test in dev: trigger sync, see "X drops, Y sent" |

All tests via `make test-app` (`cargo test`). Strict TDD: RED (failing test) → GREEN (impl) → REFACTOR per task.

## Migration / Rollout

Migration 007 is forward-only:
```sql
CREATE TABLE IF NOT EXISTS price_drop_notifications (
  sku           TEXT PRIMARY KEY,
  last_notified INTEGER NOT NULL,
  last_price    REAL NOT NULL,
  channel       TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_price_drop_notifications_sku_ts
  ON price_drop_notifications(sku, last_notified);
```
Reuses existing migration runner (`split_statements` is BEGIN/END-aware — confirmed at `migrations/mod.rs:232`). No `schema_meta.db_version` bump logic needed; runner updates it.

Chained PRs (both target `main`, NOT stacked on each other):
- **PR 1 — Data layer (≤200 LoC)**: migration 007, `get_last_price` + `record_price` + tests, `services/price_drop.rs` (is_price_drop + Thresholds + PriceDrop + 10 tests). Zero user-visible change.
- **PR 2 — Wiring (≤250 LoC)**: `SyncResult.drops`, `upsert_products` integration, `sync_command` dispatch, `AppNotificationAlert` real impl, frontend toast + types, integration tests.

Rollback: revert PR 2 first (no schema change, sync returns to no-drops); PR 1 reverts cleanly (new table only).

## Edge Cases

| Case | Behaviour | Tested? |
|------|-----------|---------|
| First sync after ship (no `price_history` row) | `get_last_price` returns `None` → `is_price_drop` returns `None` → no drop | ✅ PDN S6 |
| Ntfy/webhook not configured (empty `alert_config`) | Sync still returns drops in `SyncResult.drops`; `sync_command` skips dispatch with `tracing::warn!` | ✅ SYNC req 4 |
| DB write failure during `price_history` INSERT | Log via `tracing::error!`, do NOT fail the whole sync | ✅ SYNC req 4 |
| Concurrent syncs | Already rejected by `SyncInProgress` at `sync.rs:81` | ✅ existing |
| `price_history` grows unbounded | No change vs today (already grows via tests; production writer is the only new contributor) | future retention job |
| `webhook` returns HTTP 500 | `last_notified` NOT updated → next sync retries | ✅ PDN S10 |
| `webhook` returns HTTP 4xx | Same as 5xx — treated as failure | ✅ PDN S10 |
| `price_history` row for SKU has same `price` as today | `is_price_drop` returns `None` (no decrease) | ✅ PDN S5 |
| `price` increases | `is_price_drop` returns `None` | ✅ PDN S5 |
| Same SKU, multiple sources | Each `(sku, source_id)` pair gets its own `price_history` row; cooldown is per-SKU (not per-source) | documented in spec |

## Risks

| Risk | Likelihood | Mitigation |
|------|------------|------------|
| False positives on first sync after ship | High (by design) | First-obs suppression: `price_history` empty → no drop. **Tested.** |
| `last_notified` race on two syncs in same hour | Med | Atomic upsert in cooldown table; second sync re-reads the just-written row. **Tested.** |
| Cooldown table growth | Low | Bounded (~1 row per SKU that ever dropped, ~8MB for 100k SKUs) |
| `AppHandle` injected into service layer by mistake | Low | Code review + the `AppNotificationAlert` stub stays a stub; the bridge lives in `sync_command` only |
| Threshold defaults (10% / $50) feel wrong in prod | Med | `pub const` + settings keys read; future UI is a small PR |
| Extra `INSERT` per product on sync (~10k → ~10k inserts) | Low | Negligible overhead; wrapped in the same transaction as `products_meta` if perf becomes an issue |
| `webhook` URL is SSRF-target (RFC 1918) | Existing | `validate_webhook_url` already rejects IP literals (`alert_service.rs:33`) — reused |

## Out of Scope (deferred to later changes)

- User-tunable threshold UI (settings keys are read; no UI to edit)
- Per-SKU opt-out column (could be added to 007 schema later without breaking)
- Notifications history/log view
- Background scheduler (sync is on-demand only today)
- Drops in the export ZIP (`price_drop_notifications` is local cooldown state, not user-meaningful history)
- Email / Slack / Discord channels

## Open Questions

None — all four open questions from the exploration were resolved by the proposal:
1. PR strategy → chained (PR 1 data, PR 2 wiring), both target `main`
2. Thresholds → 10% / $50 / 24h confirmed
3. `AppNotificationAlert` resolution → stub + command-layer bridge (matches `test_alert_channel` pattern)
4. Export of drops → excluded (cooldown is local state)

Ready for `sdd-tasks`.
