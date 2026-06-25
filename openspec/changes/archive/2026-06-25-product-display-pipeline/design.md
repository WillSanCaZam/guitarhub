# Design: Product Display Pipeline

## Technical Approach

Single `ProductQueryService` with 4 read methods backed by `products_meta` queries (+ `price_history` JOIN for drops). Thin Tauri commands delegate to it (mirror `dashboard_command` pattern). Frontend: home page already calls commands with `.catch(() => [])` — no changes needed. Detail page needs no refactoring. New `/catalog` route mounts existing `SearchPanel`.

## Architecture Decisions

| Decision | Option A | Option B | Choice | Rationale |
|----------|----------|----------|--------|-----------|
| Service granularity | Single `ProductQueryService` | Per-query services | **Single service** | Queries share same table and return type; mirrors `DashboardRepo` pattern. No behavioral overlap risk. |
| Return type | `Vec<RawProduct>` | New DTO | **`Vec<RawProduct>`** | Home page already types as `RawProduct[]`. No extra fields needed — `specs_json` is parsed client-side in `ProductDetail.svelte`. |
| Price drops query | Subquery with `price_history` MIN/MAX | Application-layer join | **Subquery in SQL** | DB does the heavy lifting. Index `idx_price_history_sku_recorded` on `(sku, recorded_at)` already exists. |
| Detail SKU matching | `LOWER(sku) = LOWER(?)` | `sku = ?` COLLATE NOCASE | **`LOWER(sku) = LOWER(?)`** | Portabe across SQLite configs; no schema changes needed. |
| Route pattern | Standalone `/catalog` | Modal/overlay | **Standalone route** | Minimal surface area — single `.svelte` file mounting existing `SearchPanel`. Zero new Svelte logic. |

## Data Flow

```
+-- Tauri IPC ---+     +---- Command ------+     +---- Service --------+     +---- SQLite ----+
|                |     |                   |     |                     |     |                |
| Home page      |-->  | get_featured_     |-->  | ProductQueryService |-->  | products_meta  |
| (+page.svelte) |     | products          |     | .get_featured()     |     | WHERE is_active|
|                |     |                   |     |                     |     | =1 ORDER BY    |
|                |-->  | get_price_drops   |-->  | .get_price_drops()  |-->  | RANDOM()       |
|                |     |                   |     |                     |     |                |
|                |-->  | get_new_arrivals  |-->  | .get_new_arrivals() |-->  | synced_at DESC |
|                |     |                   |     |                     |     |                |
| Detail page    |-->  | get_product_      |-->  | .get_by_sku()       |-->  | LOWER(sku)=    |
| ([sku]/+page)  |     | detail            |     |                     |     | LOWER(?)       |
|                |     |                   |     |                     |     |                |
| /catalog       |-->  | search_products   |-->  | FtsSearchService    |-->  | products_fts   |
| (SearchPanel)  |     | (exists)          |     | (exists)            |     |                |
+----------------+     +-------------------+     +---------------------+     +----------------+

Price drops path (price_history JOIN):
  products_meta m
  JOIN (SELECT sku, MIN(recorded_at) first, MAX(recorded_at) last
        FROM price_history GROUP BY sku
        HAVING last_price < first_price) drops
```

## SQL Queries

**Featured** (`get_featured_products`):
```sql
SELECT sku, source_id, name, brand, model, category, subcategory,
       price, currency, condition, availability, url, image_url,
       specs_json, seller, location, synced_at
FROM products_meta
WHERE is_active = 1
ORDER BY RANDOM()
LIMIT ?
```

**Price Drops** (`get_price_drops`):
```sql
SELECT m.sku, m.source_id, m.name, m.brand, m.model, m.category,
       m.subcategory, m.price, m.currency, m.condition, m.availability,
       m.url, m.image_url, m.specs_json, m.seller, m.location, m.synced_at
FROM products_meta m
JOIN (
    SELECT ph.sku,
           (SELECT price FROM price_history
            WHERE sku = ph.sku ORDER BY recorded_at ASC LIMIT 1) AS first_price,
           (SELECT price FROM price_history
            WHERE sku = ph.sku ORDER BY recorded_at DESC LIMIT 1) AS last_price
    FROM price_history ph
    GROUP BY ph.sku
    HAVING last_price < first_price
) drops ON m.sku = drops.sku
WHERE m.is_active = 1
ORDER BY (drops.first_price - drops.last_price) DESC
LIMIT ?
```

