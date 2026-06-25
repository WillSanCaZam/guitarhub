# Tasks: Add Soft-Delete for Delisted Products

## Review Workload Forecast

| Field | Value |
|-------|-------|
| Estimated changed lines | ~150–180 |
| 400-line budget risk | Low |
| Chained PRs recommended | No |
| Suggested split | Single PR |
| Delivery strategy | single-pr-default |
| Chain strategy | pending |

Decision needed before apply: No
Chained PRs recommended: No
Chain strategy: pending
400-line budget risk: Low

## Phase 1: Database Migration

- [x] 1.1 Create `src-tauri/src/repository/sqlite/migrations/011_soft_delete.sql` — `ALTER TABLE products_meta ADD COLUMN is_active INTEGER DEFAULT 1`, `ADD COLUMN delisted_at INTEGER`
- [x] 1.2 Create `src-tauri/src/repository/sqlite/migrations/011_soft_delete.down.sql` — `ALTER TABLE products_meta DROP COLUMN is_active`, `DROP COLUMN delisted_at`
- [x] 1.3 Update migration test helpers in `migrations/mod.rs` to chain through v011

## Phase 2: Sync Service — Delisting Logic

- [x] 2.1 Add `delisted: u32` to `SyncResult` in `src-tauri/src/services/sync.rs`
- [x] 2.2 Create temp-table helper in `sync.rs`: insert batch SKUs into session-scoped temp table
- [x] 2.3 Add Phase 4 in `upsert_products`: after batch upsert, run `UPDATE products_meta SET is_active=0, delisted_at=? WHERE source_id=? AND is_active=1 AND sku NOT IN (SELECT sku FROM temp_batch)`, return delisted count
- [x] 2.4 Update all 3 `SyncResult` construction sites (sync_catalog×2, sync_local_catalog) to include delisted count

## Phase 3: Search — is_active Filter

- [x] 3.1 Add `include_inactive: bool` (default `false`) to `SearchFilters` in `src-tauri/src/domain/product.rs`
- [x] 3.2 In `src-tauri/src/services/search.rs`, append `AND m.is_active = 1` to WHERE when `include_inactive` is false

## Phase 4: Testing

- [x] 4.1 Test: SKU present in current batch stays active after sync
- [x] 4.2 Test: SKU absent from current batch is marked `is_active=0, delisted_at` set
- [x] 4.3 Test: Already-delisted SKU unchanged by subsequent delisting pass
- [x] 4.4 Test: Re-listed SKU (previously delisted) gets `is_active=1, delisted_at=NULL`
- [x] 4.5 Test: Search excludes inactive products by default; `include_inactive=true` includes them
- [x] 4.6 Test: `SyncResult.delisted` reports correct count
