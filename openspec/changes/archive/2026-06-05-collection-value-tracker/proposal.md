# Proposal: Collection Value Tracker

## Intent

GuitarHub has no way to track owned gear or its estimated value. This change adds a minimal collection tracker with gain/loss visibility.

## Scope

### In Scope
- Migration 007: `collection` table (purchase_price, purchase_date, condition, notes)
- Backend CRUD: add, remove, list, get_collection_value, get_collection_count
- Value: fallback latest `price_history` → `products_meta.price`
- Cell 8: collection count, total value, gain/loss badge
- ProductCard: "Add to collection" button
- Export: include `collection.json`

### Out of Scope
- Wishlist add/remove (wishlist has no CRUD today)
- Value-over-time chart / snapshots
- Photos, serial numbers, insurance reports
- Currency conversion
- Dedicated `/collection` route

## Capabilities

### New
- `collection-crud`: add, remove, list, update items
- `collection-value`: estimated value and gain/loss aggregation
- `collection-export`: include `collection.json` in ZIP

### Modified
- `dashboard-data`: add `get_collection_count` and `get_collection_value`
- `bento-grid-ui`: Cell 8 becomes "My Collection"
- `db-migration-runner`: migration 007

## Approach

**Schema**: separate `collection` table (not wishlist extension). Keeps domains distinct.

**CRUD**: follow existing `PriceHistoryRepo` pattern. Commands registered in `main.rs`.

**Value**: per-item `COALESCE(latest price_history, products_meta.price)`. Custom items (NULL `sku`) show as unvalued.

**2-PR strategy**:
- **PR 1**: Schema + backend CRUD + export + tests (~380 LoC)
- **PR 2**: Dashboard + frontend actions + store updates (~380 LoC)

## Affected Areas

| Area | Impact | Description |
|------|--------|-------------|
| `src-tauri/migrations/007_add_collection.sql` | New | collection schema |
| `src-tauri/src/repository/collection.rs` | New | CollectionRepo |
| `src-tauri/src/commands/collection_command.rs` | New | CRUD IPC commands |
| `src-tauri/src/services/export_service.rs` | Modified | Include collection.json |
| `src-tauri/src/main.rs` | Modified | Register commands |
| `src-tauri/src/commands/dashboard_command.rs` | Modified | collection stats |
| `src/lib/stores/dashboard.ts` | Modified | collection stats store |
| `src/lib/components/ProductCard.svelte` | Modified | Add-to-collection button |
| `src/routes/+page.svelte` | Modified | Cell 8 content |

## Risks

| Risk | Likelihood | Mitigation |
|------|------------|------------|
| Sparse price_history | Med | Fallback to products_meta.price; NULL sku items unvalued |
| Currency mismatch | Med | No conversion in MVP; defer |
| Migration 007 ordering | Low | Must follow 006; runner validates gaps |
| Test maintenance | Low | Update export tests hardcoding migration count |

## Rollback Plan

PR 1: drop `collection` table, remove migration 007, unregister commands. Revert.
PR 2: revert Svelte changes; Cell 8 returns to empty "Price Trends" state.

## Dependencies

- Migration 006 applied (wishlist schema aligned).

## Success Criteria

- [ ] `collection` table created via migration 007
- [ ] add/remove/list/get_collection_value commands registered and tested
- [ ] Cell 8 displays collection count and estimated value
- [ ] ProductCard renders "Add to collection" button
- [ ] Export ZIP includes `collection.json`
- [ ] All tests pass (`make test`)
