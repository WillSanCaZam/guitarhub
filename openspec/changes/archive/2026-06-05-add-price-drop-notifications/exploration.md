# Exploration: add-price-drop-notifications

**Change name**: `add-price-drop-notifications`
**Date**: 2026-06-05
**Phase**: sdd-explore

---

## Current State

### What exists today

| Component | Status | Location |
|---|---|---|
| `AlertDispatcher` trait | ✅ complete, tested | `src-tauri/src/services/alert_service.rs:63` |
| `NtfyAlert` impl | ✅ complete, tested | `alert_service.rs:120` |
| `WebhookAlert` impl | ✅ complete, tested | `alert_service.rs:215` |
| `AppNotificationAlert` | ⚠️ stub only (logs via `tracing::info!`) | `alert_service.rs:88` |
| `tauri-plugin-notification` | ✅ registered, permissions granted | `Cargo.toml:27`, `main.rs:18`, `capabilities/default.json` |
| Settings UI (channel + config) | ✅ complete | `src/lib/components/Settings.svelte` |
| `test_alert_channel` IPC cmd | ✅ complete, tested | `commands/settings_command.rs:99` |
| `AlertChannel` enum (App / Ntfy / Webhook) | ✅ complete, tested serialization | `alert_service.rs:7` |
| `CatalogSyncService` state machine | ✅ complete, tested | `services/sync.rs:27` |
| `sync_catalog` IPC cmd | ✅ complete | `commands/sync_command.rs:8` |
| `price_history` table (schema) | ✅ exists (migrations 001, 004) | `migrations/001_init.sql:76`, `004_add_price_source.sql` |
| `PriceHistoryRepo` (read queries) | ✅ complete (`get_history`, `get_insight`) | `repository/price_history.rs:40` |
| `wishlist` table | ✅ exists (migration 006) | `migrations/006_wishlist_schema.sql` |
| **Production writer for `price_history`** | ❌ **MISSING** | — |

### The critical gap (verified by grep)

A grep for `INSERT INTO price_history` across `src-tauri/src/**/*.rs` returns **only test code**:

| File | Lines | Production? |
|---|---|---|
| `repository/price_history.rs:489` | `insert_point` test helper | ❌ test |
| `commands/price_command.rs` (10 occurrences) | test fixtures | ❌ test |
| `services/export_service.rs:260, 271` | inside `seed_data()` test helper | ❌ test |
| `migrations/mod.rs:1053, 1059, 1083` | test fixtures | ❌ test |

**`CatalogSyncService::upsert_products` uses `INSERT OR REPLACE INTO products_meta` and never writes to `price_history`.** The product's old price is therefore lost on every sync — there is no "previous price" to compare against.

### How `AppNotificationAlert` is wired today

- The trait is implemented and unit-tested, but the `AppNotificationAlert` is a **service-layer stub** that just calls `tracing::info!` (line 98).
- Real native notifications are sent **only** by `test_alert_channel` in `settings_command.rs:108-113`, which uses `tauri_plugin_notification::NotificationExt` directly with the `AppHandle`.
- The `AppState` struct (`lib.rs:14-19`) holds `pool`, `image_cache_service`, `http_client` — **no `AlertDispatcher` and no `AppHandle`** is stored. There is no service or background scheduler that invokes a dispatcher when a sync completes.

### Anti-spam primitives

- No `notifications_log` table exists.
- No `notified_at` timestamp is stored anywhere for a SKU.
- No cooldown logic.
- No per-SKU user toggle for "do not notify".

### UI surface for notifications

- No notifications log/history view exists in `src/lib/components/`.
- `Settings.svelte` is the only place that mentions "alert".
- No "watchlist" concept — there is a `wishlist` table, but it is not used as a notification target and the export service exports it as a separate file (`wishlist.json`).
- No user-facing notification preference beyond `alert_channel` + `alert_config` (string).

### What "current price vs previous price" looks like today

The only data sources that could answer "did the price drop?" are:

1. `products_meta.price` — the *current* synced price, overwritten on every sync.
2. `price_history` — historical observations. `PriceHistoryRepo::get_insight` already has a subquery returning the most recent recorded price for a SKU (`(SELECT price FROM price_history WHERE sku = ?1 ORDER BY recorded_at DESC LIMIT 1)`).

So the *plumbing* for "previous price" is half-built: the table and the read query exist, but **no code path writes to it** in production.

