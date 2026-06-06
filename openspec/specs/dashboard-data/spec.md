# Capability: dashboard-data

## Purpose

Provide lightweight Rust IPC commands that supply aggregate metrics for the bento-grid dashboard cells. All commands are read-only, query existing tables, and MUST return within 50ms.

## Requirements

### Requirement: get_total_products command MUST return product count

The `get_total_products` Tauri command MUST execute `SELECT COUNT(*) FROM products_meta` and return the count as a `u64`. The command MUST be registered in `main.rs` under the Tauri `generate_handler!` macro.

#### Scenario: Returns exact count for populated catalog

- GIVEN the products_meta table contains 1,247 rows
- WHEN the frontend invokes `get_total_products`
- THEN the command returns `1247` within 50ms

#### Scenario: Returns zero for empty catalog

- GIVEN the products_meta table has zero rows
- WHEN the frontend invokes `get_total_products`
- THEN the command returns `0` within 50ms

#### Scenario: Frontend displays count in Cell 3

- GIVEN `get_total_products` returns `1247`
- WHEN the dashboard renders Cell 3
- THEN the count `1247` is displayed with the label "Products"

### Requirement: get_wishlist_count command MUST return wishlist count

The `get_wishlist_count` Tauri command MUST execute `SELECT COUNT(*) FROM wishlist` and return the count as a `u64`. The command MUST be registered in `main.rs` under the Tauri `generate_handler!` macro.

#### Scenario: Returns exact count for populated wishlist

- GIVEN the wishlist table contains 23 rows
- WHEN the frontend invokes `get_wishlist_count`
- THEN the command returns `23` within 50ms

#### Scenario: Returns zero for empty wishlist

- GIVEN the wishlist table has zero rows
- WHEN the frontend invokes `get_wishlist_count`
- THEN the command returns `0` within 50ms

#### Scenario: Frontend displays count in Cell 4

- GIVEN `get_wishlist_count` returns `23`
- WHEN the dashboard renders Cell 4
- THEN the count `23` is displayed with the label "Wishlist"

### Requirement: get_recent_searches command MUST return last 5 unique searches

The `get_recent_searches` Tauri command MUST read the last 5 unique search queries from the frontend's localStorage (key: `guitarhub_recent_searches`), deduplicated by query string (case-insensitive). The return type MUST be `Vec<String>`. The command MUST be registered in `main.rs` under the Tauri `generate_handler!` macro.

#### Scenario: Returns last 5 unique searches

- GIVEN localStorage contains `["fender", "gibson", "fender", "prs", "martin", "taylor"]`
- WHEN the frontend invokes `get_recent_searches`
- THEN the command returns `["taylor", "martin", "prs", "gibson", "fender"]` (most recent first, deduplicated)

#### Scenario: Returns empty list when no searches exist

- GIVEN localStorage has no `guitarhub_recent_searches` key
- WHEN the frontend invokes `get_recent_searches`
- THEN the command returns `[]`

#### Scenario: Frontend displays searches in Cell 5

- GIVEN `get_recent_searches` returns `["fender", "gibson", "prs"]`
- WHEN the dashboard renders Cell 5
- THEN the list displays as clickable links that trigger a search

### Requirement: get_collection_stats command MUST return collection metrics

The `get_collection_stats` Tauri command MUST return `(item_count: u32, total_value: f64, top_item_name: Option<String>)`. It MUST be registered in `main.rs` under the Tauri `generate_handler!` macro and MUST return within 50ms.

#### Scenario: Returns stats for populated collection

- GIVEN `collection_items` has 5 items with total estimated value `7500.0`
- WHEN the frontend invokes `get_collection_stats`
- THEN it returns `(5, 7500.0, Some("Fender Stratocaster"))` within 50ms

#### Scenario: Returns zeros for empty collection

- GIVEN `collection_items` has zero rows
- WHEN the frontend invokes `get_collection_stats`
- THEN it returns `(0, 0.0, None)` within 50ms

### Requirement: All dashboard commands MUST return within 50ms

The `get_total_products`, `get_wishlist_count`, `get_recent_searches`, and `get_collection_stats` commands MUST complete and return within 50ms under normal conditions (local SQLite, warm cache). The 50ms budget includes deserialization, query execution, and serialization.

#### Scenario: All commands complete under 50ms

- GIVEN the database has 10,000 products and 50 wishlist items
- WHEN all four commands are invoked sequentially
- THEN each command completes in under 50ms
- AND the total sequential time is under 200ms

#### Scenario: Commands fail gracefully on database error

- GIVEN the database connection is unavailable
- WHEN any dashboard command is invoked
- THEN it returns an `AppError::Database` with a user-friendly message
- AND the frontend cell displays the error in its empty state

## Out of Scope

- Background scheduler for auto-refresh
- Real-time WebSocket updates
- Per-cell customization (drag-to-rearrange)
