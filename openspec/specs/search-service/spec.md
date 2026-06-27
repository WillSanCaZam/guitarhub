# Search Service Specification

> **Status**: New capability  
> **Change**: mvp-completion

## Purpose

Provide FTS5-powered product search with input sanitization, multi-dimensional filtering (category, price range, source), pagination, and sorting — exposed as a Tauri IPC command.

## Requirements

### Requirement: search_products Tauri command MUST exist

The system MUST provide `#[tauri::command] search_products(query: String, state: State<'_, AppState>, filters: Option<SearchFilters>, sort: Option<SortOrder>, limit: Option<u32>, offset: Option<u32>) -> Result<SearchResult, AppError>`. `SearchFilters` now includes the optional `store_connection_id` field.

The frontend MUST invoke this command using field names that match the Rust `SearchFilters` and `SearchResult` serde serialization: `price_min`, `price_max`, `source`, `category`, `store_connection_id` (not `priceMin`, `priceMax`, `sourceId`, `categoryId`, `storeId`); and read `products`, `total`, `offset`, `limit` from the response (not `items`, `page`). Pagination MUST be derived as `page = (offset / limit) + 1` from the response, and the next page request MUST use `offset + limit`.

| Case | Precondition | Action | Outcome |
|------|-------------|--------|---------|
| Basic text search — includes user | FTS5 has 10 public + 3 user guitars | `search_products("guitar")` | Returns up to 13 matching products |
| Empty query | Empty string passed | Search with "" | Returns `AppError::InvalidInput` ("query too short") |
| Query too short | 1-char string | Search with "a" | Returns `AppError::InvalidInput` (min 3 chars) |
| Special chars sanitized | Query "Fender!@#" | Search executes | Special chars stripped/escaped before FTS5 MATCH |
| No results | Query "xyznonexistent" | Search | `total: 0`, `products: []` |
| Frontend reads `products` array | Response received | Frontend processes result | Reads `res.products` (not `res.items`) to populate the results list |
| Frontend sends `offset` and `limit` | User requests page 2, size 20 | `invoke` called | Sends `offset: 20`, `limit: 20` (not `page: 2, pageSize: 20`) |
| Frontend derives page from offset | Response has `offset: 20, limit: 20` | Frontend renders | Computes `page = (20 / 20) + 1 = 2` |

### Requirement: Search MUST filter by category, price, source, and store connection

The `SearchFilters` struct SHALL support optional `category: Option<String>`, `price_min: Option<f64>`, `price_max: Option<f64>`, `source: Option<String>`, and the new `store_connection_id: Option<String>`. The frontend MUST use these exact field names (snake_case) when constructing filter objects to match the Rust serde deserialization.

| Case | Precondition | Action | Outcome |
|------|-------------|--------|---------|
| Category filter — includes user | User and public guitars exist | `{ category: "Guitar" }` | Both public and user guitars returned |
| Price range filter | Products at $100, $200, $500 | Search with `{ price_min: 150, price_max: 400 }` | Only $200 product returned |
| Source filter | Products from "reverb" and user | Search with `{ source: "reverb" }` | Both public and user reverb products returned |
| Connection filter | User has Reverb listings | `{ store_connection_id: "reverb-u1" }` | Only user's Reverb products returned |
| Combined filters | All filters active | Search with category + price + source | Intersection of all filters applied |

### Requirement: Search MUST paginate

Results SHALL support `limit` (default 20, max 100) and `offset` (default 0). The response MUST include `total: u64` for total matching count.

| Case | Precondition | Action | Outcome |
|------|-------------|--------|---------|
| First page | 50 total matches | `{ limit: 10, offset: 0 }` | Returns 10 products, `total: 50` |
| Second page | 50 total matches | `{ limit: 10, offset: 10 }` | Returns next 10 products |
| Page beyond results | 5 total matches | `{ limit: 10, offset: 10 }` | Returns 0 products, `total: 5` |

### Requirement: Search MUST sort results

The `SortOrder` enum SHALL support `Relevance` (default), `PriceAsc`, `PriceDesc`, `NameAsc`, and `NameDesc`.

| Case | Precondition | Action | Outcome |
|------|-------------|--------|---------|
| Sort by price ascending | Products at $100, $500, $200 | `{ sort: "PriceAsc" }` | Returns $100, $200, $500 |
| Sort by name | Products "A", "C", "B" | `{ sort: "NameAsc" }` | Returns "A", "B", "C" |
| Default relevance | Query "guitar" | No sort specified | Returns by FTS5 rank (best match first) |

### Requirement: FTS5 MUST index user-connected products

The FTS5 index SHALL include rows from `products_meta` where `user_id IS NOT NULL`. The `search_products` command SHALL return results from both public (`user_id IS NULL`) and user-connected products. The `SearchResult` SHALL include a `source_store` field indicating which store the product originates from.

| Case | Precondition | Action | Outcome |
|------|-------------|--------|---------|
| User product in FTS5 | User listing "Fender Jazz Bass" exists | Search "Jazz Bass" | User listing returned in results |
| Source field populated | Results mixed public + user | Search results | Each product includes `source` field |
| No user products | No connections exist | Search any query | Same results as before — no regression |

### Requirement: Search MUST filter by store connection status

A new filter field `store_connection_id: Option<String>` SHALL be added to `SearchFilters` to narrow results to a specific user connection. A value of `"public"` SHALL exclude all user-connected products, returning only public-scraped items.

| Case | Precondition | Action | Outcome |
|------|-------------|--------|---------|
| Filter by connection | User has Reverb connected | `{ store_connection_id: "reverb-u1" }` | Only that user's Reverb products returned |
| Filter public only | User has connections | `{ store_connection_id: "public" }` | Only public products returned, user products excluded |
| No filter | Mixed catalog | No filter provided | All products (public + user) returned |

### Requirement: Input MUST be sanitized before FTS5 MATCH

The query MUST be sanitized to prevent FTS5 syntax errors or injection: wrap each word in double quotes, strip FTS5 operators (`*`, `"`, `(`, `)`, `NEAR`, `NOT`, `OR`, `AND`).

| Case | Precondition | Action | Outcome |
|------|-------------|--------|---------|
| Contains quotes | Query: `guitar "electric"` | Sanitize | Becomes `"guitar" "electric"` |
| Contains operators | Query: `guitar NOT bass` | Sanitize | Becomes `"guitar" "bass"` (NOT removed) |
| Chinese/unicode | Query: `吉他` | Sanitize | Becomes `"吉他"` (trigram tokenizer handles it) |