---

## Gap Analysis

For price drop notifications to work end-to-end, the following pieces are missing:

| # | Missing piece | Severity | Owner |
|---|---|---|---|
| G1 | Sync must write each upserted price to `price_history` | **Blocker** | `services/sync.rs` |
| G2 | "Previous price" comparison query (last row of `price_history` for SKU) | **Blocker** | new helper, can be added to `PriceHistoryRepo` |
| G3 | `AlertDispatcher` is not stored on `AppState`; sync service can't fire alerts | **Blocker** | `lib.rs` + `services/alert_service.rs` |
| G4 | `AppNotificationAlert` is a stub — needs a production `AppHandle`-backed variant OR a separate `dispatch_native_notification` helper at the command/service boundary | **Blocker** | `services/alert_service.rs` or new `services/notification_service.rs` |
| G5 | Anti-spam table / column to record "last notified at" per SKU | **Blocker** | new migration 007 |
| G6 | Trigger rules: % threshold, absolute threshold, per-SKU enable, cooldown | **Design decision** | new module |
| G7 | Frontend: notifications history/log view | Optional, not in MVP | new Svelte component |
| G8 | User-tunable threshold (per-channel or per-app) in Settings | Optional, not in MVP | `Settings.svelte` |
| G9 | Wire background scheduler so notifications fire even when no IPC is in flight | **Out of scope** for v1 (sync is manual / on-demand today) | n/a |

### Why G1 is the linchpin

Without G1, there is no "previous price" — `products_meta.price` is overwritten in place. The user could see the current price drop in the UI, but the **system** would have no record of what it was 5 minutes ago. Every comparison must be derived from `price_history`, so writing to `price_history` is non-negotiable.

### Anti-spam strategy before G1 is solved is moot

Even with G1 fixed, if we fire on every sync we will spam the user: the GHA cron runs every 6 hours and may detect "drops" of 1 cent on products that the user has never opened. Cooldown (per-SKU `last_notified_at`) must ship together with the trigger.

---

## Trigger Strategy

### When does a notification fire?

A price-drop notification fires when, **during a single `sync_catalog` call**, a product's new price satisfies ALL of the following:

1. **Strict drop**: `new_price < previous_price` (strict less-than, not `<=`, to avoid noise on identical prices).
2. **Materiality**: the drop is significant. Two layered checks, OR-combined:
   - **Relative drop**: `pct_drop = (previous - new) / previous >= 0.10` (10% lower).
   - **Absolute drop**: `previous - new >= $50` (catches 1% drops on $5000 items that matter to the user).
3. **Cooldown**: no notification has been sent for this SKU within the last `cooldown_hours` setting (default 24h). A user might also explicitly opt out per-SKU (out of scope for v1).
4. **Channel configured**: `alert_channel` setting is set. If `alert_channel` is empty/missing, the sync still records the drop in `price_history` but no notification fires. (Default in Settings.svelte is `'app'`, so this is the implicit behaviour.)
5. **New SKU (first observation)**: skip — there is no "previous" to compare against. (No notification on the first ever sync of a product; subsequent syncs notify.)

### Why OR-combine relative and absolute?

- A 2% drop on a $200 pedal is irrelevant noise; a 2% drop on a $4000 amp is $80 saved and very interesting.
- A 15% drop on a $30 string set is interesting; the same % on a $50 pedal is uninteresting.
- A flat rule (10% AND $50) misses legitimate drops in both tails; OR catches both.

### Where the trigger lives

A new `PriceDropDetector` (pure function, in `services/price_drop.rs` or similar):

```rust
fn is_price_drop(
    new_price: f64,
    previous_price: Option<f64>, // None = first observation
    threshold_pct: f64,           // default 0.10
    threshold_abs: f64,           // default 50.0
) -> Option<PriceDrop> {
    let prev = previous_price?;
    if new_price >= prev { return None; }                    // strict less-than
    let drop_abs = prev - new_price;
    let drop_pct = drop_abs / prev;
    if drop_pct >= threshold_pct || drop_abs >= threshold_abs {
        Some(PriceDrop { previous: prev, new: new_price, drop_abs, drop_pct })
    } else {
        None
    }
}
```

### Where the trigger runs

Two options:

