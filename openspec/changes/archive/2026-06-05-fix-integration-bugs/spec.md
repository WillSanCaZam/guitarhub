# Spec: Fix Integration Bugs

## Summary

Delta specs for 4 integration bugs across 3 capabilities. All fixes align frontend, backend, and database on consistent data shapes.

## Domain: db-migration-runner

### MODIFIED: Apply unapplied migrations in order

Each migration MUST preserve all columns from the prior schema when recreating tables. Migration 002 MUST use explicit INSERT column lists (not `SELECT *`).

| Scenario | GIVEN | WHEN | THEN |
|----------|-------|------|------|
| Migration 002 preserves columns | 17-column `products_meta` with data | Migration 002 applied | All 17 columns retained, no data lost |
| Migration 002 recreates FTS5 triggers | Table dropped and recreated | Migration completes | `products_fts_ai`, `_ad`, `_au` triggers exist |
| Migration 002 explicit INSERT | Data copy step | INSERT INTO ... SELECT | Uses explicit column list |
| Migration 006 wishlist alignment | 4-column wishlist | Migration 006 applied | 10 columns matching `WishlistRow` struct |
| Migration 006 preserves data | Existing wishlist rows | Migration 006 applied | Original values retained, new columns NULL |

### ADDED: Migration 002 MUST declare all 17 columns

`002_add_url_validation.sql` MUST define `products_meta_new` with all 17 columns from `001_init.sql`.

### ADDED: Migration 006 MUST align wishlist with export_service

`006_wishlist_schema.sql` MUST produce a `wishlist` table matching `WishlistRow` (10 columns: id, sku, name, brand, price, currency, image_url, product_url, notes, added_at).

## Domain: search-service

### MODIFIED: search_products command and frontend alignment

Frontend MUST use Rust field names: `products` (not `items`), `offset`/`limit` (not `page`/`pageSize`), `price_min`/`price_max`/`source` (not `priceMin`/`priceMax`/`sourceId`).

| Scenario | GIVEN | WHEN | THEN |
|----------|-------|------|------|
| Snake_case filters | Search page loaded | User searches with filters | Frontend sends `price_min`, `price_max`, `source` |
| Read products array | Response received | Frontend processes result | Reads `res.products` not `res.items` |
| Pagination from offset | Page 2, size 20 | Invoke called | Sends `offset: 20, limit: 20` |
| Derive page from offset | Response has `offset: 20, limit: 20` | Frontend renders | Computes page = `(20/20)+1 = 2` |

## Domain: wu1-tauri-wiring

### MODIFIED: Tauri builder invoke handler

`generate_handler!` MUST include `delete_setting`.

| Scenario | GIVEN | WHEN | THEN |
|----------|-------|------|------|
| delete_setting callable | App running | `invoke("delete_setting", {key})` | Returns `Ok(())` |
| All settings registered | App starts | Handler initialized | `get_setting`, `save_setting`, `delete_setting` present |

## Out of Scope

- `ProductDetail` component cleanup (dead code — separate change)
- Wishlist CRUD commands/UI (feature gap — separate change)
- Corrupted DB recovery for users who ran broken migration 002 (deferred)

## Acceptance Criteria

| Criterion | Verification |
|-----------|-------------|
| `make test` passes | All migration-runner and search tests green |
| Fresh DB: 17-column products_meta + FTS5 triggers | Run all migrations, inspect schema |
| Search IPC renders without field errors | Frontend displays search results |
| Wishlist export: 10-column CSV | `export_data` succeeds on migrated DB |
| `delete_setting` callable | `invoke("delete_setting")` returns Ok |
