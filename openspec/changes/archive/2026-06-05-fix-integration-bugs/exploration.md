## Exploration: fix-integration-bugs

### Current State

GuitarHub has three confirmed integration bugs where different parts of the codebase disagree on data shapes. Additionally, three smaller gaps exist (unregistered command, unused component, incomplete wishlist feature). All bugs are in code that has tests — but the tests use their own inline schemas, masking the real migration/runtime mismatches.

---

### Bug 1: Frontend-backend field mismatch in search — CONFIRMED

**Root cause**: The Svelte frontend (`+page.svelte`) was written against a different API contract than what the Rust backend actually serializes.

**Field mismatches on the response** (Rust `SearchResult` → frontend):

| Rust field | Frontend expects | Match? |
|------------|------------------|--------|
| `products` | `res.items` | NO |
| `total` | `res.total` | YES |
| `offset` | `res.page` | NO (different concept) |
| `limit` | (not used) | — |

**Field mismatches on the filter input** (frontend → Rust `SearchFilters`):

| Frontend sends | Rust expects | Match? |
|----------------|--------------|--------|
| `priceMin` | `price_min` | NO (camelCase vs snake_case) |
| `priceMax` | `price_max` | NO |
| `sourceId` | `source` | NO (different name entirely) |
| `category` | `category` | YES |

**Affected files**:
- `src/routes/+page.svelte` — lines 27, 33-35 (the only consumer)
- `src-tauri/src/domain/product.rs` — `SearchResult` (line 99), `SearchFilters` (line 75)

**Recommended fix**: Update the frontend to match the Rust API. Rust is the source of truth — it has comprehensive tests and is the data owner.
- `res.items` → `res.products`
- `res.page` → derive from `Math.floor(res.offset / res.limit) + 1`
- `filters.priceMin` → `filters.price_min`
- `filters.priceMax` → `filters.price_max`
- `filters.sourceId` → `filters.source`

**Effort**: Low (~10 lines changed in 1 file)

---

### Bug 2: Migration 002 drops 6 columns from products_meta — CONFIRMED (CRITICAL)

**Root cause**: Migration 002 recreates `products_meta` to add URL CHECK constraints but omits 6 columns from the new table definition.

**Migration 001 columns** (17 total):
`sku, source_id, name, brand, model, category, subcategory, specs_json, price, currency, condition, availability, url, image_url, seller, location, synced_at`

**Migration 002 `products_meta_new` columns** (11 total):
`sku, source_id, price, currency, condition, availability, url, image_url, seller, location, synced_at`

**Missing**: `name, brand, model, category, subcategory, specs_json`

**Three cascading failures**:
1. `INSERT OR IGNORE INTO products_meta_new SELECT * FROM products_meta` — column count mismatch (17 vs 11), SQL error
2. `DROP TABLE products_meta` — destroys all product data even though the copy failed
3. FTS5 triggers (from migration 001) reference `new.name`, `new.brand`, etc. — columns that no longer exist. Any subsequent INSERT/UPDATE on products_meta triggers a SQL error

**Additional issue**: Migration 002 does NOT recreate the FTS5 triggers after the table swap. Even if columns were correct, the triggers from migration 001 are dropped with the old table.

**Affected files**:
- `src-tauri/src/repository/sqlite/migrations/002_add_url_validation.sql` — the buggy migration
- `src-tauri/src/repository/sqlite/migrations/001_init.sql` — original schema + FTS triggers
- `src-tauri/src/services/search.rs` — search depends on these columns and FTS

**Impact**:
- **Fresh installs**: Migration 001 runs fine, migration 002 destroys the table. Search is completely broken.
- **Upgrades with data**: Data loss. The `SELECT *` fails, then `DROP TABLE` wipes everything.

**Recommended fix**: Rewrite migration 002 to:
1. Include ALL 17 columns in `products_meta_new`
2. Use explicit column names in `INSERT INTO ... SELECT` (not `SELECT *`)
3. Recreate FTS5 triggers after the table rename

**Effort**: Medium (~40 lines in 1 file, needs careful testing)

---

### Bug 3: Wishlist schema mismatch — CONFIRMED

**Root cause**: Migration 001 creates a minimal 4-column wishlist, but the export service was written against a richer 10-column schema that was never migrated.

**Migration 001 wishlist** (4 columns):
```sql
wishlist(sku TEXT PK, added_at INTEGER, price_at_add REAL, notes TEXT)
```

**export_service.rs `WishlistRow`** (10 columns):
`id, sku, name, brand, price, currency, image_url, product_url, notes, added_at`