| Option | Pros | Cons |
|---|---|---|
| **A: Inside `CatalogSyncService` (per product)** | Co-located with the price write; immediate; can capture `synced_at` once | Tightly couples sync to alerts; harder to unit-test in isolation |
| **B: After sync completes, batch process** | Cleaner separation; one query, one dispatcher loop | Two passes; the in-memory new prices are not available without re-querying |

**Recommendation**: Option A. During `upsert_products`, after the `INSERT OR REPLACE INTO products_meta`, do: (1) read the previous price from `price_history` for that SKU, (2) write the new price to `price_history`, (3) run the trigger, (4) if it fires, push a `PriceDrop` onto a `Vec` that the service returns in `SyncResult`. The caller (`sync_command`) iterates the vec and dispatches each via the configured channel. This is testable end-to-end with `httpmock` and avoids re-querying.

### Cooldown

Stored in a new `price_drop_notifications` table (migration 007):

```sql
CREATE TABLE price_drop_notifications (
  sku           TEXT PRIMARY KEY,
  last_notified INTEGER NOT NULL,        -- epoch seconds
  last_price    REAL NOT NULL,           -- last notified "new" price
  channel       TEXT NOT NULL DEFAULT '' -- which channel handled it
);
```

The sync upserts this row only when a drop fires. To check cooldown: `last_notified + cooldown_secs > now` ⇒ skip. Default cooldown = 24h (86_400s), configurable via `cooldown_hours` setting.

---

## Data Flow

```
User clicks "Sync" in UI
  └─→ invoke('sync_catalog', { url })
        └─→ CatalogSyncService::sync_catalog(url)
              │
              ├─ Download + parse JSON (unchanged)
              ├─ State machine: downloading → validating → sanitizing → inserting → done
              │
              └─ upsert_products(source_id, products)         [MODIFIED]
                   For each product p:
                     1. SELECT price FROM products_meta WHERE sku = p.sku
                          → previous_in_catalog_price (may be NULL on first sync)
                     2. INSERT OR REPLACE INTO products_meta (...) [unchanged]
                     3. SELECT price FROM price_history
                          WHERE sku = p.sku
                          ORDER BY recorded_at DESC LIMIT 1
                          → previous_history_price (may be NULL)
                     4. effective_previous = previous_history_price
                                          ?? previous_in_catalog_price
                     5. is_price_drop(p.price, effective_previous, ...)
                          → Some(drop) or None
                     6. If Some(drop):
                          a. Check cooldown (price_drop_notifications.last_notified)
                          b. If cooldown expired, push PriceDrop to Vec<PriceDrop> drops
                     7. INSERT INTO price_history (sku, price, recorded_at, source_id)
                          VALUES (p.sku, p.price, now, source_id)
                           [ALWAYS — first observation is just a baseline]
              │
              └─ Returns SyncResult { source_id, products_loaded, products_updated,
                                       state, progress, drops: Vec<PriceDrop> }
                                                  [NEW FIELD: drops]
        └─→ sync_command: dispatch each drop via AlertDispatcher
              │
              ├─ Read alert_channel from settings
              ├─ Build dispatcher:
              │     "app"     → AppNotificationAlert (service-layer stub)
              │     "ntfy"    → NtfyAlert::new(topic)
              │     "webhook" → WebhookAlert::new(url)
              └─ For each drop: dispatcher.send(title, message, &http_client)
                    title   = "Price drop: <product name>"
                    message = "Was $<prev>, now $<new> (<pct>% off, $<abs> less)"
              │
              └─ On success: UPSERT price_drop_notifications (last_notified = now, last_price = new)
              └─ On failure: log via tracing::error!, do NOT update last_notified
                    (lets the next sync retry the dispatch)

Frontend receives SyncResult
  └─→ SyncButton displays "X price drops detected, Y notifications sent"
        (NEW — a small toast or counter, NOT a full notifications log in v1)
```

### Why "service-layer stub" for `AppNotificationAlert` is fine here

The trait was designed with this in mind. The `send()` method on the stub only logs, but in production the `sync_command` caller can:
- Either keep using `AppNotificationAlert` (stub) and rely on a `Tauri` window event / frontend listener to show in-app toasts.
- OR branch at the command layer: if channel == "app", call `app.notification().builder()...show()` directly, the same way `test_alert_channel` does.

