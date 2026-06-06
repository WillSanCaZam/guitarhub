# Tasks: Collection Value Tracker

## Review Workload Forecast

| Field | Value |
|-------|-------|
| Estimated changed lines | ~700 |
| 400-line budget risk | Medium |
| Chained PRs recommended | Yes |
| Suggested split | PR 1 (backend) → PR 2 (frontend) |
| Delivery strategy | auto-chain |
| Chain strategy | stacked-to-main |

Decision needed before apply: No
Chained PRs recommended: Yes
Chain strategy: stacked-to-main
400-line budget risk: Medium

### Suggested Work Units

| Unit | Goal | Likely PR | Notes |
|------|------|-----------|-------|
| 1 | Schema + backend CRUD, value logic, IPC, export | PR 1 | Merges to main; includes all Rust tests |
| 2 | Dashboard cell, ProductCard, collection view, store | PR 2 | Stacked on PR 1; includes frontend tests |

## Phase 1: Foundation

- [x] 1.1 Create `src-tauri/migrations/008_collection_items.sql`: table, index, condition CHECK constraint
- [x] 1.2 Create `src-tauri/src/repository/collection.rs`: `CollectionItem` and `CollectionItemInput` structs

## Phase 2: Core Backend

- [x] 2.1 Implement `CollectionRepo::add` + roundtrip test (RED → GREEN)
- [x] 2.2 Implement `CollectionRepo::get_all` + `get_by_id` with tests
- [x] 2.3 Implement `CollectionRepo::update` + `remove` with tests
- [x] 2.4 Write pure `estimated_value` function: 90d price_history avg → `products_meta.price` → `0.0`; test all 3 scenarios
- [x] 2.5 Implement `CollectionRepo::get_stats` (total_items, total_value, top_item); test empty and populated cases
- [x] 2.6 Create `src-tauri/src/commands/collection_command.rs`: 5 IPC commands; register in `main.rs`; test each
- [x] 2.7 Modify `src-tauri/src/services/export_service.rs` to include `collection_items.json`; update export tests

## Phase 3: Frontend Integration

- [x] 3.1 Build DashboardCell (Cell 8): total items, total value, top item via `get_collection_stats`
- [x] 3.2 Add conditional "Add to collection" button to `src/lib/components/ProductCard.svelte`; wire `add_to_collection` IPC
- [x] 3.3 Create `src/lib/components/CollectionView.svelte`: grid with name, brand, purchase_price, estimated_value, gain/loss, remove button
- [x] 3.4 Add `/collection` route accessible from dashboard or nav
- [x] 3.5 Create writable Svelte store for collection data; reactive updates on add/remove

## Phase 4: Verification

- [x] 4.1 Run `make test`, `npm run build`, `tsc`, `cargo test`, `clippy`; fix errors
