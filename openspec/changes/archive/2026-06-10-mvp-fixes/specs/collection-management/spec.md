# Delta for collection-management

> **Change**: mvp-fixes — Semantic bug C5

## MODIFIED Requirements

### Requirement: Estimated value computation

`estimated_value` MUST return `None` when neither `price_history` (last 90 days) nor `products_meta.price` has data for the given SKU. If `price_history` has data, it returns the average. If only `products_meta` has data, it falls back to that price. The `get_stats` aggregation MUST treat `None` as `0.0` so that total collection value remains correct.

(Previously: `estimated_value` returned `Some(0.0)` when both lookups missed, conflating "no data" with "zero-value product". Callers could not distinguish unknown SKU from a genuinely $0-priced item.)

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