The cleanest path is **the second one** — `sync_command` (which has `AppHandle` available via the `#[tauri::command]` macro) special-cases "app" to use `NotificationExt`, exactly mirroring `test_alert_channel`. The trait's `AppNotificationAlert` stub is then used only by tests.

---

## Anti-Spam Approach

Three layered guards. All three MUST hold for a notification to fire.

| Layer | Mechanism | Default | Configurable? |
|---|---|---|---|
| **1. Materiality** | `is_price_drop()` function: relative ≥10% OR absolute ≥ $50 | yes | yes (new settings `drop_threshold_pct`, `drop_threshold_abs`) |
| **2. Cooldown** | `price_drop_notifications.last_notified + cooldown_hours * 3600 > now` ⇒ skip | 24h | yes (setting `cooldown_hours`) |
| **3. First-observation suppression** | No previous price recorded (neither in `price_history` nor `products_meta`) ⇒ skip | always | n/a |

Optional v2 (not in scope):
- Per-SKU opt-out (would need a UI element on the product card).
- Per-source opt-out (some sources are noisier than others).
- Daily digest instead of immediate notification (would batch all drops in a 24h window and send one summary at e.g. 8am).

### Failure-mode anti-spam

If a dispatcher send fails (e.g. ntfy.sh is down), do **not** update `last_notified`. The next sync will retry, but with the same cooldown. This is intentional: silent retry on next sync is better than losing a drop or spamming.

### Edge cases the detector must handle

| Case | Behaviour |
|---|---|
| New product, first ever sync | No `price_history` row, no `products_meta` row (or row with no prior price) → no notification. The price is still written to `price_history` as the baseline. |
| Product was out of stock, now in stock at a new price | The `products_meta` row may have a stale `price` from months ago; if it's lower than the new in-stock price, the system correctly identifies "no drop". If higher, the new in-stock price might look like a drop — this is *correct* behaviour and the user wants to know. |
| Price unchanged across syncs | `new >= previous` → no notification. |
| Currency change | Currently, the codebase does not normalize currencies (USD only today). If a product changes currency, this is a data anomaly, not a price drop. v1 will treat it as a drop if numeric value drops. Out of scope to fix here. |
| Same SKU, multiple sources | Each (sku, source_id) is treated independently. A drop on Reverb does not affect a drop notification for the same SKU on eBay. (Consistent with how `get_insight` already aggregates per source.) |

---

## Implementation Scope

Estimate using the same pattern as the `enhance-price-insight-confidence` change (single PR, additive, all-under-budget):

| File | Change | LoC |
|---|---|---|
| `src-tauri/src/repository/sqlite/migrations/007_price_drop_notifications.sql` (new) | New table + index | ~10 |
| `src-tauri/src/repository/sqlite/migrations/mod.rs` | Register migration 007; add 1–2 tests for the new table | ~40 |
| `src-tauri/src/repository/price_history.rs` | New method `get_last_price(sku) -> Option<f64>` (~10 LoC + 3 tests) | ~25 |
| `src-tauri/src/repository/sqlite/price_drop_notifications.rs` (new) | New repo: `get_last_notified(sku) -> Option<NotifiedRow>`, `upsert(sku, ts, price, channel)` | ~50 |
| `src-tauri/src/services/sync.rs` | In `upsert_products`: query prev price from `price_history`, insert into `price_history`, run detector, push to `drops` vec; add `PriceDrop` struct; add `drops: Vec<PriceDrop>` to `SyncResult` | ~80 |
| `src-tauri/src/services/price_drop.rs` (new) | `is_price_drop()` pure function + `PriceDrop` struct + threshold constants + 8–10 unit tests | ~120 |
| `src-tauri/src/services/alert_service.rs` | Factory `build_dispatcher(channel, config) -> Result<Box<dyn AlertDispatcher + Send + Sync>, AppError>`; default thresholds `pub const` | ~40 |
| `src-tauri/src/lib.rs` | Add `alert_dispatcher_factory: ...` or just the http_client already in AppState — no new field needed if dispatcher is built per-call (likely simpler) | ~5 |
| `src-tauri/src/commands/sync_command.rs` | After `svc.sync_catalog(...)`, dispatch each drop; special-case "app" to use `NotificationExt`; return `SyncResult` with `drops` | ~50 |
| `src-tauri/src/main.rs` | No change (commands already registered) | 0 |
| `src/lib/components/SyncButton.svelte` or `+page.svelte` | Display "X price drops, Y sent" toast on completion | ~20 |
| `src/lib/types/sync.ts` (new) | Mirror `SyncResult` with `drops: PriceDrop[]` | ~20 |
| **Total** | | **~460 lines** |

