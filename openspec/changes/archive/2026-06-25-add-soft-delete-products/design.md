# Design: Soft-Delete for Delisted Products

## Problem

The current sync layer does `INSERT OR REPLACE` with no diff logic. When a SKU disappears from a source catalog (item sells, delists, or is removed), it stays in `products_meta` indefinitely with no way to distinguish "still active" from "no longer listed". This means:

- Users see stale products that are no longer available.
- The UI has no way to filter or hide delisted items.
- Adding a second source (Guitar Center) without this foundation doubles the stale-data surface.

## Non-Goal

- This change does NOT modify any adapter or scraper code.
- This change does NOT modify `availability` semantics. `availability` (`in_stock`/`out_of_stock`/`unknown`) describes whether a currently-listed item has stock right now. That is orthogonal to whether the item is still listed at all.

## Technical Approach

Add two columns to `products_meta` and a diff pass in the Rust sync service that detects delisted SKUs and marks them as inactive.

### Schema change (migration 011)

```sql
ALTER TABLE products_meta ADD COLUMN is_active    INTEGER DEFAULT 1;
ALTER TABLE products_meta ADD COLUMN delisted_at  INTEGER;
```

- `is_active`: `1` (active, default), `0` (delisted — no longer present in the source).
- `delisted_at`: Unix epoch timestamp of when the sync first detected the SKU was absent. `NULL` while active.

Existing rows get `is_active = 1, delisted_at = NULL` — no data migration needed.

The FTS triggers (products_fts) do NOT need updating. FTS queries should filter by `is_active = 1` in their WHERE clause (handled in the search service, not in this change).

### Sync layer change

After `upsert_products` completes, add a delisting detection pass in `sync_catalog` (and `sync_local_catalog`):

```rust
// Phase 4: Soft-delete detection
// Mark as inactive any previously-active SKU from this source
// that was NOT present in the current sync batch.
let delisted = sqlx::query(
    "UPDATE products_meta
     SET is_active = 0, delisted_at = ?2
     WHERE source_id = ?1
       AND is_active = 1
       AND sku NOT IN (SELECT sku FROM current_batch)"
)
.bind(source_id)
.bind(synced_at)
.execute(&self.pool)
.await?;
```

The "current batch" SKUs need to be available for the NOT IN query. Options:

1. **Temp table**: Insert current batch SKUs into a temp table, then diff. Cleanest for large batches.
2. **IN list**: Pass SKUs as a parameter list. Works for moderate catalog sizes (< 10K SKUs).
3. **Track in sync_state**: Store the previous sync's SKU set in `sync_state` JSON column.

**Chosen approach**: Temp table. Insert current batch SKUs into a session-scoped temp table, run the diff UPDATE, then drop the temp table. This avoids blowing out the query size and keeps the diff logic self-contained within the transaction.

### SyncResult changes

Add a `delisted: u32` field to `SyncResult` so the caller (command layer, UI) knows how many items were marked inactive.

### Frontend / Search changes

- `search.rs` queries MUST add `AND is_active = 1` (or make it configurable via a filter param for admin/debug views).
- The `search` Tauri command should expose an optional `include_inactive: bool` parameter (default `false`).

## File Changes

| File | Action | Description |
|------|--------|------------|
| `src-tauri/src/repository/sqlite/migrations/011_soft_delete.down.sql` | Create | Revert: `ALTER TABLE products_meta DROP COLUMN is_active` and `DROP COLUMN delisted_at` |
| `src-tauri/src/repository/sqlite/migrations/011_soft_delete.sql` | Create | `ALTER TABLE products_meta ADD COLUMN is_active INTEGER DEFAULT 1`, `ADD COLUMN delisted_at INTEGER` |
| `src-tauri/src/repository/sqlite/migrations/mod.rs` | Modify | Add migration 011 |
| `src-tauri/src/services/sync.rs` | Modify | Add Phase 4 soft-delete diff pass in `sync_catalog` and `sync_local_catalog`. Add `delisted` to `SyncResult`. |
| `src-tauri/src/services/search.rs` | Modify | Add `AND is_active = 1` to default queries. Add `include_inactive` filter parameter. |
| `src-tauri/src/repository/product.rs` | Modify | (Optional) Add `batch_delete_inactive` method if temp-table approach is abstracted. |

## Dependencies

- **None.** This change is self-contained and does not depend on any adapter changes.

Products from Reverb (the only current source) will also get soft-delete treatment on the next sync after this migration is applied. That is correct — any item that has already sold and disappeared from Reverb's API will be marked inactive on first sync.

## Rollout

```
PR: add-soft-delete-products  (migration + sync logic + search filter)
     Single PR, ~200-350 changed lines.
```

No data migration script needed — existing rows default to `is_active = 1, delisted_at = NULL`.

Every subsequent sync of any source will automatically detect and mark delisted items.

## Testing Strategy

| Layer | What | Approach |
|-------|------|----------|
| Unit (Rust) | SKU present in current batch stays active | Sync a batch with SKU-001, then a second batch with same SKU → assert `is_active = 1` |
| Unit (Rust) | SKU absent from current batch gets delisted | Sync a batch with SKU-001, then a second batch WITHOUT SKU-001 → assert `is_active = 0, delisted_at` is set |
| Unit (Rust) | Already-delisted SKU stays delisted | Assert that re-syncing without a delisted SKU does not change `delisted_at` |
| Unit (Rust) | Re-listed SKU is reactivated | Sync a batch with SKU-001 (delisted in prior run), then SKU-001 appears again → assert `is_active = 1, delisted_at = NULL` |
| Integration | Search filters out inactive by default | Query search → assert no delisted products returned |
| Integration | Search includes inactive with flag | Query with `include_inactive=true` → assert delisted products returned |
