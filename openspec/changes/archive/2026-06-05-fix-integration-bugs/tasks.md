# Tasks: Fix Integration Bugs

## Review Workload Forecast

| Field | Value |
|-------|-------|
| Estimated changed lines | 320–370 |
| 400-line budget risk | Medium |
| Chained PRs recommended | Yes |
| Suggested split | PR 1 (migrations: runner + 002 + 006) → PR 2 (frontend + wiring + test alignment) |
| Delivery strategy | ask-always |
| Chain strategy | pending |

Decision needed before apply: Yes
Chained PRs recommended: Yes
Chain strategy: pending
400-line budget risk: Medium

### Suggested Work Units

| Unit | Goal | Likely PR | Notes |
|------|------|-----------|-------|
| 1 | Fix migration runner + rewrite 002 + add 006 | PR 1 | ~215 lines; foundation for all DB work; includes tests |
| 2 | Search field alignment + delete_setting + export test alignment | PR 2 | ~125 lines; independent of PR 1; frontend + wiring fixes |

## Phase 1: Migration Runner Fix (prerequisite)

- [x] 1.1 RED: Add test in `src-tauri/src/repository/sqlite/migrations/mod.rs` `#[cfg(test)]` that runs a migration SQL containing `CREATE TRIGGER ... BEGIN ... ; ... END;` and asserts the trigger is created without split errors
- [x] 1.2 GREEN: Replace `sql.split(';')` (line 89) with a `split_statements()` function that tracks `BEGIN`/`END` depth — only split on `;` when depth is 0 (~15 lines)
- [x] 1.3 Add test: verify plain semicolon-separated statements (no BEGIN/END) still split correctly (regression guard)

## Phase 2: Migration 002 Rewrite (depends on Phase 1)

- [x] 2.1 RED: Add test in `mod.rs` that runs 001→002 on in-memory DB, asserts `PRAGMA table_info(products_meta)` returns 17 columns
- [x] 2.2 RED: Add test that inserts a product row after 001→002, then queries `products_fts` to verify FTS5 triggers fire
- [x] 2.3 GREEN: Rewrite `src-tauri/src/repository/sqlite/migrations/002_add_url_validation.sql` — define `products_meta_new` with all 17 columns from 001, explicit `INSERT INTO ... SELECT (col1, col2, ...)` list, recreate FTS5 virtual table + 3 triggers + 3 indexes (~60 lines SQL)
- [x] 2.4 Add corrupted-DB guard: at top of 002, log warning if `PRAGMA table_info` returns < 17 columns, then proceed with full rewrite

## Phase 3: Migration 006 Wishlist Schema (independent)

- [x] 3.1 RED: Add test in `mod.rs` that runs 001→...→006, asserts `PRAGMA table_info(wishlist)` returns 10 columns matching `WishlistRow` (id, sku, name, brand, price, currency, image_url, product_url, notes, added_at)
- [x] 3.2 RED: Add test that seeds wishlist rows before 006, runs migration, asserts original values preserved and new columns are NULL
- [x] 3.3 GREEN: Create `src-tauri/src/repository/sqlite/migrations/006_wishlist_schema.sql` — recreate-table pattern: `CREATE TABLE wishlist_v2` (10 cols), `INSERT INTO wishlist_v2 (sku, added_at, notes) SELECT ...`, `DROP TABLE wishlist`, `ALTER TABLE wishlist_v2 RENAME TO wishlist` (~20 lines)

## Phase 4: Search Field Alignment (independent)

- [x] 4.1 Create `src/lib/types/search.ts` with `SearchFilters`, `SearchResult`, and `SortOrder` TypeScript interfaces matching Rust serde output
- [x] 4.2 Update `src/routes/+page.svelte`: `res.items` → `res.products`, `page`/`pageSize` → compute from `offset`/`limit`, filter keys `priceMin`/`priceMax`/`sourceId` → `price_min`/`price_max`/`source` (~10 lines changed)

## Phase 5: Wiring Fix (independent)

- [x] 5.1 Add `guitarhub_lib::commands::settings_command::delete_setting` to `generate_handler!` in `src-tauri/src/main.rs` (1 line)

## Phase 6: Export Test Alignment (independent, benefits from Phase 1)

- [x] 6.1 Replace inline `test_pool()` in `src-tauri/src/services/export_service.rs` tests with `MigrationRunner` that applies real migrations 001→006 — removes ~30 lines of duplicated schema SQL
- [x] 6.2 Replace inline `test_pool()` in `src-tauri/src/commands/export_command.rs` tests with same migration-runner approach — removes ~30 lines of duplicated schema SQL
- [x] 6.3 Verify `make test` passes with all migration-runner, export, and search tests green
