# Soft-Delete Specification

> **Status**: New capability
> **Change**: add-soft-delete-products

## Purpose

Schema columns, sync-layer diff logic, and search filtering to detect delisted products and exclude them by default — preventing stale-data accumulation.

## Requirements

### Requirement: Migration 011 MUST add delisting columns

Migration `011_soft_delete.sql` MUST add `is_active INTEGER DEFAULT 1` and `delisted_at INTEGER` (nullable, unix epoch) to `products_meta`. The down migration MUST drop both columns. Existing rows inherit `is_active=1, delisted_at=NULL` — no data migration needed.

| Case | Precondition | Action | Outcome |
|------|-------------|--------|---------|
| Migration applies | `products_meta` exists | Run migration 011 | Both columns added, existing rows default to `is_active=1` |
| Down migration | Columns exist | Run down migration | Both columns removed with no data loss |
| New product inserted | Schema has columns | `INSERT OR REPLACE` | Defaults to `is_active=1, delisted_at=NULL` |

### Requirement: Sync diff pass MUST soft-delete absent SKUs

After `upsert_products`, `sync_catalog` MUST detect delisted SKUs per source: currently-active SKUs absent from the current batch SHALL be marked `is_active=0, delisted_at=now()`. The implementation MUST use a session-scoped temp table for current batch SKUs. Already-delisted SKUs MUST NOT be re-touched.

| Case | Precondition | Action | Outcome |
|------|-------------|--------|---------|
| SKU absent from batch | SKU-001 is `is_active=1`, not in current batch | Sync diff pass | SKU-001 becomes `is_active=0, delisted_at` set |
| SKU still present | SKU-001 `is_active=1`, in current batch | Sync diff pass | SKU-001 stays `is_active=1, delisted_at=NULL` |
| Already delisted SKU | SKU-001 `is_active=0`, not in batch | Sync diff pass | SKU-001 unchanged, `delisted_at` preserved |
| Different source | SKU-001 absent from Reverb batch, present in GC batch | Reverb sync | Only Reverb-source rows affected |

### Requirement: Reappearing SKU MUST reactivate

When a delisted SKU reappears in a future batch, `upsert_products` MUST reset it to `is_active=1, delisted_at=NULL`, regardless of how many cycles it was absent.

| Case | Precondition | Action | Outcome |
|------|-------------|--------|---------|
| Delisted SKU returns | SKU-001 `is_active=0, delisted_at=1000` | Sync batch includes SKU-001 | SKU-001 becomes `is_active=1, delisted_at=NULL` |
| Never-delisted SKU | SKU-001 `is_active=1` | Sync batch includes SKU-001 | No change to `is_active` or `delisted_at` |

### Requirement: SyncResult MUST report delisted count

`SyncResult` MUST add a `delisted: u32` field populated by the diff pass. The Tauri command and frontend receive this in the response.

| Case | Precondition | Action | Outcome |
|------|-------------|--------|---------|
| Some delisted | 3 SKUs soft-deleted | Sync completes | `SyncResult.delisted = 3` |
| None delisted | All current SKUs in batch | Sync completes | `SyncResult.delisted = 0` |

### Requirement: Search MUST exclude inactive by default

All search queries MUST append `AND is_active = 1`. The `search_products` command MUST accept `include_inactive: Option<bool>` (default `false`); when `true`, the filter is omitted.

| Case | Precondition | Action | Outcome |
|------|-------------|--------|---------|
| Default search | Inactive products exist | `search_products("guitar")` | No inactive products returned |
| Include inactive | Inactive products exist | `search_products("guitar", { include_inactive: true })` | Inactive products included |
| Admin view | User enables "show delisted" | Frontend sets flag | All products searchable |

### Requirement: is_active MUST be orthogonal to availability

`is_active` MUST NOT affect `availability` (`in_stock`/`out_of_stock`/`unknown`). A product MAY be active but out of stock or inactive but was in stock. No sync logic SHALL infer one from the other.

| Case | Precondition | Action | Outcome |
|------|-------------|--------|---------|
| Active but out of stock | Product `is_active=1, availability=out_of_stock` | Search | Included in default search |
| Inactive but was in stock | Product `is_active=0, availability=in_stock` | Search | Excluded from default search; `availability` preserved |
| Sync updates availability | Product `is_active=1`, batch says out_of_stock | Sync | Only `availability` changes, `is_active` untouched |
