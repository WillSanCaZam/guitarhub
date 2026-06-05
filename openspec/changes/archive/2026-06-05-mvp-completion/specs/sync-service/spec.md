# Delta for sync-service

> **Change**: mvp-completion  
> **Status**: Modified — JSON file stub upgraded to remote catalog download + upsert state machine

## ADDED Requirements

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

## MODIFIED Requirements

### Requirement: SyncService trait MUST be defined

The system MUST define a `SyncService` trait with method `async fn sync_catalog(&self, url: &str) -> Result<SyncResult, AppError>`. `SyncResult` MUST contain `products_loaded: u32`, `products_updated: u32`, `state: SyncState`, and `progress: f32`.
(Previously: Trait had `sync_from_json(path: &str)` with only counts)

| Case | Precondition | Action | Outcome |
|------|-------------|--------|---------|
| Trait compiles | Trait with `sync_catalog` defined | `cargo build` | Compiles successfully |
| Real catalog upsert | Valid URL with 50 products | `sync_catalog(url)` | 50 rows in `products_meta`; `loaded: 50` |

### Requirement: sync_catalog Tauri command MUST accept URL

`#[tauri::command] sync_catalog(url: String, state: State<'_, AppState>) -> Result<SyncResult, AppError>` SHALL replace the previous path-based command.
(Previously: Command took `path: String`)

| Case | Precondition | Action | Outcome |
|------|-------------|--------|---------|
| IPC with URL | Command registered | `invoke('sync_catalog', { url })` | Real sync runs, `SyncResult` returned |
| Missing URL arg | No `url` provided | Frontend calls without url | Tauri returns deserialization error |

## REMOVED Requirements

### Requirement: JsonFixtureLoader MUST upsert products

(Reason: Replaced by remote `CatalogDownloader` that fetches from GitHub Pages URL. Fixture-based loading is no longer the primary sync path.)
