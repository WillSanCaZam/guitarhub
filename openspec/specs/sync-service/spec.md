# Sync Service Specification

> **Status**: Stable  
> **Change**: mvp-completion (upgraded from stub to remote catalog sync)

## Purpose

Provide a remote catalog download service with a full sync state machine, SQLite upsert, and Tauri IPC command. The service fetches a `CatalogFile` JSON from a remote URL (GitHub Pages), transitions through lifecycle states, and upserts products into `products_meta`.

## Requirements

### Requirement: SyncService trait MUST be defined

The system MUST define a `SyncService` trait with method `async fn sync_catalog(&self, url: &str) -> Result<SyncResult, AppError>`. `SyncResult` MUST contain `products_loaded: u32`, `products_updated: u32`, `state: SyncState`, and `progress: f32`.

| Case | Precondition | Action | Outcome |
|------|-------------|--------|---------|
| Trait compiles | Trait with `sync_catalog` defined | `cargo build` | Compiles successfully |
| Real catalog upsert | Valid URL with 50 products | `sync_catalog(url)` | 50 rows in `products_meta`; `loaded: 50` |

### Requirement: sync_catalog Tauri command MUST accept URL

`#[tauri::command] sync_catalog(url: String, state: State<'_, AppState>) -> Result<SyncResult, AppError>` SHALL replace the previous path-based command.

| Case | Precondition | Action | Outcome |
|------|-------------|--------|---------|
| IPC with URL | Command registered | `invoke('sync_catalog', { url })` | Real sync runs, `SyncResult` returned |
| Missing URL arg | No `url` provided | Frontend calls without url | Tauri returns deserialization error |

### Requirement: Sync state machine MUST track lifecycle

The `sync_state` table MUST transition through `idle → downloading → validating → sanitizing → inserting → done | failed`. Each transition SHALL update `started_at`, `completed_at`, and `error_message` on failure.

| Case | Precondition | Action | Outcome |
|------|-------------|--------|---------|
| Full lifecycle succeeds | sync_state is `idle` | Catalog sync runs successfully | Transitions through all states, ends at `done` |
| Network failure mid-sync | sync_state is `downloading` | HTTP fetch fails | sync_state set to `failed` with error message |
| Concurrent sync rejected | sync_state is `downloading` | New sync request arrives | Returns `AppError::SyncInProgress` |

### Requirement: SyncResult MUST include progress and state

`SyncResult` MUST add `progress: f32` (0.0–1.0) and `state: SyncState` fields.

| Case | Precondition | Action | Outcome |
|------|-------------|--------|---------|
| Partial progress | 100 products to insert | 50 inserted | `progress` SHALL be ~0.5 |
| State reported | Sync in `validating` phase | Check `SyncResult` | `state` SHALL be `validating` |

### Requirement: upsert_products MUST record price history

`CatalogSyncService::upsert_products` MUST, for every successfully upserted product, write a row to `price_history` with `(sku, price, recorded_at = now)`. The write MUST occur after the `products_meta` INSERT and MUST NOT alter the return type of the helper called by the state machine.

| Case | Precondition | Action | Outcome |
|------|-------------|--------|---------|
| First sync writes history | `price_history` is empty | `upsert_products` with 3 products | 3 rows with `recorded_at = now` |
| Second sync appends | One row for SKU `X` at `100.0` | Second sync ingests `X` at `100.0` | Second row appended |

### Requirement: SyncResult MUST carry detected drops

`SyncResult` MUST add a `drops: Vec<PriceDrop>` field. The field MUST be empty when no drop fires and MUST be populated by `upsert_products` from the in-pass drop evaluation.

| Case | Precondition | Action | Outcome |
|------|-------------|--------|---------|
| Drops surfaced | 2 products, 1 drops 15% | Sync completes | `SyncResult.drops` has 1 entry |
| No drops on first sync | `price_history` empty | First sync | `SyncResult.drops` is empty |

### Requirement: sync_command MUST dispatch detected drops

`sync_command` MUST, after `sync_catalog` returns, read `settings.alert_channel`, build the corresponding `AlertDispatcher`, and invoke `dispatcher.send(&drop)` for each entry in `SyncResult.drops`. The channel-to-dispatcher mapping MUST support at least `ntfy`, `webhook`, and `app`.

| Case | Precondition | Action | Outcome |
|------|-------------|--------|---------|
| Ntfy dispatch | `alert_channel = "ntfy"` | `sync_command` runs | `NtfyAlert::send` invoked once |
| Webhook dispatch | `alert_channel = "webhook"` | `sync_command` runs | `WebhookAlert::send` invoked once |

### Requirement: Dispatch failures MUST NOT block sync

`sync_command` MUST treat any `Err` from `dispatcher.send` as non-fatal: log and continue. It MUST still return `Ok(SyncResult)` to the frontend.

| Case | Precondition | Action | Outcome |
|------|-------------|--------|---------|
| Partial failure | 3 drops, drop 2 fails | `sync_command` runs | Drops 1 and 3 attempted; `Ok(SyncResult)` returned |

### Requirement: Frontend toast reports drops and sent counts

After a successful `sync_catalog`, the frontend MUST display a toast: `"X price drops, Y sent"` where `X = drops.length` and `Y = successful sends`.

| Case | Precondition | Action | Outcome |
|------|-------------|--------|---------|
| All succeed | `drops = 3`, all `Ok` | Frontend receives result | Toast reads `"3 price drops, 3 sent"` |
| Partial failure | `drops = 3`, 1 `Err` | Frontend receives result | Toast reads `"3 price drops, 2 sent"` |
