# Proposal: MVP Fixes

## Intent

Fix 6 critical panics/bugs and fill 2 blocking MVP gaps so GuitarHub v0.1 is safe to ship. Defer design polish and nice-to-haves to later changes.

## Scope

### In Scope

1. **C1-C3**: Replace 4 `unwrap()` on `SystemTime::now().duration_since(UNIX_EPOCH)` with `unwrap_or_default()` — `dashboard_command.rs:49` already safe; fix `sync_command.rs:30` and `sync.rs:55,112-115`.
2. **C4**: Replace `expect()` in `image_cache.rs:139` with graceful `Err` handling — return `ImageCacheError::DownloadFailed` when the watch channel drops.
3. **C5**: Change `collection.rs:271` `estimated_value` to return `None` when no data exists (both `price_history` and `products_meta` miss) instead of `Some(0.0)`. Update `get_stats` to treat `None` as `0.0` for aggregation.
4. **C6**: Guard `PriceChart.svelte` — skip `invoke('get_price_history')` when `sku` is falsy/undefined.
5. **D6**: Fix `collection.ts:63` — pass `condition` through from caller instead of hardcoding `'good'`.
6. **D7**: Fix `collection.ts:61` — change `||` to `??` for currency fallback so empty string `""` is not treated as falsy.
7. **Wishlist CRUD**: Add add/remove/list Tauri commands (Rust + JS bindings) and basic UI view — table and count query already exist.
8. **User-agent version**: Align `GuitarHub/0.1` in `lib.rs:67` and `image_cache.rs:88` with `package.json` version `0.2.0`.

### Out of Scope

- D1 (http:// defense-in-depth) — security hardening, separate change
- D2 (specs_json dead field) — low priority, cosmetic
- D3 (dashboard raw SQL) — architectural refactor, post-MVP
- D5 (search filters UI) — feature addition, post-MVP
- D8/D9 (scraper category/API) — Python scraper, separate change
- Dark mode completeness — budget not justified for v0.1
- TypeScript all Svelte components — large refactor, post-MVP
- CI frontend build check — CI pipeline change, separate change
- Scrape workflow fix — ops/infra, separate change
- `window.confirm()` in CollectionView — UX polish, post-MVP
- Repository trait inconsistency — architectural, post-MVP

## Capabilities

### New Capabilities

- `wishlist-crud`: Add, remove, and list wishlist items via Tauri commands with JS bindings

### Modified Capabilities

- `local-image-cache`: Fix panic in request coalescing when watch channel drops (C4)
- `collection-management`: `estimated_value` must return `None` for unknown SKUs instead of `Some(0.0)` (C5)
- `sync-service`: Replace `unwrap()` timestamps with `unwrap_or_default()` (C1-C3)
- `ui`: PriceChart guard for empty SKU; collection input fixes (C6, D6, D7)

## Approach

**Batch 1 — Panic fixes (C1-C4)**: Replace all `unwrap()` on `SystemTime::now().duration_since(UNIX_EPOCH)` with `unwrap_or_default()` in `sync_command.rs` and `sync.rs`. Replace `expect()` in `image_cache.rs:139` with proper `Err` return. These are one-liner changes each, no API contract changes.

**Batch 2 — Semantic bugs (C5, C6)**: Change `estimated_value` to return `None` when both lookups miss, keeping `get_stats` aggregating correctly. Add falsy guard in PriceChart `onMount`.

**Batch 3 — Input fixes (D6, D7, user-agent)**: Fix `addToCollection` to accept `condition` and use `??` for currency. Bump user-agent strings to `0.2.0`.

**Batch 4 — Wishlist CRUD**: Add Rust commands (`add_to_wishlist`, `remove_from_wishlist`, `get_wishlist`), a `WishlistRepo`, JS store + bindings, and a minimal Svelte view wired into the sidebar/nav.

## Affected Areas

| Area | Impact | Description |
|------|--------|-------------|
| `src-tauri/src/commands/sync_command.rs` | Modified | `unwrap()` → `unwrap_or_default()` |
| `src-tauri/src/services/sync.rs` | Modified | 2x `unwrap()` → `unwrap_or_default()` |
| `src-tauri/src/services/image_cache.rs` | Modified | `expect()` → graceful `Err` |
| `src-tauri/src/repository/collection.rs` | Modified | `estimated_value` return `None` |
| `src/lib/components/PriceChart.svelte` | Modified | SKU falsy guard |
| `src/lib/stores/collection.ts` | Modified | condition + `??` fix |
| `src/lib/types/collection.ts` | Modified | condition field in input type |
| `src-tauri/src/lib.rs` | Modified | user-agent `0.1` → `0.2.0` |
| `src-tauri/src/commands/wishlist_command.rs` | New | Wishlist add/remove/list commands |
| `src-tauri/src/repository/wishlist.rs` | New | WishlistRepo CRUD |
| `src/lib/stores/wishlist.ts` | New | Wishlist JS store |
| `src/lib/types/wishlist.ts` | New | Wishlist TS types |
| `src/routes/wishlist/+page.svelte` | New | Wishlist UI page |

## Risks

| Risk | Likelihood | Mitigation |
|------|------------|------------|
| `estimated_value` `None` breaks frontend consumers that expect `0` | Low | Frontend already expects `number \| null` via TypeScript — audit render paths |
| Wishlist commands need new migration or reuse table | Low | Table already exists via migration 006; just add repo + commands |
| Image cache channel drop is rare but possible under memory pressure | Low | Returning `Err` is safe; callers already handle `Err` |

## Rollback Plan

Each batch is independently revertible via `git revert`. For the wishlist batch, removing the 4 new files + the `invoke_handler` entries reverts cleanly. No schema changes (wishlist table already exists).

## Dependencies

- Wishlist depends on migration 006 (already applied).

## Success Criteria

- [ ] Zero `unwrap()` / `expect()` on `SystemTime` or watch channels remain in production paths
- [ ] `estimated_value("nonexistent-sku")` returns `None`, not `Some(0.0)`
- [ ] `PriceChart.svelte` renders empty state (no invoke) when `sku` is falsy
- [ ] `addToCollection` accepts condition from caller; `??` used for currency
- [ ] User-agent reports `0.2.0`
- [ ] Wishlist add/remove/list round-trips through Tauri IPC
- [ ] All existing tests pass; new wishlist commands have tests