# Design: Fix Integration Bugs

## Technical Approach

Four integration bugs stem from cross-layer contract violations: DB migrations drop columns, services assume schemas that don't exist, and the frontend reads fields the backend never sends. The fix strategy is **contract-first**: align each layer to the authoritative source (Rust domain types + migration 001 schema) and add integration tests that verify contracts against real migrations instead of inline schemas.

A fifth bug was discovered during design: the migration runner's `split(';')` breaks `CREATE TRIGGER` statements whose `BEGIN...END` bodies contain internal semicolons. Migration 001's FTS5 triggers likely never execute through the runner. This must be fixed before migration 002 can recreate triggers.

## Architecture Decisions

| Decision | Options | Tradeoff | Choice |
|----------|---------|----------|--------|
| Migration 002 rewrite strategy | (A) Rewrite in-place with all 17 cols (B) Add new migration 002b | A is simpler for pre-release; B safer for deployed DBs | **A** — pre-release, no deployed users |
| Migration runner split fix | (A) Track BEGIN/END depth (B) Use `sqlite3_exec` batch | A is 15 lines; B requires unsafe FFI | **A** — minimal, testable |
| Wishlist migration approach | (A) ALTER TABLE add columns (B) Recreate table | A can't change PK (sku→id); B supports full schema | **B** — PK change required by WishlistRow |
| Search field alignment | (A) Frontend adapts to Rust (B) Rust adapts to frontend | A: Rust is tested source-of-truth; B: breaks Rust tests | **A** — 10 lines in 1 file |
| TypeScript API types | (A) Add TS interfaces (B) Leave untyped | A prevents future drift; B is faster | **A** — prevents recurrence |
| Export test schema source | (A) Use migration runner in tests (B) Keep inline schemas | A catches drift; B is faster but masks bugs | **A** — the root cause of bug 3 |

## Data Flow

### Search (after fix)

```
Frontend (+page.svelte)          Tauri IPC              Rust Backend
─────────────────────────     ──────────────          ──────────────
invoke('search_products',
  { query, filters: {
      price_min, price_max,      ──serde──→      SearchFilters {
      source, category },                           price_min, price_max,
  sort, page, pageSize })                           source, category }
                                                     │
                                                     ▼
                                               FtsSearchService.search()
                                                     │
                                                     ▼
                                               SearchResult {
  results = res.products       ←──serde──        products, total,
  page = floor(offset/limit)+1                    offset, limit }
```

### Migration 002 (after fix)

```
Migration 001 (17 cols)  ──→  Migration 002 rewrite
  products_meta (17 cols)       products_meta_new (17 cols + CHECK)
  products_fts (FTS5)           INSERT INTO ... SELECT (explicit cols)
  triggers (3x)                 DROP old → RENAME new
                                Recreate FTS5 + triggers (3x)
                                Recreate indexes (3x)
```

## File Changes

| File | Action | Description |
|------|--------|-------------|
| `src-tauri/src/repository/sqlite/migrations/mod.rs` | Modify | Fix `split(';')` to track BEGIN/END depth for trigger bodies |
| `src-tauri/src/repository/sqlite/migrations/002_add_url_validation.sql` | Rewrite | All 17 columns, explicit INSERT...SELECT, FTS5 + triggers + indexes |
| `src-tauri/src/repository/sqlite/migrations/006_wishlist_schema.sql` | Create | Recreate wishlist with 10-column schema matching `WishlistRow` |
| `src/routes/+page.svelte` | Modify | `res.items`→`res.products`, derive page from offset/limit, snake_case filters |
| `src/lib/types/search.ts` | Create | TypeScript interfaces for `SearchResult`, `SearchFilters`, `SortOrder` |
| `src-tauri/src/main.rs` | Modify | Add `delete_setting` to `generate_handler!` |
| `src-tauri/src/services/export_service.rs` | Modify | Tests use migration runner instead of inline schema |
| `src-tauri/src/commands/export_command.rs` | Modify | Tests use migration runner instead of inline schema |

## Interfaces / Contracts

