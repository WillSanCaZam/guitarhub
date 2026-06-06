# Archive Report: add-price-drop-notifications

> **Change**: `add-price-drop-notifications`
> **Date**: 2026-06-05
> **Phase**: sdd-archive
> **Mode**: openspec (file-based)

## Change Summary

This change added price drop notifications to GuitarHub. It closes the loop on the existing alerting infrastructure (`AlertDispatcher` trait, `NtfyAlert`, `WebhookAlert`, `AppNotificationAlert`) by detecting price drops during catalog sync, applying anti-spam gates, and dispatching notifications via the user's configured channel.

Key behaviors:
- Every sync writes each upserted price to `price_history` (previously only written by tests).
- A pure `is_price_drop` function detects drops of >=10% OR >=$50.
- Three-layer anti-spam: first-observation suppression (no baseline = no drop), materiality threshold, and a 24-hour per-SKU cooldown via `price_drop_notifications` table.
- `sync_command` dispatches drops via `ntfy`, `webhook`, or `app` channel, records cooldown on success, and retries on failure.
- Frontend displays a toast: "X price drops, Y sent" after each sync.

Delivery was split into two stacked PRs (both target `main`) to stay under the 400-line review budget:
- **PR 1**: Data layer — migration 007, price history accessors, `price_drop_notifications` repo, and `is_price_drop` pure function with 10 unit tests.
- **PR 2**: Wiring — sync integration, command dispatch, frontend toast, and end-to-end tests.

## Final Commits

### PR 1 — Data Layer
1. `ef6d6ea` chore(db): add migration 007 for price drop notification cooldown
2. `1bbd13f` feat(price-drop): add price history accessors and drop detection
3. `af725c2` Merge PR 1: price-drop-notifications data layer

### PR 2 — Wiring
4. `eefe94d` feat(sync): wire price drop dispatch into sync_command
5. `d93557e` feat(frontend): add sync toast for price drop notifications
6. `5e9b629` Merge PR 2: price-drop-notifications wiring

## Test Status

| Layer | Count | Status |
|-------|-------|--------|
| Unit tests (`is_price_drop`, repo helpers) | ~13 | PASS |
| Integration tests (sync wiring, cooldown, dispatch) | ~15 | PASS |
| Frontend manual smoke | 1 | PASS |
| **Total** | **258/258** | **PASS** |

- `clippy` clean (zero warnings)
- No critical issues in verification

## Capabilities Added / Modified

| Capability | Action | Details |
|------------|--------|---------|
| `price-drop-notifications` | **Created** | New top-level spec covering detection algorithm, cooldown rules, dispatcher integration, anti-spam guarantees, and toast counter. 3 requirements, 11 scenarios. |
| `sync-service` | **Updated** | Delta merged into main spec. 5 new requirements added: record price history, carry `SyncResult.drops`, dispatch drops, failure isolation, and frontend toast reporting. 8 new scenarios. No existing requirements modified or removed. |

## Files Changed (Implementation)

- `src-tauri/src/repository/sqlite/migrations/007_price_drop_notifications.sql` (new)
- `src-tauri/src/repository/price_drop_notifications.rs` (new)
- `src-tauri/src/repository/price_history.rs` (modified)
- `src-tauri/src/repository/mod.rs` (modified)
- `src-tauri/src/services/price_drop.rs` (new)
- `src-tauri/src/services/sync.rs` (modified)
- `src-tauri/src/commands/sync_command.rs` (rewritten)
- `src/lib/stores/sync.ts` (new)
- `src/routes/+layout.svelte` (modified)
- `src/routes/+page.svelte` (modified)

## Artifacts Archived

All SDD artifacts preserved in `openspec/changes/archive/2026-06-05-add-price-drop-notifications/`:

| Artifact | Status |
|----------|--------|
| `exploration.md` | Preserved |
| `proposal.md` | Preserved |
| `spec.md` | Preserved |
| `specs/price-drop-notifications/spec.md` | Synced to main specs |
| `specs/sync-service/spec.md` | Synced to main specs |
| `design.md` | Preserved |
| `tasks.md` | Preserved (7/7 tasks complete, 2.1-2.4 completed during apply) |

## Source of Truth Updated

- `openspec/specs/price-drop-notifications/spec.md` — new capability spec
- `openspec/specs/sync-service/spec.md` — updated with price-drop wiring requirements

## Follow-up Items

1. **User-tunable threshold UI**: Settings keys (`drop_threshold_pct`, `drop_threshold_abs`, `cooldown_hours`) are read but not exposed in `Settings.svelte`. Future change: add sliders/inputs.
2. **Notifications history/log view**: Deferred to v2. The `price_drop_notifications` table is queryable for this.
3. **Background scheduler**: Currently only fires on manual sync. A future change could add a cron or background task.
4. **Per-SKU opt-out**: The `price_drop_notifications` schema can accept an `opted_out` column without a new migration.
5. **Price history retention**: `price_history` grows unbounded with the new writer. A future retention / pruning job is recommended.

## SDD Cycle Complete

The change has been fully planned, implemented, verified, and archived.

**Status**: ARCHIVED
**Next**: Ready for the next change.
