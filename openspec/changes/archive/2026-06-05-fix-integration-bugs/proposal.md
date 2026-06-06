# Proposal: Fix Integration Bugs

## Intent

Three integration bugs cause runtime failures where frontend, backend, and database disagree on data shapes. Migration 002 silently destroys product data, the search frontend can't parse backend responses, and wishlist export crashes on schema mismatch. A fourth trivial gap leaves `delete_setting` unreachable. This change fixes all four to restore a working integration path.

## Scope

### In Scope
- Rewrite migration 002 to preserve all 17 columns and recreate FTS5 triggers
- Add migration 006 to align wishlist table with `export_service.rs` expectations
- Update `+page.svelte` search fields to match Rust `SearchResult`/`SearchFilters`
- Register `delete_setting` in `generate_handler!` macro in `main.rs`

### Out of Scope
- `ProductDetail` component cleanup (dead code — separate change)
- Wishlist CRUD commands/UI (feature gap — separate change)
- Corrupted DB recovery tooling for users who ran broken migration 002 (flag risk, defer implementation)

## Capabilities

### New Capabilities
None

### Modified Capabilities
- `search-service`: Frontend must use Rust field names (`products`, `offset`, `price_min`, `price_max`, `source`)
- `db-migration-runner`: Migration 002 rewritten with full column set + FTS5 trigger recreation; new migration 006 for wishlist schema
- `wu1-tauri-wiring`: `delete_setting` added to invoke handler

## Approach

Fix in dependency order:

1. **Migration 002** (critical): Rewrite `002_add_url_validation.sql` — include all 17 columns in `products_meta_new`, use explicit column list in `INSERT INTO ... SELECT`, recreate FTS5 triggers after table rename
2. **Migration 006** (wishlist): New `006_wishlist_schema.sql` — `ALTER TABLE` or recreate pattern to add `id, name, brand, price, currency, image_url, product_url` columns. Align `export_command.rs` tests to use migration runner
3. **Search fields**: Update `+page.svelte` — `res.items` → `res.products`, derive page from `offset/limit`, rename filter keys to snake_case
4. **delete_setting**: Add to `generate_handler!` in `main.rs`

## Affected Areas

| Area | Impact | Description |
|------|--------|-------------|
| `src-tauri/src/repository/sqlite/migrations/002_add_url_validation.sql` | Modified | Rewrite with all 17 columns + FTS5 triggers |
| `src-tauri/src/repository/sqlite/migrations/006_wishlist_schema.sql` | New | Wishlist table schema alignment |
| `src/routes/+page.svelte` | Modified | Search field names to match Rust API |
| `src-tauri/src/main.rs` | Modified | Register `delete_setting` command |
| `src-tauri/src/commands/export_command.rs` | Modified | Tests use migration runner instead of inline schema |

## Risks

| Risk | Likelihood | Mitigation |
|------|------------|------------|
| Users with corrupted DBs from broken migration 002 | Low (pre-release) | Detection query in migration 002 to check column count; log warning with recovery instructions |
| SQLite ALTER TABLE limitations for wishlist | Med | Use recreate-table pattern if ALTER TABLE can't add all columns |
| Migration 002 rewrite changes already-applied migration hash | Low (pre-release) | Migration runner uses sequential versioning, not content hashing |

## Rollback Plan

Each fix is isolated to its own file. Revert by:
1. Restore original `002_add_url_validation.sql` from git history
2. Delete `006_wishlist_schema.sql`
3. Revert `+page.svelte` field names
4. Remove `delete_setting` from `generate_handler!`

No cross-file entanglement — each fix can be reverted independently.

## Dependencies

- None external. All fixes use existing stack (SQLite, Tauri IPC, Svelte).

## Success Criteria

- [ ] `make test` passes with all migration-runner and search tests green
- [ ] Fresh DB migration produces `products_meta` with 17 columns and working FTS5 triggers
- [ ] `search_products` IPC returns data the frontend renders without field errors
- [ ] Wishlist export produces CSV with all 10 columns from migrated schema
- [ ] `delete_setting` is callable from frontend via `invoke()`
