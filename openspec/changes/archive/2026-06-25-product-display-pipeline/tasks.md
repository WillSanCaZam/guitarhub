# Tasks: Product Display Pipeline

## Review Workload Forecast

| Field | Value |
|-------|-------|
| Estimated changed lines | ~350–380 |
| 400-line budget risk | Low |
| Chained PRs recommended | No |
| Suggested split | Single PR |
| Delivery strategy | ask-on-risk |
| Chain strategy | pending |

Decision needed before apply: No
Chained PRs recommended: No
Chain strategy: pending
400-line budget risk: Low

## Phase 1: Service Tests (TDD RED)

- [x] 1.1 In `services/product_query.rs`: write `setup_db()` in-memory SQLite with `products_meta` + `price_history` tables, seed data, and write failing tests — empty catalog, active-only filter, case-insensitive SKU match, price-drop ordering

## Phase 2: Service + Commands (TDD GREEN)

- [x] 2.1 Implement `ProductQueryService` with 4 methods — `get_featured(limit)`, `get_price_drops(limit)`, `get_new_arrivals(limit)`, `get_by_sku(sku)` — each delegating to `services/product_query.rs`
- [x] 2.2 Create `commands/product_command.rs` — 4 `#[tauri::command]` fns (`get_featured_products`, `get_price_drops`, `get_new_arrivals`, `get_product_detail`) using `State<AppState>` and delegating to `ProductQueryService`

## Phase 3: Registration

- [x] 3.1 Add `pub mod product_query;` to `services/mod.rs`
- [x] 3.2 Add `pub mod product_command;` to `commands/mod.rs`
- [x] 3.3 Register 4 commands in `main.rs` `generate_handler![]`

## Phase 4: Frontend

- [x] 4.1 Create `routes/catalog/+page.svelte` importing `<SearchPanel />` wired with `filterState` + `collectionStore`

## Phase 5: Verify

- [x] 5.1 `cargo test` — all tests pass (420/420)
- [x] 5.2 `make lint` — clippy, svelte-check all pass

## Dependencies

```
1.1 (service tests) → 2.1 (service impl) → 2.2 (commands) → 3.1/3.2/3.3 (registration)
4.1 (catalog route) is independent — can run in parallel with Phase 2–3
5.1/5.2 (verify) is last
```
