# Delta for search-service

## MODIFIED Requirements

### Requirement: search_products Tauri command MUST exist

The system MUST provide `#[tauri::command] search_products(query: String, state: State<'_, AppState>, filters: Option<SearchFilters>, sort: Option<SortOrder>, limit: Option<u32>, offset: Option<u32>) -> Result<SearchResult, AppError>`.

The frontend MUST invoke this command using field names that match the Rust `SearchFilters` and `SearchResult` serialization: `price_min`, `price_max`, `source` (not `priceMin`, `priceMax`, `sourceId`); and read `products`, `total`, `offset`, `limit` from the response (not `items`, `page`).

(Previously: Command existed but frontend used mismatched field names causing silent data loss.)

#### Scenario: Frontend sends snake_case filter keys

- GIVEN the search page is loaded
- WHEN the user submits a search with price and source filters
- THEN the frontend sends `{ price_min, price_max, source }` (not `priceMin`, `priceMax`, `sourceId`)
- AND the Rust backend deserializes the filters without error

#### Scenario: Frontend reads products array from response

- GIVEN `search_products` returns a `SearchResult` with field `products`
- WHEN the frontend receives the response
- THEN it reads `res.products` (not `res.items`) to populate the results list

#### Scenario: Frontend derives pagination from offset and limit

- GIVEN `SearchResult` returns `offset` and `limit` (not `page`)
- WHEN the frontend receives the response
- THEN it computes the current page as `(offset / limit) + 1`
- AND uses `offset + limit` for the next page request

#### Scenario: Frontend sends offset and limit parameters

- GIVEN the user requests page 2 with page size 20
- WHEN the frontend invokes `search_products`
- THEN it sends `offset: 20` and `limit: 20` (not `page: 2, pageSize: 20`)

### Requirement: Search MUST filter by category, price, and source

The `SearchFilters` struct SHALL support optional `category: Option<String>`, `price_min: Option<f64>`, `price_max: Option<f64>`, and `source: Option<String>`. The frontend MUST use these exact field names when constructing filter objects.

(Previously: Frontend used camelCase names that did not match Rust serde deserialization.)

#### Scenario: Price range filter with correct field names

- GIVEN products at $100, $200, $500 exist
- WHEN frontend sends `{ price_min: 150, price_max: 400 }`
- THEN only the $200 product is returned

#### Scenario: Source filter with correct field name

- GIVEN products from "reverb" and "ebay" exist
- WHEN frontend sends `{ source: "reverb" }`
- THEN only reverb products are returned