**New Arrivals** (`get_new_arrivals`):
```sql
SELECT sku, source_id, name, brand, model, category, subcategory,
       price, currency, condition, availability, url, image_url,
       specs_json, seller, location, synced_at
FROM products_meta
WHERE is_active = 1
ORDER BY synced_at DESC
LIMIT ?
```

**Detail by SKU** (`get_product_detail`):
```sql
SELECT sku, source_id, name, brand, model, category, subcategory,
       price, currency, condition, availability, url, image_url,
       specs_json, seller, location, synced_at
FROM products_meta
WHERE LOWER(sku) = LOWER(?)
  AND is_active = 1
```

## Error Handling

| Scenario | Result |
|----------|--------|
| SKU not found / inactive | `Err(AppError::NotFound)` — serialises as `"not found"` |
| Empty string SKU | `Err(AppError::InvalidInput("sku is required"))` |
| DB error (any query) | `Err(AppError::Database(e.to_string()))` — from `sqlx::Error` auto-conversion |
| Discovery returns 0 rows | `Ok(vec![])` — empty array, not an error |
| Malformed `specs_json` | Returned as-is (string). Client-side `JSON.parse` in `ProductDetail.svelte` catches and returns `{}`. |

All discovery commands use `?` propagation for `sqlx::Error` → `AppError::Database`. Detail command adds explicit `NotFound` check after query.

## File Changes

| File | Action | Description |
|------|--------|-------------|
| `src-tauri/src/services/product_query.rs` | **Create** | `ProductQueryService` with 4 read methods + in-memory tests |
| `src-tauri/src/services/mod.rs` | Modify | Add `pub mod product_query;` |
| `src-tauri/src/commands/product_command.rs` | **Create** | 4 thin Tauri commands delegating to `ProductQueryService` |
| `src-tauri/src/commands/mod.rs` | Modify | Add `pub mod product_command;` |
| `src-tauri/src/main.rs` | Modify | Register 4 commands in `generate_handler![]` |
| `src/routes/catalog/+page.svelte` | **Create** | Mount `<SearchPanel />` with `filterState` + `collectionStore` |

## Frontend Changes

- **Home page** (`+page.svelte`): Zero changes. Already calls commands with `.catch(() => [])`.
- **Detail page** (`[sku]/+page.svelte`): Zero changes. Already invokes `get_product_detail` with error/loading states.
- **SearchPanel** already imports correctly — the `/catalog` route simply mounts it.
- **GearCard** accepts `RawProduct`-compatible shape (interface `GearCardProduct` is structurally compatible).

## Testing Strategy

| Layer | What | How |
|-------|------|-----|
| Service unit | SQL correctness via in-memory SQLite | Seed `products_meta` (+ `price_history` for drops), invoke each method, assert returned products. Mirror `search.rs` test infrastructure. |
| Service: empty catalog | All 4 methods return `vec![]` or `NotFound` | Empty DB, invoke each, assert correct |
| Service: inactive filter | `is_active=0` products excluded | Seed active + inactive, assert only active returned |
| Service: case-insensitive SKU | `get_by_sku("FENDER-001")` vs `"fender-001"` | Insert with mixed case, query with different case |
| Service: price drops ordering | Biggest drop first | Seed 3 products with known drop amounts, assert order |

No integration/E2E tests for this change — existing `.catch(() => [])` guards make missing commands non-fatal. E2E coverage via existing `ci.yml`.

## Spec Discrepancy Note

The `product-discovery` spec names the ordering column `created_at DESC` but the `products_meta` table schema has `synced_at` (no `created_at` column). The design uses `synced_at` — the actual column that exists. This was confirmed by checking the table schema in `search.rs` test fixtures and the `001_init` migration. The spec will be corrected at archive time.