### Migration 002 — Explicit Column List

```sql
INSERT OR IGNORE INTO products_meta_new
  (sku, source_id, name, brand, model, category, subcategory, specs_json,
   price, currency, condition, availability, url, image_url, seller, location, synced_at)
SELECT
   sku, source_id, name, brand, model, category, subcategory, specs_json,
   price, currency, condition, availability, url, image_url, seller, location, synced_at
FROM products_meta;
```

### Migration 006 — Wishlist Schema

```sql
-- Recreate: PK changes from sku to id (autoincrement)
CREATE TABLE wishlist_new (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    sku         TEXT,
    name        TEXT,
    brand       TEXT,
    price       REAL,
    currency    TEXT,
    image_url   TEXT,
    product_url TEXT,
    notes       TEXT,
    added_at    INTEGER
);
-- Migrate existing data (only sku, added_at, notes survive; new cols get NULL)
INSERT INTO wishlist_new (sku, added_at, notes)
  SELECT sku, added_at, notes FROM wishlist;
DROP TABLE wishlist;
ALTER TABLE wishlist_new RENAME TO wishlist;
```

### TypeScript API Contract

```typescript
interface SearchFilters {
  category: string | null;
  price_min: number | null;
  price_max: number | null;
  source: string | null;
}
interface SearchResult {
  products: RawProduct[];
  total: number;
  offset: number;
  limit: number;
}
```

## Testing Strategy

| Layer | What | Approach |
|-------|------|----------|
| Unit | Migration runner BEGIN/END splitting | Test with trigger SQL containing internal semicolons |
| Unit | Migration 002 column preservation | Run 001→002 on in-memory DB, assert 17 columns via `PRAGMA table_info` |
| Unit | Migration 002 FTS triggers work | Insert row after migration, verify FTS index updated |
| Unit | Migration 006 wishlist schema | Run 001→...→006, assert 10 columns, verify data migration |
| Unit | Migration 002 corrupted DB detection | Simulate 11-column table (broken 002 already ran), verify detection |
| Integration | Export service against real migrations | Replace inline `test_pool()` with migration runner, verify `SELECT *` returns 10 cols |
| Integration | Full migration chain 001→006 | Run all migrations, verify final schema matches all service expectations |
| E2E | Search IPC returns renderable data | Existing WebDriver tests (weekly) — no new infra needed |

### Corrupted DB Detection (Migration 002)

For users who already ran the broken migration 002:

```sql
-- Detection: check if products_meta has fewer than 17 columns
-- PRAGMA table_info returns one row per column
-- If count < 17, log warning and attempt recovery
```

Add at the top of migration 002:
```sql
-- Guard: if products_meta exists with < 17 columns, it was corrupted
-- by a previous broken migration. Log and recreate from scratch.
```

Since this is pre-release, the guard logs a warning and proceeds with the full rewrite. The `DROP TABLE` in the rewrite cleans up the corrupted table.

## Migration / Rollout

**Pre-release**: No deployed users. All fixes apply cleanly on fresh DB.

**Migration runner fix** (BEGIN/END splitting) is a prerequisite for migration 002's trigger recreation. Must be implemented first.

**Fix order** (dependency graph):

```
migration_runner_fix ──→ migration_002_rewrite
                              │
migration_006 (independent)   │
                              │
search_fields (independent)   │
                              │
delete_setting (independent)  │
                              ▼
                    export_test_alignment (uses runner)
```

Parallelizable: migration_006, search_fields, delete_setting can all be done concurrently with the migration_002 track.

## Security Implications

No new attack surface. Migration 002's URL CHECK constraints (`LIKE 'https://%'`) are security hardening — the rewrite preserves them. The migration runner fix is purely internal parsing logic.

## Open Questions

- [ ] Should migration 002 include a `PRAGMA table_info` check to detect and warn about already-corrupted DBs, or just silently fix them? (Recommendation: log warning + fix)
- [ ] Should the TypeScript interfaces live in `src/lib/types/search.ts` or be co-located with the component? (Recommendation: `src/lib/types/` for reusability)
