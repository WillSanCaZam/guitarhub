# Delta for db-migration-runner

## MODIFIED Requirements

### Requirement: Apply unapplied migrations in order

The system MUST execute each unapplied `.sql` file sequentially. If migration `001` is applied but `003` exists without `002`, the runner MUST fail. Each migration MUST preserve all columns from the prior schema when recreating tables.

(Previously: Required sequential execution but did not mandate column preservation during table recreation.)

#### Scenario: Fresh DB gets all migrations

- GIVEN an empty database
- WHEN migrations 001 through 006 are applied
- THEN `products_meta` has 17 columns, FTS5 triggers exist, and `wishlist` has 10 columns

#### Scenario: Migration 002 preserves all product columns

- GIVEN migration 001 has been applied (17-column `products_meta` with data)
- WHEN migration 002 is applied
- THEN `products_meta` retains all 17 columns (sku, source_id, name, brand, model, category, subcategory, specs_json, price, currency, condition, availability, url, image_url, seller, location, synced_at)
- AND no product data is lost

#### Scenario: Migration 002 recreates FTS5 triggers

- GIVEN migration 002 drops and recreates `products_meta`
- WHEN the migration completes
- THEN triggers `products_fts_ai`, `products_fts_ad`, and `products_fts_au` exist on the new `products_meta` table
- AND FTS5 search returns results for indexed product data

#### Scenario: Migration 002 uses explicit INSERT column list

- GIVEN migration 002 copies data from old to new table
- WHEN the INSERT INTO ... SELECT executes
- THEN it uses an explicit column list (not `SELECT *`) to prevent column-count mismatch

#### Scenario: Migration 006 aligns wishlist schema

- GIVEN migrations 001-005 have been applied (wishlist has 4 columns: sku, added_at, price_at_add, notes)
- WHEN migration 006 is applied
- THEN `wishlist` has 10 columns: id (INTEGER PRIMARY KEY AUTOINCREMENT), sku, name, brand, price, currency, image_url, product_url, notes, added_at
- AND existing wishlist rows are preserved with NULL defaults for new columns

## ADDED Requirements

### Requirement: Migration 002 MUST declare all 17 columns in products_meta_new

The `002_add_url_validation.sql` migration MUST define `products_meta_new` with all 17 columns matching the schema from `001_init.sql`, plus the CHECK constraints on `url` and `image_url`.

#### Scenario: Column count matches migration 001

- GIVEN `001_init.sql` creates `products_meta` with 17 columns
- WHEN `002_add_url_validation.sql` creates `products_meta_new`
- THEN `products_meta_new` has exactly 17 columns

### Requirement: Migration 006 MUST align wishlist with export_service expectations

The `006_wishlist_schema.sql` migration MUST transform the `wishlist` table to match the 10-column schema expected by `export_service.rs` `WishlistRow` struct.

#### Scenario: Export succeeds after migration 006

- GIVEN migration 006 has been applied
- WHEN `export_data` is invoked
- THEN `SELECT * FROM wishlist` returns rows deserializable into `WishlistRow` without column mismatch errors

#### Scenario: Existing wishlist data preserved

- GIVEN wishlist has rows with (sku, added_at, price_at_add, notes) before migration 006
- WHEN migration 006 is applied
- THEN existing rows retain their original values; new columns default to NULL
