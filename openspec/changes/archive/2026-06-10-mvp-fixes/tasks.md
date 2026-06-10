# Tasks: MVP Fixes

## Review Workload Forecast

| Field | Value |
|-------|-------|
| Estimated changed lines | 400–500 |
| 400-line budget risk | Medium |
| Chained PRs recommended | Yes |
| Suggested split | PR 1 (WU1+WU2+WU3: fixes) → PR 2 (WU4: wishlist) |
| Delivery strategy | ask-on-risk |
| Chain strategy | stacked-to-main |

Decision needed before apply: Yes
Chained PRs recommended: Yes
Chain strategy: stacked-to-main
400-line budget risk: Medium

### Suggested Work Units

| Unit | Goal | Likely PR | Notes |
|------|------|-----------|-------|
| 1 | Panic + semantic + input fixes (C1–C6, D6, D7, user-agent) | PR 1 | targets main; surgical one-liners + small test changes |
| 2 | Wishlist CRUD (new feature) | PR 2 | targets main after PR 1; new structs, commands, store, page |

---

## Phase 1: Panic Fixes (C1–C4)

- [x] 1.1 Replace `unwrap()` with `unwrap_or_default()` at `src-tauri/src/commands/sync_command.rs:30` — timestamp computation for alert cooldown
- [x] 1.2 Replace `unwrap()` with `unwrap_or_default()` at `src-tauri/src/services/sync.rs:55` — `set_state` timestamp
- [x] 1.3 Replace `unwrap()` with `unwrap_or_default()` at `src-tauri/src/services/sync.rs:112-115` — `upsert_products` `synced_at` timestamp
- [x] 1.4 Replace `expect()` calls with `?` error propagation at `src-tauri/src/services/image_cache.rs:139-146` — return `ImageCacheError::DownloadFailed` instead of panicking on watch channel drop
- [x] 1.5 Change `"GuitarHub/0.1"` to `"GuitarHub/0.2.0"` in `src-tauri/src/lib.rs:67` (HTTP client user-agent)
- [x] 1.6 Change `"GuitarHub/0.1"` to `"GuitarHub/0.2.0"` in `src-tauri/src/services/image_cache.rs:88` (image cache HTTP client user-agent)

**Verification**: `cargo test` passes; `grep -rn 'unwrap()' src-tauri/src/commands/sync_command.rs src-tauri/src/services/sync.rs | grep -i 'duration_since\|UNIX_EPOCH'` returns nothing; `grep -rn 'expect(' src-tauri/src/services/image_cache.rs | grep -i 'watch\|in_flight\|channel'` returns nothing.

---

## Phase 2: Semantic Bug Fixes (C5, C6)

- [x] 2.1 Change `Ok(fallback.or(Some(0.0)))` to `Ok(fallback)` in `src-tauri/src/repository/collection.rs:271` — `estimated_value` returns `None` for unknown SKUs
- [x] 2.2 Update `estimated_value_zero_when_no_data` test assertion from `Some(0.0)` to `None` in `src-tauri/src/repository/collection.rs`
- [x] 2.3 Add falsy SKU guard in `src/lib/components/PriceChart.svelte` — skip `invoke('get_price_history')` when `sku` is falsy/undefined/whitespace-only; render empty state immediately

**Verification**: `estimated_value("unknown-sku")` returns `None`; `get_stats` still returns correct totals (existing `unwrap_or(0.0)` handles `None`); PriceChart renders empty state without IPC call when sku is falsy.

---

## Phase 3: Input Fixes (D6, D7)

- [x] 3.1 Add `condition` property to `addToCollection` input type in `src/lib/types/collection.ts` if a separate input type exists, otherwise update the call site in `src/lib/stores/collection.ts`
- [x] 3.2 Change `condition: 'good'` to `condition: product.condition ?? 'good'` in `src/lib/stores/collection.ts:63`
- [x] 3.3 Change `purchase_currency: product.currency || 'USD'` to `purchase_currency: product.currency ?? 'USD'` in `src/lib/stores/collection.ts:61`

**Verification**: `addToCollection` accepts `condition` from caller; empty-string currency is preserved (not replaced with `'USD'`); existing call sites still default to `'good'` when condition is omitted.

---

## Phase 4: Wishlist CRUD — Rust Backend

- [x] 4.1 Create `src-tauri/src/repository/wishlist.rs` with `WishlistItemInput`, `WishlistItem`, and `WishlistRepo` — struct with `new(pool)`, `add`, `remove`, `get_all` methods; follows `CollectionRepo` concrete-struct pattern
- [x] 4.2 Add `pub mod wishlist;` to `src-tauri/src/repository/mod.rs`
- [x] 4.3 Create `src-tauri/src/commands/wishlist_command.rs` with `add_to_wishlist`, `remove_from_wishlist`, `get_wishlist` commands — extracted `_cmd` functions for testability, matching `collection_command.rs` pattern
- [x] 4.4 Add `pub mod wishlist_command;` to `src-tauri/src/commands/mod.rs`
- [x] 4.5 Register 3 wishlist commands in `src-tauri/src/main.rs` `generate_handler![]`

**Verification**: `cargo test` compiles and passes; wishlist commands are accessible via Tauri IPC.

---

## Phase 5: Wishlist CRUD — Frontend

- [x] 5.1 Create `src/lib/types/wishlist.ts` with `WishlistItem` and `WishlistItemInput` interfaces matching Rust structs
- [x] 5.2 Create `src/lib/stores/wishlist.ts` with `writable<WishlistStore>`, `loadWishlist`, `addToWishlist`, `removeFromWishlist` — follows `collection.ts` pattern
- [x] 5.3 Create `src/routes/wishlist/+page.svelte` — renders wishlist items from `wishlistStore`, shows empty state, includes remove button per item
- [x] 5.4 Add wishlist nav link with count badge in `src/routes/+layout.svelte` — import `wishlistStore`, show item count, link to `/wishlist`

**Verification**: App compiles; navigating to `/wishlist` shows wishlist page; badge updates on add/remove.

---

## Phase 6: Tests

- [x] 6.1 Add unit tests for `WishlistRepo::add`, `remove`, `get_all` in `src-tauri/src/repository/wishlist.rs` — in-memory SQLite, same pattern as `CollectionRepo` tests
- [x] 6.2 Add unit tests for `add_to_wishlist_cmd`, `remove_from_wishlist_cmd`, `get_wishlist_cmd` in `src-tauri/src/commands/wishlist_command.rs` — in-memory pool, test round-trips
- [x] 6.3 Add test for `estimated_value` returning `None` for null SKU in `src-tauri/src/repository/collection.rs`
- [x] 6.4 Add test for `image_cache` watch channel drop returning `Err(ImageCacheError::DownloadFailed)` — verify no panic when sender is dropped (covered by 1.4's `concurrent_request_gets_error_when_fetcher_fails` test)

**Verification**: `cargo test` — all tests pass, including new wishlist and estimated_value tests.