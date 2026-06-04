# Sync Service Specification

> **Status**: New capability (stub)  
> **Change**: fix-critical-fallas

## Purpose

Provide a contract and stub implementation for loading product catalog data from JSON fixtures into the SQLite database, enabling E2E data flow for demo and test scenarios.

## Requirements

### Requirement: SyncService trait MUST be defined

The system MUST define a `SyncService` trait in `src-tauri/src/services/sync.rs` with at least one method `async fn sync_from_json(&self, path: &str) -> Result<SyncResult, AppError>`. `SyncResult` MUST contain `products_loaded: u32` and `products_updated: u32`.

#### Scenarios

| Case | Precondition | Action | Outcome |
|------|-------------|--------|---------|
| Trait compiles | `SyncService` defined with `sync_from_json` | `cargo build` | Compilation succeeds |
| SyncResult returned | Valid JSON fixture | `sync_from_json(path)` | Returns `SyncResult` with counts |

### Requirement: JsonFixtureLoader MUST upsert products

The system MUST provide `JsonFixtureLoader` implementing `SyncService`. It MUST parse a `CatalogFile` JSON schema (`{ products: Vec<CatalogProduct> }`) and upsert each product into `products_meta`. A test fixture JSON with at least 2 sample products MUST exist.

#### Scenarios

| Case | Precondition | Action | Outcome |
|------|-------------|--------|---------|
| Empty DB load | No products in `products_meta` | `sync_from_json(valid_fixture)` | 2 rows inserted; `sync_state` updated |
| Duplicate upsert | Product "P001" exists at price 100.0 | Load fixture with same SKU, price 150.0 | 1 row for "P001"; price updated to 150.0 |
| Malformed JSON | Fixture file has invalid JSON | `sync_from_json(path)` | Returns `AppError::InvalidInput` |

### Requirement: sync_catalog Tauri command MUST exist

The system MUST provide `#[tauri::command] sync_catalog(path: String, state: State<'_, AppState>) -> Result<SyncResult, AppError>` registered in `main.rs` invoke handler.

#### Scenarios

| Case | Precondition | Action | Outcome |
|------|-------------|--------|---------|
| IPC trigger | `sync_catalog` registered | Frontend `invoke('sync_catalog', { path })` | SQLite upserts, `SyncResult` returned |