### 400-line review budget

This is **slightly over** the default 400-line budget. Two options:

1. **Single PR, exception accepted** — the change is additive and tightly scoped.
2. **Chained PRs (2)**:
   - **PR 1 — Data layer** (≤ 200 LoC): migration 007, `price_history.get_last_price`, `price_drop_notifications` repo, `is_price_drop()` function + tests. **No behavior change visible to user.**
   - **PR 2 — Wiring** (≤ 250 LoC): modify `sync.rs` to write `price_history` + emit `drops`, modify `sync_command` to dispatch, modify `+page.svelte` to show toast, add type file.

**Recommendation**: chained PRs. The data layer is independently testable in isolation and the wiring PR is a small surface on top. This matches the `fix-integration-bugs` pattern.

### What we explicitly do NOT do in v1

- No per-SKU opt-out UI
- No notifications log/history view
- No daily digest
- No user-tunable thresholds UI (constants in code for now, settings keys are read but no UI to edit them)
- No background scheduler (notifications fire only when user triggers a sync, which is the current state)
- No email channel
- No Slack/Discord channel (would need its own dispatcher; defer)

---

## Affected Areas

- `src-tauri/src/services/sync.rs` — write to `price_history` on every upsert, run detector, collect drops
- `src-tauri/src/services/price_drop.rs` (new) — pure detector function
- `src-tauri/src/services/alert_service.rs` — dispatcher factory
- `src-tauri/src/commands/sync_command.rs` — dispatch after sync
- `src-tauri/src/repository/price_history.rs` — new `get_last_price()` method
- `src-tauri/src/repository/sqlite/price_drop_notifications.rs` (new) — cooldown state
- `src-tauri/src/repository/sqlite/migrations/007_price_drop_notifications.sql` (new) — schema
- `src-tauri/src/repository/sqlite/migrations/mod.rs` — register migration
- `src/lib/types/sync.ts` (new) — TS mirror of `SyncResult.drops`
- `src/lib/components/SyncButton.svelte` or `src/routes/+page.svelte` — show toast on drops

**No** changes needed to: `tauri.conf.json` (notification permissions already granted), `Cargo.toml` (all deps already present), `Settings.svelte` UI in v1 (existing channel config is reused), `mvp-foundation` or other archived specs.

---

## Risks

1. **Backfill on first run** — existing users have a populated `products_meta` but an empty `price_history`. The first sync after this change ships will detect "drops" on every product where the new price is lower than the current `products_meta.price` (which is the same as the new price, so… actually, no drop will be detected). **The risk is the inverse**: a product whose catalog `price` was just refreshed with a HIGHER number will be silently re-baselined. Verify: a 10% drop on a product that's never been seen will *not* fire (no previous in `price_history` or `products_meta.price`). The first sync populates `price_history` as the baseline. Subsequent syncs can detect drops. ✅ Safe.

2. **Concurrency** — sync runs while the user is also viewing a product. The single `INSERT OR REPLACE` into `products_meta` is atomic. The new `INSERT INTO price_history` is also atomic. The detector runs in Rust, not in a transaction with the writes, but the worst case is a duplicate dispatch (the same drop fires twice). Mitigation: write a `(sku, recorded_at)` unique index so the same observation can't be inserted twice in the same second; this is cheap insurance.

3. **Cooldown table growth** — `price_drop_notifications` will have ~one row per SKU that ever had a drop fired. For 100k SKUs this is 100k rows, ~8 MB. Bounded and safe.

4. **HTTP failure on dispatch** — if ntfy.sh is unreachable, the dispatcher retries once (existing behaviour in `send_inner`). If still failing, the drop is logged and `last_notified` is **not** updated. The next sync will retry. **Worst case**: user sees a delay, not a loss. ✅ Safe.

5. **Frontend toast vs. native notification confusion** — the user might think they're getting in-app toasts when the channel is "webhook", or vice versa. Mitigation: the toast shows the count + channel name explicitly, e.g. "3 price drops — 3 sent via ntfy".

