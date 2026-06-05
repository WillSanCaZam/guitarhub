# DB Migration Runner Specification

## Purpose

Schema migration discovery, tracking, and application for the app's local SQLite database. Guarantees the on-disk schema matches the code the app was built with.

## Requirements

### Requirement: Discover `.sql` migration files

The system MUST scan a designated `migrations/` directory at startup and collect all `.sql` files sorted by numerical prefix (e.g., `001_init.sql`, `002_add_index.sql`).

#### Scenarios

| Case | Precondition | Action | Outcome |
|------|-------------|--------|---------|
| Happy | Directory has `001_init.sql`, `002_add_index.sql` | Scan | Returns `[001_init.sql, 002_add_index.sql]` in order |
| Empty dir | No files in `migrations/` | Scan | Returns empty list, no error |
| Non-numeric prefix | File `setup.sql` exists | Scan | Skips file, logs warning |

### Requirement: Track applied state in `schema_meta`

The system MUST read `db_version` from `schema_meta` to determine current schema version. Missing table or key MUST be treated as version `0`.

#### Scenarios

| Case | Precondition | Action | Outcome |
|------|-------------|--------|---------|
| Happy | `schema_meta` has `db_version = '2'` | Read version | Returns `2` |
| Fresh DB | No `schema_meta` table | Read version | Assumes `0`, applies all migrations |
| Corrupt value | `db_version = 'abc'` | Read version | Returns error, app must not start |

### Requirement: Apply unapplied migrations in order

The system MUST execute each unapplied `.sql` file sequentially. If migration `001` is applied but `003` exists without `002`, the runner MUST fail. Each migration MUST preserve all columns from the prior schema when recreating tables. When data is copied between tables during a migration, the system MUST use an explicit `INSERT INTO ... SELECT (col1, col2, ...)` column list (not `SELECT *`) to prevent column-count mismatches.

The SQL statement splitter MUST track `BEGIN`/`END` depth so that semicolons inside trigger bodies (e.g., `CREATE TRIGGER ... BEGIN ... ; ... END;`) do not split the enclosing statement. The splitter MUST also strip line comments (`-- ...`) before parsing so that semicolons in comment text are not treated as statement terminators.

#### Scenarios

| Case | Precondition | Action | Outcome |
|------|-------------|--------|---------|
| Fresh DB | Version `0`, migrations `001` and `002` exist | Apply | Both run, `db_version` set to `2` |
| Incremental | Version `1`, `002_add_index.sql` exists | Apply | Only `002` runs, `db_version` updated to `2` |
| Up-to-date | Version `2`, all migrations present | Check | No SQL executed, no error |
| Gap in sequence | Version `0`, only `002` exists | Apply | Error returned, no partial apply |
| SQL failure | `001_init.sql` has syntax error | Execute | Error returned, DB unchanged |
| Migration 002 preserves all columns | 17-column `products_meta` with data | Migration 002 applied | All 17 columns retained, no data lost |
| Migration 002 recreates FTS5 triggers | `products_meta` dropped and recreated | Migration completes | `products_fts_ai`, `products_fts_ad`, `products_fts_au` triggers exist on the new table |
| Migration 002 uses explicit INSERT list | Data copy step | `INSERT INTO ... SELECT` | Uses an explicit column list (not `SELECT *`) |
| Migration 006 aligns wishlist schema | 4-column wishlist (sku, added_at, price_at_add, notes) | Migration 006 applied | `wishlist` has 10 columns: `id` (INTEGER PRIMARY KEY AUTOINCREMENT), `sku`, `name`, `brand`, `price`, `currency`, `image_url`, `product_url`, `notes`, `added_at` |
| Migration 006 preserves wishlist data | Existing wishlist rows present | Migration 006 applied | Original `sku`, `added_at`, `notes` values retained; new columns default to NULL |
| Trigger with BEGIN/END body | SQL `CREATE TRIGGER ... BEGIN ... ; ... END;` | Split statements | Trigger statement kept whole, not split on the internal `;` |
| Line comment with semicolon | SQL contains `; -- end of stmt` | Split statements | Comment semicolon is ignored, statement is not split |

### Requirement: Log migration activity

The system MUST log each applied migration. The system MAY use the `tracing` crate.

#### Scenario: Success logged

- GIVEN migration `002_add_index.sql` is applied
- WHEN it succeeds
- THEN a `tracing::info!` records `"Applied migration 002_add_index.sql"`

### Requirement: Migration 002 MUST declare all 17 columns in `products_meta_new`

The `002_add_url_validation.sql` migration MUST define `products_meta_new` with all 17 columns matching the schema from `001_init.sql`: `sku`, `source_id`, `name`, `brand`, `model`, `category`, `subcategory`, `specs_json`, `price`, `currency`, `condition`, `availability`, `url`, `image_url`, `seller`, `location`, `synced_at`. The migration MUST also preserve the CHECK constraints on `url` and `image_url`.

#### Scenario: Column count matches migration 001

- GIVEN `001_init.sql` creates `products_meta` with 17 columns
- WHEN `002_add_url_validation.sql` creates `products_meta_new`
- THEN `products_meta_new` has exactly 17 columns

### Requirement: Migration 006 MUST align wishlist with `export_service` expectations

The `006_wishlist_schema.sql` migration MUST transform the `wishlist` table to match the 10-column schema expected by `export_service.rs`'s `WishlistRow` struct. The migration MUST change the primary key from `sku` to a new autoincrement `id` column. Existing data (sku, added_at, notes) MUST be preserved; new columns MUST default to NULL.

#### Scenario: Export succeeds after migration 006

- GIVEN migration 006 has been applied
- WHEN `export_data` is invoked
- THEN `SELECT * FROM wishlist` returns rows deserializable into `WishlistRow` without column-mismatch errors

#### Scenario: Existing wishlist data preserved

- GIVEN `wishlist` has rows with `(sku, added_at, price_at_add, notes)` before migration 006
- WHEN migration 006 is applied
- THEN existing rows retain their original values; new columns default to NULL

## Acceptance Criteria

| Criterion | How to verify |
|-----------|---------------|
| Fresh DB gets all migrations | Start with empty DB — all `.sql` applied, `db_version` matches count |
| Upgrade only new migrations | Set `db_version=1`, start — only `002` runs |
| Idle is no-op | Start up-to-date — zero SQL executed |
| Bad SQL fails cleanly | Inject `CREAT TABLE` — error, schema unchanged |
| Missing sequence fails | Remove `001`, start — error, no partial apply |

## Out of Scope

- Migration rollback or undo
- Python-side migrations (scraper has no local SQLite)
- Data migrations (DDL only)
- Visual migration progress UI
