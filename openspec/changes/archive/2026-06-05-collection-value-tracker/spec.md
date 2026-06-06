# Delta Specs for collection-value-tracker

## Capability: collection-management (NEW)

### Purpose
Backend schema, CRUD commands, value computation, and export integration for user-owned guitar collection.

### ADDED Requirements

#### REQ-COL-1: collection_items table schema

The system MUST create a `collection_items` table with columns: `id INTEGER PRIMARY KEY AUTOINCREMENT`, `sku TEXT`, `name TEXT`, `brand TEXT`, `purchase_price REAL`, `purchase_currency TEXT`, `purchase_date INTEGER`, `condition TEXT`, `serial_number TEXT`, `notes TEXT`, `image_url TEXT`, `added_at INTEGER NOT NULL`.

##### Scenario: Migration creates table

- GIVEN a fresh database
- WHEN migrations run
- THEN `collection_items` exists with all 11 columns

#### REQ-COL-2: add_to_collection command

The `add_to_collection` command MUST insert a row into `collection_items` and return the new `id`.

##### Scenario: Add valid item

- GIVEN SKU `GUITAR-001` with name "Stratocaster"
- WHEN `add_to_collection` is invoked
- THEN a row is persisted with the given fields
- AND the returned `id` is a positive integer

#### REQ-COL-3: remove_from_collection command

The `remove_from_collection` command MUST delete the row by `id`.

##### Scenario: Remove existing item

- GIVEN `collection_items` has a row with `id = 5`
- WHEN `remove_from_collection(5)` is invoked
- THEN the row is deleted
- AND subsequent `get_collection` excludes it

#### REQ-COL-4: get_collection command

The `get_collection` command MUST return all items with an `estimated_value` field computed per REQ-COL-7.

##### Scenario: Get populated collection

- GIVEN 3 items exist in `collection_items`
- WHEN `get_collection` is invoked
- THEN it returns a vec of 3 items
- AND each item includes `estimated_value`

##### Scenario: Get empty collection

- GIVEN `collection_items` has zero rows
- WHEN `get_collection` is invoked
- THEN it returns an empty vec

#### REQ-COL-5: update_collection_item command

The `update_collection_item` command MUST modify any mutable field by `id`.

##### Scenario: Update purchase price

- GIVEN item `id = 3` has `purchase_price = 1000.0`
- WHEN `update_collection_item(3, {purchase_price: 1200.0})` is invoked
- THEN the row reflects `purchase_price = 1200.0`
- AND other fields remain unchanged

#### REQ-COL-6: Condition enum

The `condition` column MUST accept only: `mint`, `excellent`, `good`, `fair`, `poor`.

##### Scenario: Invalid condition rejected

- GIVEN an insert with `condition = "broken"`
- WHEN the SQL executes
- THEN it MUST fail with a constraint error

#### REQ-COL-7: Estimated value computation

`estimated_value` MUST equal the average `price` from `price_history` for the SKU in the last 90 days. If no history exists, it MUST fall back to `products_meta.price`.

##### Scenario: Value from price history average

- GIVEN `price_history` has prices `1000`, `1100`, `1200` for SKU `X` in the last 90 days
- WHEN `get_collection` runs
- THEN `estimated_value` for SKU `X` is `1100.0`

##### Scenario: Value fallback to products_meta

- GIVEN SKU `Y` has no `price_history` rows in 90 days
- AND `products_meta.price = 800.0` for `Y`
- WHEN `get_collection` runs
- THEN `estimated_value` for SKU `Y` is `800.0`

#### REQ-COL-8: Total collection value

`total_collection_value` MUST be the sum of `estimated_value` across all collection items.

##### Scenario: Sum with multiple items

- GIVEN 3 items with estimated values `1000`, `2000`, `1500`
- WHEN total value is computed
- THEN the result is `4500.0`

#### REQ-COL-9: Export integration

The export ZIP MUST include `collection_items.json`.

##### Scenario: Export includes collection

- GIVEN `collection_items` has 2 rows
- WHEN `export_data` is invoked
- THEN the ZIP contains `collection_items.json` with 2 items

---