**export_command.rs tests** create their own 10-column version — this is why tests pass but production would fail.

At runtime, `SELECT * FROM wishlist` returns 4 columns, but `sqlx::query_as::<_, WishlistRow>` expects 10 → runtime error on export.

**Affected files**:
- `src-tauri/src/repository/sqlite/migrations/001_init.sql` — line 69-74 (4-column schema)
- `src-tauri/src/services/export_service.rs` — lines 113-135 (10-column struct), lines 43-44 (SELECT *)
- `src-tauri/src/commands/export_command.rs` — lines 39-44 (test uses 10-column schema)

**Recommended fix**: Add migration 006 to alter the wishlist table to the 10-column schema. Do NOT modify migration 001 (it's already applied). Update export_command.rs tests to use the migration runner instead of inline schemas.

**Effort**: Medium (~30 lines: new migration + test alignment)

---

### Additional Gap: `delete_setting` not registered — CONFIRMED

- Function defined at `src-tauri/src/commands/settings_command.rs:42`
- NOT listed in `src-tauri/src/main.rs:20-30` invoke_handler
- Frontend has no references to `delete_setting` either
- **Fix**: Add `guitarhub_lib::commands::settings_command::delete_setting` to the `generate_handler!` macro in main.rs
- **Effort**: Trivial (1 line)

### Additional Gap: `ProductDetail` unused — CONFIRMED

- Component exists at `src/lib/components/ProductDetail.svelte` (51 lines)
- Zero imports across the entire `src/` directory
- It imports `PriceChart` which IS used elsewhere
- **Fix**: Either wire it into a product detail route/modal, or remove it as dead code
- **Effort**: Trivial (delete) or Medium (wire up a route)

### Additional Gap: Wishlist CRUD commands/UI missing — CONFIRMED

- No `add_to_wishlist`, `remove_from_wishlist`, `list_wishlist` commands exist
- No wishlist UI components in the frontend
- Only the export service reads from the wishlist table
- The feature is a skeleton: schema + export, no user-facing functionality
- **Fix**: Out of scope for this bug-fix change — flag as future work
- **Effort**: High (new feature, not a bug fix)

---

### Approaches

1. **Fix all 3 bugs + 2 trivial gaps in one change** — comprehensive, prevents partial fixes
   - Pros: Single pass, all integration issues resolved together
   - Cons: Larger change, harder to review
   - Effort: Medium

2. **Fix bugs 1-3 only, defer gaps** — focused on runtime failures
   - Pros: Smaller scope, clear priority
   - Cons: `delete_setting` remains broken (1-line fix left on the table)
   - Effort: Medium

3. **Fix per bug in separate changes** — maximum reviewability
   - Pros: Each fix is independently testable and reviewable
   - Cons: Migration 002 fix is urgent and shouldn't wait
   - Effort: Medium (same total, more overhead)

### Recommendation

**Approach 1**: Fix all 3 bugs + `delete_setting` registration in a single change. Defer `ProductDetail` cleanup and wishlist CRUD as separate future work. The bugs are interconnected (migration 002 breaks search, wishlist mismatch breaks export) and should be fixed together to validate the full integration path.

### Risks

- **Migration 002 rewrite**: If any user has already run the broken migration 002, their database is corrupted. Need a recovery path or detection mechanism.
- **Wishlist migration**: Adding columns to an existing table requires careful `ALTER TABLE` or table recreation. SQLite's `ALTER TABLE` limitations may require the recreate pattern.
- **Frontend field rename**: Low risk — only one file uses these fields, and the change is mechanical.
- **Test alignment**: Export tests use inline schemas that diverge from migrations. Fixing the production code without fixing tests creates a false sense of security.

### Fix Scope Estimate

| Fix | Files | ~Lines |
|-----|-------|--------|
| Bug 1 (search fields) | 1 | ~10 |
| Bug 2 (migration 002) | 1 | ~40 |
| Bug 3 (wishlist schema) | 1-2 | ~30 |
| delete_setting registration | 1 | ~1 |
| **Total** | **4-5** | **~80** |

### Dependencies (fix order)

1. **Bug 2 first** — migration 002 is the most critical (data loss risk). Must be fixed before any search testing.
2. **Bug 3 second** — wishlist migration needed before export works.
3. **Bug 1 third** — frontend field names, independent of DB fixes.
4. **delete_setting** — trivial, can be done anytime.

### Ready for Proposal

**Yes** — all bugs are confirmed with exact file/line evidence. The orchestrator should proceed to the proposal phase with the recommendation to fix bugs 1-3 + delete_setting registration, deferring ProductDetail and wishlist CRUD.
