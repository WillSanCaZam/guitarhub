# Capability: collection-management

## Purpose

Backend schema, CRUD commands, value computation, and export integration for user-owned guitar collection.

## Requirements

### Requirement: collection_items table schema

The system MUST create a `collection_items` table with columns: `id INTEGER PRIMARY KEY AUTOINCREMENT`, `sku TEXT`, `name TEXT`, `brand TEXT`, `purchase_price REAL`, `purchase_currency TEXT`, `purchase_date INTEGER`, `condition TEXT`, `serial_number TEXT`, `notes TEXT`, `image_url TEXT`, `added_at INTEGER NOT NULL`.

#### Scenario: Migration creates table

- GIVEN a fresh database
- WHEN migrations run
- THEN `collection_items` exists with all 11 columns

### Requirement: add_to_collection command

The `add_to_collection` command MUST insert a row into `collection_items` and return the new `id`.

#### Scenario: Add valid item

- GIVEN SKU `GUITAR-001` with name "Stratocaster"
- WHEN `add_to_collection` is invoked
- THEN a row is persisted with the given fields
- AND the returned `id` is a positive integer

### Requirement: remove_from_collection command

The `remove_from_collection` command MUST delete the row by `id`.

#### Scenario: Remove existing item

- GIVEN `collection_items` has a row with `id = 5`
- WHEN `remove_from_collection(5)` is invoked
- THEN the row is deleted
- AND subsequent `get_collection` excludes it

### Requirement: get_collection command

The `get_collection` command MUST return all items with an `estimated_value` field computed per the estimated value requirement.

#### Scenario: Get populated collection

- GIVEN 3 items exist in `collection_items`
- WHEN `get_collection` is invoked
- THEN it returns a vec of 3 items
- AND each item includes `estimated_value`

#### Scenario: Get empty collection

- GIVEN `collection_items` has zero rows
- WHEN `get_collection` is invoked
- THEN it returns an empty vec

### Requirement: update_collection_item command

The `update_collection_item` command MUST modify any mutable field by `id`.

#### Scenario: Update purchase price

- GIVEN item `id = 3` has `purchase_price = 1000.0`
- WHEN `update_collection_item(3, {purchase_price: 1200.0})` is invoked
- THEN the row reflects `purchase_price = 1200.0`
- AND other fields remain unchanged

### Requirement: Condition enum

The `condition` column MUST accept only: `mint`, `excellent`, `good`, `fair`, `poor`.

#### Scenario: Invalid condition rejected

- GIVEN an insert with `condition = "broken"`
- WHEN the SQL executes
- THEN it MUST fail with a constraint error

### Requirement: Estimated value computation

`estimated_value` MUST return `None` when neither `price_history` (last 90 days) nor `products_meta.price` has data for the given SKU. If `price_history` has data, it returns the average. If only `products_meta` has data, it falls back to that price. The `get_stats` aggregation MUST treat `None` as `0.0` so that total collection value remains correct.

(Previously: `estimated_value` returned `Some(0.0)` when both lookups missed, conflating "no data" with "zero-value product".)

#### Scenario: Value from price history average

- GIVEN `price_history` has prices `1000`, `1100`, `1200` for SKU `X` in the last 90 days
- WHEN `get_collection` runs
- THEN `estimated_value` for SKU `X` is `Some(1100.0)`

#### Scenario: Value fallback to products_meta

- GIVEN SKU `Y` has no `price_history` rows in 90 days
- AND `products_meta.price = 800.0` for `Y`
- WHEN `get_collection` runs
- THEN `estimated_value` for SKU `Y` is `Some(800.0)`

#### Scenario: Value is None when no data exists

- GIVEN SKU `Z` has no `price_history` rows and no `products_meta` entry
- WHEN `get_collection` runs
- THEN `estimated_value` for SKU `Z` is `None`
- AND `get_stats` treats that `None` as `0.0` in the total

#### Scenario: Null SKU returns None

- GIVEN a collection item with `sku = NULL`
- WHEN `get_collection` runs
- THEN `estimated_value` for that item is `None`

#### Scenario: Stats aggregation treats None as 0.0

- GIVEN 2 items: one with `estimated_value = Some(500.0)` and one with `estimated_value = None`
- WHEN `get_stats` computes `total_value`
- THEN `total_value` is `500.0` (None treated as 0.0)
- AND `top_item_value` is `500.0`

### Requirement: Total collection value

`total_collection_value` MUST be the sum of `estimated_value` across all collection items.

#### Scenario: Sum with multiple items

- GIVEN 3 items with estimated values `1000`, `2000`, `1500`
- WHEN total value is computed
- THEN the result is `4500.0`

### Requirement: Export integration

The export ZIP MUST include `collection_items.json`.

#### Scenario: Export includes collection

- GIVEN `collection_items` has 2 rows
- WHEN `export_data` is invoked
- THEN the ZIP contains `collection_items.json` with 2 items

## Out of Scope

- Photos upload (image_url only, no file upload)
- Insurance export (deferred)
- Collection value chart over time (deferred, needs new table)
- One-click sell listing (deferred)