## Capability: collection-ui (NEW)

### Purpose
Frontend integration for collection management: dashboard cell, product card action, and collection view.

### ADDED Requirements

#### REQ-CUI-1: Cell 8 collection stats

Cell 8 in the bento grid MUST display collection stats: total items count, total estimated value, and top item by estimated value.

##### Scenario: Cell 8 renders stats

- GIVEN the dashboard loads
- WHEN Cell 8 is rendered
- THEN it shows total items, total value, and top item name

#### REQ-CUI-2: ProductCard add button

`ProductCard` MUST show an "Add to collection" button when the SKU is not already in `collection_items`.

##### Scenario: Add button visible

- GIVEN a product with SKU `NEW-001` not in collection
- WHEN the card renders
- THEN an "Add to collection" button is visible

#### REQ-CUI-3: Collection view accessible

A collection view MUST be reachable via `/collection` route.

##### Scenario: Navigate to collection

- GIVEN the user clicks a collection link
- WHEN navigation completes
- THEN the `/collection` route renders

#### REQ-CUI-4: Collection view grid

The collection view MUST display items in a grid showing `estimated_value`, `purchase_price`, and gain/loss (`estimated_value - purchase_price`).

##### Scenario: Grid shows gain/loss

- GIVEN an item with `purchase_price = 1000` and `estimated_value = 1200`
- WHEN the collection grid renders
- THEN it displays a gain of `+200`

#### REQ-CUI-5: Collection remove action

The collection view MUST allow removing items.

##### Scenario: Remove from collection view

- GIVEN the collection view shows item `id = 5`
- WHEN the user clicks remove and confirms
- THEN the item is removed
- AND the grid updates

---

## MODIFIED Capability: bento-grid-ui

### Requirement: Cell sizes MUST follow the bento pattern

Each cell MUST be assigned a grid span class that maps to one of four size variants:

| Cell | Size | grid-column | grid-row |
|------|------|-------------|----------|
| Cell 1 (Search Results) | Hero (2x2) | span 2 | span 2 |
| Cell 2 (Sync Status + Drops) | Wide (2x1) | span 2 | span 1 |
| Cell 3 (Total Products) | Standard (1x1) | span 1 | span 1 |
| Cell 4 (Wishlist Count) | Standard (1x1) | span 1 | span 1 |
| Cell 5 (Recent Searches) | Standard (1x1) | span 1 | span 1 |
| Cell 6 (Featured Deal) | Wide (2x1) | span 2 | span 1 |
| Cell 7 (Quick Settings) | Standard (1x1) | span 1 | span 1 |
| Cell 8 (Collection Stats) | Standard (1x1) | span 1 | span 1 |
| Cell 9 (App Info) | Standard (1x1) | span 1 | span 1 |

(Previously: Cell 8 was "Price Trend Mini-Chart" with no data source.)

#### Scenario: Cell 8 shows collection stats

- GIVEN the dashboard is rendered on desktop
- WHEN inspecting Cell 8
- THEN it displays total collection items, total value, and top item name
- AND it occupies exactly 1 column and 1 row

---

## MODIFIED Capability: dashboard-data

### Requirement: Dashboard commands MUST include collection metrics

The `get_collection_stats` command MUST return `(item_count: u32, total_value: f64, top_item_name: Option<String>)`. It MUST be registered in `main.rs` under the Tauri `generate_handler!` macro and MUST return within 50ms.

(Previously: No collection-related dashboard commands existed.)

#### Scenario: Returns stats for populated collection

- GIVEN `collection_items` has 5 items with total estimated value `7500.0`
- WHEN the frontend invokes `get_collection_stats`
- THEN it returns `(5, 7500.0, Some("Fender Stratocaster"))` within 50ms

#### Scenario: Returns zeros for empty collection

- GIVEN `collection_items` has zero rows
- WHEN the frontend invokes `get_collection_stats`
- THEN it returns `(0, 0.0, None)` within 50ms

## Out of Scope

- Photos upload (image_url only, no file upload)
- Insurance export (deferred)
- Collection value chart over time (deferred, needs new table)
- One-click sell listing (deferred)
