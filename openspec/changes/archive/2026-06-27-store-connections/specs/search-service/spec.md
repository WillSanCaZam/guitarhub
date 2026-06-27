# Delta for search-service

## ADDED Requirements

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

## MODIFIED Requirements

### Requirement: search_products Tauri command MUST exist

The system MUST provide `#[tauri::command] search_products(query: String, state: State<'_, AppState>, filters: Option<SearchFilters>, sort: Option<SortOrder>, limit: Option<u32>, offset: Option<u32>) -> Result<SearchResult, AppError>`. `SearchFilters` now includes the optional `store_connection_id` field.
(Previously: SearchFilters had no connection-specific filter)

| Case | Precondition | Action | Outcome |
|------|-------------|--------|---------|
| Basic text search — includes user | FTS5 has 10 public + 3 user guitars | `search_products("guitar")` | Returns up to 13 matching products |
| Empty query | Empty string | Search with "" | Returns `AppError::InvalidInput` |
| Special chars sanitized | Query "Fender!@#" | Search executes | Special chars stripped before FTS5 MATCH |

### Requirement: Search MUST filter by category, price, and source

The `SearchFilters` struct SHALL support optional `category: Option<String>`, `price_min: Option<f64>`, `price_max: Option<f64>`, `source: Option<String>`, and the new `store_connection_id: Option<String>`.
(Previously: no `store_connection_id` filter)

| Case | Precondition | Action | Outcome |
|------|-------------|--------|---------|
| Category filter — includes user | User and public guitars exist | `{ category: "Guitar" }` | Both public and user guitars returned |
| Source filter | Products from "reverb" and user | `{ source: "reverb" }` | Both public and user reverb products returned |
| Connection filter | User has Reverb listings | `{ store_connection_id: "reverb-u1" }` | Only user's Reverb products returned |
