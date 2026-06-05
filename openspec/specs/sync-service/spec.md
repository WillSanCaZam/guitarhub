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