6. **Threshold defaults** — 10% and $50 are chosen by intuition. Different markets (vintage vs. new) may want different defaults. Mitigation: named `pub const` so they are easy to retune; settings keys (`drop_threshold_pct`, `drop_threshold_abs`, `cooldown_hours`) are read from `settings` table even if no UI exposes them yet. Future change adds the UI.

7. **No `AppHandle` in `AlertDispatcher` trait** — the existing trait was deliberately designed without `AppHandle` to keep service-layer code testable. The command layer is the right place to bridge to `tauri_plugin_notification`. The plan is consistent with that separation.

8. **Sync takes longer** — writing to `price_history` on every product is one extra `INSERT` per product. For 10k products this is ~10k inserts in a single transaction, dominated by the existing `INSERT OR REPLACE INTO products_meta`. Negligible overhead. Mitigation: wrap the per-product writes in a single transaction if perf becomes an issue (defer until measured).

9. **PriceHistoryRepo::get_insight correctness** — `get_insight` already counts points and computes min/avg, and *its* read patterns do not change. The new writer is purely additive. No risk to existing insights/UI badges.

10. **Spec drift** — the `sync-service` spec (`openspec/specs/sync-service/spec.md`) will need a delta to specify "MUST write each upserted price to `price_history`". This is a low-risk additive change to the spec.

---

## Open Questions

These are decisions the user (or the orchestrator via `sdd-propose`) should make before implementation:

1. **Single PR or chained PRs?** Recommendation: chained (PR 1 = data layer, PR 2 = wiring). Confirm with user.
2. **Thresholds**: 10% / $50 / 24h cooldown — confirm or override.
3. **Cooldown table vs. settings-only**: do we store `last_notified` in a table (recommended) or derive it from `price_history` (looks at the last "notified" marker column on each row)? The table approach is much cleaner.
4. **Toast UI**: a small counter in the existing SyncButton, or a dedicated banner / modal? v1 recommendation: minimal counter.
5. **Background sync / scheduler**: confirmed out of scope for v1, but should we open a separate change for it?
6. **Per-SKU opt-out**: confirmed out of scope for v1, but should we leave a column in `price_drop_notifications` reserved for it? (e.g. `opted_out INTEGER NOT NULL DEFAULT 0` so a future change can add it without migration.)
7. **`AppNotificationAlert` resolution**: keep the stub and let the command layer use `NotificationExt` directly (matches existing `test_alert_channel` pattern), or upgrade the stub to take an `AppHandle`? The first is more testable; the second is more uniform. Recommendation: option 1.
8. **Should drops appear in the export ZIP?** (Currently `price_history` is exported; the new `price_drop_notifications` table is local-only state.) Recommendation: do NOT export — it is local cooldown state, not user-meaningful history. If the user re-installs, the cooldown resets, which is the correct behaviour.

---

## Recommendation

This is a **medium-sized, additive change** with one architectural decision that the user should weigh in on: **should sync write to `price_history` on every upsert?** Today, that table is populated by tests only. The decision is essentially "yes, the table is meant to be the source of truth for historical price observations" — but it is non-obvious and worth surfacing.

After user confirmation, the path is:

1. `sdd-propose` — produce the proposal and delta spec (the `sync-service` spec will need an additive requirement).
2. `sdd-spec` — write the delta specs for `sync-service` (new req: write to `price_history`) and a new top-level spec `price-drop-notifications` (or fold into `sync-service`).
3. `sdd-design` — design the detector function, the cooldown table, the dispatcher factory, the toast UI.
4. `sdd-tasks` — chained PRs (PR 1 data layer, PR 2 wiring).
5. `sdd-apply` — strict TDD, red-green-refactor.
6. `sdd-verify` — full `cargo test` + frontend manual smoke.
7. `sdd-archive` — sync delta specs into main, move change to `archive/`.

**The biggest open question to surface to the user** is item 1 above (PR strategy) and the threshold defaults (item 2). Everything else can be a sensible default that the user overrides if they push back.

---

## Ready for Proposal

**Yes**, with the caveat that the orchestrator should ask the user about:

1. PR strategy: single (≤ 460 LoC) vs. chained (PR 1 ≤ 200, PR 2 ≤ 250).
2. Threshold defaults: 10% / $50 / 24h cooldown — accept or override.
3. `AppNotificationAlert` resolution: keep stub + command-layer bridge (recommended) vs. upgrade the stub to take `AppHandle`.

Once those are answered, `sdd-propose` can produce a tight proposal.
