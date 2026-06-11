# Delta for dashboard-data

## MODIFIED Requirements

### Requirement: get_total_products command MUST return product count

The `get_total_products` Tauri command MUST execute `SELECT COUNT(*) FROM products_meta` and return the count as a `u32`.
(Previously: returned `u64`)

#### Scenario: Returns exact count for populated catalog

- GIVEN the products_meta table contains 1,247 rows
- WHEN the frontend invokes `get_total_products`
- THEN the command returns `1247`

#### Scenario: Returns zero for empty catalog

- GIVEN the products_meta table has zero rows
- WHEN the frontend invokes `get_total_products`
- THEN the command returns `0`

#### Scenario: Frontend displays count in Cell 3

- GIVEN `get_total_products` returns `1247`
- WHEN the dashboard renders Cell 3
- THEN the count `1247` is displayed with the label "Products"

### Requirement: get_collection_stats command MUST return collection metrics

The `get_collection_stats` Tauri command MUST return `(item_count: u32, total_value: f64, top_item_name: Option<String>, top_item_value: f64)`.
(Previously: missing `top_item_value: f64`)

#### Scenario: Returns stats for populated collection

- GIVEN `collection_items` has 5 items with total estimated value `7500.0`
- WHEN the frontend invokes `get_collection_stats`
- THEN it returns `(5, 7500.0, Some("Fender Stratocaster"), 2500.0)`

#### Scenario: Returns zeros for empty collection

- GIVEN `collection_items` has zero rows
- WHEN the frontend invokes `get_collection_stats`
- THEN it returns `(0, 0.0, None, 0.0)`

### Requirement: All commands MUST return within 50ms

The six dashboard commands MUST complete within 50ms under normal conditions (local SQLite, warm cache), including deserialization and serialization.
(Previously: referenced "four" commands)

#### Scenario: All commands complete under 50ms

- GIVEN the database has 10,000 products and 50 wishlist items
- WHEN all six commands are invoked sequentially
- THEN each command completes in under 50ms
- AND the total sequential time is under 300ms

#### Scenario: Commands fail gracefully on database error

- GIVEN the database connection is unavailable
- WHEN any dashboard command is invoked
- THEN it returns an `AppError::Database` with a user-friendly message
- AND the frontend cell displays the error in its empty state

## ADDED Requirements

### Requirement: get_categories command MUST return distinct product categories

The `get_categories` Tauri command MUST execute `SELECT DISTINCT category FROM products_meta ORDER BY category` and return the result as `Vec<String>`.

#### Scenario: Returns empty list for empty catalog

- GIVEN the products_meta table has zero rows
- WHEN the frontend invokes `get_categories`
- THEN the command returns `[]`

#### Scenario: Returns distinct sorted categories

- GIVEN the products_meta table contains products with categories "Pedal", "Guitar", "Amplifier", "Bass", and "Guitar" (duplicate)
- WHEN the frontend invokes `get_categories`
- THEN the command returns `["Amplifier", "Bass", "Guitar", "Pedal"]`

### Requirement: record_search command MUST persist a search query

The `record_search` Tauri command MUST accept a `query: String` parameter, insert it into the `recent_searches` table with the current Unix timestamp, and return `()`. If the same query already exists, it MUST update the timestamp via `ON CONFLICT DO UPDATE`.

#### Scenario: Persists a new search query

- GIVEN the recent_searches table is empty
- WHEN the frontend invokes `record_search` with `query = "Les Paul"`
- THEN the recent_searches table contains one row with `query = "Les Paul"`

#### Scenario: Updates timestamp for duplicate query

- GIVEN the recent_searches table contains a row with `query = "Stratocaster"` and `searched_at = 1000`
- WHEN the frontend invokes `record_search` with `query = "Stratocaster"`
- THEN the row's `searched_at` is updated to the current timestamp

#### Scenario: Allows multiple distinct queries

- GIVEN the recent_searches table is empty
- WHEN the frontend invokes `record_search` with `query = "Telecaster"` three times
- THEN the recent_searches table contains exactly 1 row for "Telecaster"
