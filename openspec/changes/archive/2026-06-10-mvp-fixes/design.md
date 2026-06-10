# Design: MVP Fixes

## Technical Approach

Four batches of changes, from panic fixes to new feature. Batches 1-3 are surgical one-liners. Batch 4 (Wishlist CRUD) follows the existing concrete-struct repo pattern (no trait) and mirrors the collection command/store architecture.

## Architecture Decisions

### Decision: Rust repo pattern — concrete struct, no trait

**Choice**: `WishlistRepo` as a concrete struct with `new(pool: SqlitePool) -> Self`, same as `CollectionRepo`
**Alternatives considered**: Trait-based repo (like `SettingsRepository`) for testability
**Rationale**: 5 of 6 repos in this project use concrete structs (`CollectionRepo`, `PriceHistoryRepo`, `ImageCacheRepo`, `PriceDropNotificationsRepo`). Only `SettingsRepository` uses a trait, because it's injected into `ImageCacheService` and needs mocking. Wishlist commands construct the repo inline from `state.pool` per-request, identical to collection commands — no external injection point needs a trait. Follow the prevailing pattern.

### Decision: Wishlist command structure — thin wrapper over repo

**Choice**: Extract pure-logic `_cmd` functions (e.g. `add_to_wishlist_cmd(pool, input)`) alongside `#[tauri::command]` wrappers, matching `collection_command.rs`
**Alternatives considered**: Command calls repo directly without extracted function
**Rationale**: The existing collection commands use this pattern for testability without Tauri runtime. Maintaining consistency is more valuable than saving the extraction for just 3 commands.

### Decision: estimated_value returns `Option<f64>` (already the type) but `None` instead of `Some(0.0)`

**Choice**: Change the final fallback in `estimated_value()` from `fallback.or(Some(0.0))` to `fallback` (returning `None` when both lookups miss). `get_stats` already uses `unwrap_or(0.0)`.
**Alternatives considered**: Return `0.0` and use a separate boolean flag
**Rationale**: `CollectionItem.estimated_value` is already `Option<f64>` in Rust and `number | null` in TypeScript. The frontend already uses `?? 0` via `collectionValue.ts`. Changing the semantic from `Some(0.0)` to `None` requires zero frontend type changes — only one test assertion changes.

### Decision: Frontend wishlist store follows `collection.ts` pattern

**Choice**: `writable<WishlistStore>` with `loadWishlist`, `addToWishlist`, `removeFromWishlist` functions
**Alternatives considered**: Svelte 5 `$state` rune-based store
**Rationale**: All existing stores (`collection.ts`, `sync.ts`, `dashboard.ts`) use `writable` from `svelte/store`. Switching to runes would be inconsistent and is a post-MVP refactor.

## Data Flow

### Batch 1-3 (simple fixes)

```
sync_command.rs:30   unwrap() → unwrap_or_default()
sync.rs:55,112-115  unwrap() → unwrap_or_default()
image_cache.rs:139  expect() → Err(DownloadFailed)
collection.rs:271   .or(Some(0.0)) → (return fallback directly)
PriceChart.svelte   add falsy guard before invoke()
collection.ts:63   'good' → function param (condition)
collection.ts:61   || → ??
lib.rs:67, image_cache.rs:88  GuitarHub/0.1 → GuitarHub/0.2.0
```

### Batch 4 (Wishlist CRUD)

```
Frontend                          Tauri IPC                     Rust
──────────                        ─────────                     ────
wishlist.ts                       invoke('add_to_wishlist')  → add_to_wishlist_cmd()
  ┌─ WishlistStore                 → invoke('remove_from_wishlist') → remove_from_wishlist_cmd()
  │  items: WishlistItem[]          → invoke('get_wishlist')  → get_wishlist_cmd()
  │  loading: bool
  │  error: string|null                      ↓
  └─ loadWishlist()               Result<...> ← WishlistRepo::new(pool).add/remove/get_all
                                              ← wishlist table (migration 006)
+page.svelte ──→ wishlistStore
+layout.svelte ──→ wishlistStore.items.length (count badge)
```

## File Changes

| File | Action | Description |
|------|--------|-------------|
| `src-tauri/src/repository/collection.rs` | Modify | Change `estimated_value` final fallback to return `None` instead of `Some(0.0)` |
| `src-tauri/src/commands/sync_command.rs` | Modify | Replace `unwrap()` with `unwrap_or_default()` on line 30 |
| `src-tauri/src/services/sync.rs` | Modify | Replace `unwrap()` with `unwrap_or_default()` on lines 55, 112-115 |
| `src-tauri/src/services/image_cache.rs` | Modify | Replace `expect()` with `?` error propagation on watch channel; bump user-agent to `0.2.0` |
| `src-tauri/src/lib.rs` | Modify | Bump user-agent to `GuitarHub/0.2.0` |
| `src/lib/components/PriceChart.svelte` | Modify | Add falsy SKU guard in `onMount` |
| `src/lib/stores/collection.ts` | Modify | Add `condition` param to `addToCollection`; change `\|\|` to `??` |
| `src/lib/types/collection.ts` | Modify | Add `condition` to `addToCollection` input type (if separate input type exists) |
| `src-tauri/src/repository/wishlist.rs` | **Create** | `WishlistRepo` with `add`, `remove`, `get_all` methods |
| `src-tauri/src/repository/mod.rs` | Modify | Add `pub mod wishlist;` |
| `src-tauri/src/commands/wishlist_command.rs` | **Create** | `add_to_wishlist`, `remove_from_wishlist`, `get_wishlist` commands |
| `src-tauri/src/commands/mod.rs` | Modify | Add `pub mod wishlist_command;` |
| `src-tauri/src/main.rs` | Modify | Register 3 wishlist commands in `generate_handler!` |
| `src/lib/stores/wishlist.ts` | **Create** | Wishlist store with `loadWishlist`, `addToWishlist`, `removeFromWishlist` |
| `src/lib/types/wishlist.ts` | **Create** | `WishlistItem` and `WishlistItemInput` interfaces |
| `src/routes/wishlist/+page.svelte` | **Create** | Wishlist page component |
| `src/routes/+layout.svelte` | Modify | Add wishlist nav link with count badge |

## Interfaces / Contracts

### Rust: WishlistRepo

```rust
// src-tauri/src/repository/wishlist.rs
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WishlistItemInput {
    pub sku: Option<String>,
    pub name: Option<String>,
    pub brand: Option<String>,
    pub price: Option<f64>,
    pub currency: Option<String>,
    pub image_url: Option<String>,
    pub product_url: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, sqlx::FromRow)]
pub struct WishlistItem {
    pub id: i64,
    pub sku: Option<String>,
    pub name: Option<String>,
    pub brand: Option<String>,
    pub price: Option<f64>,
    pub currency: Option<String>,
    pub image_url: Option<String>,
    pub product_url: Option<String>,
    pub notes: Option<String>,
    pub added_at: Option<i64>,
}

pub struct WishlistRepo { pool: SqlitePool }

impl WishlistRepo {
    pub fn new(pool: SqlitePool) -> Self;
    pub async fn add(&self, input: &WishlistItemInput) -> Result<i64, sqlx::Error>;
    pub async fn remove(&self, id: i64) -> Result<(), sqlx::Error>;
    pub async fn get_all(&self) -> Result<Vec<WishlistItem>, sqlx::Error>;
}
```

### Rust: Tauri commands

```rust
#[tauri::command]
pub async fn add_to_wishlist(
    input: WishlistItemInput, state: State<'_, AppState>
) -> Result<i64, AppError>;

#[tauri::command]
pub async fn remove_from_wishlist(
    id: i64, state: State<'_, AppState>
) -> Result<(), AppError>;

#[tauri::command]
pub async fn get_wishlist(
    state: State<'_, AppState>
) -> Result<Vec<WishlistItem>, AppError>;
```

### TypeScript: WishlistStore

```typescript
// src/lib/stores/wishlist.ts
export interface WishlistStore {
  items: WishlistItem[];
  loading: boolean;
  error: string | null;
}

export const wishlistStore = writable<WishlistStore>({...});
export async function loadWishlist(): Promise<void>;
export async function addToWishlist(input: WishlistItemInput): Promise<void>;
export async function removeFromWishlist(id: number): Promise<void>;
```

### Key change: estimated_value fallback

```rust
// Before:
Ok(fallback.or(Some(0.0)))

// After — remove the .or(Some(0.0)) fallback:
Ok(fallback)
```

This makes `estimated_value("unknown-sku")` return `None` instead of `Some(0.0)`. `get_stats` already uses `unwrap_or(0.0)` to aggregate, so total values remain correct.

### Key change: image_cache watch channel (line 139-146)

```rust
// Before:
rx.wait_for(|v| v.is_some()).await.expect("in_flight watch channel closed unexpectedly");
return rx.borrow().as_ref().expect("result should be Some after wait_for").clone();

// After:
rx.wait_for(|v| v.is_some()).await
    .map_err(|_| ImageCacheError::DownloadFailed("in_flight watch channel closed unexpectedly".into()))?;
return rx.borrow().as_ref().cloned().ok_or_else(|| ImageCacheError::DownloadFailed("in_flight watch channel closed unexpectedly".into()))?;
```

### Key change: collection.ts

```typescript
// Before:
condition: 'good',
purchase_currency: product.currency || 'USD',

// After:
condition: product.condition ?? 'good',
purchase_currency: product.currency ?? 'USD',
```

## Testing Strategy

| Layer | What to Test | Approach |
|-------|-------------|----------|
| Unit | `WishlistRepo::add/remove/get_all` | In-memory SQLite, same pattern as `CollectionRepo` tests |
| Unit | `estimated_value` returns `None` for unknown SKU | Modify existing `estimated_value_zero_when_no_data` assertion |
| Unit | `estimated_value` returns `None` for null SKU | New test case |
| Unit | `get_stats` treats `None` as `0.0` | Existing test should still pass; add explicit `None` case |
| Unit | `image_cache` watch channel drop returns `Err` | New test: drop sender, verify `Err(DownloadFailed)` |
| Unit | `sync_command`/`sync.rs` `unwrap_or_default` | Existing tests already use valid timestamps — no new tests needed |
| Unit | `wishlist_command` round-trips | Test `_cmd` functions with in-memory pool |
| Integration | Wishlist add → remove → get round-trip via store | Vitest mock `invoke` |
| E2E | Wishlist page renders items, remove button works | WDIO test |

## Migration / Rollout

No migration required. The `wishlist` table already exists via migration 006. The `estimated_value` change is a semantic code change, not a schema change. Each batch is independently revertible via `git revert`.

## Open Questions

- [ ] Should the wishlist page include an "add by URL" or search-based add, or just the manual form for MVP? (Proposal scope says basic CRUD, so manual add only for now.)
- [ ] Should the nav badge update reactively on every page load, or only when the wishlist page is visited? (Recommend: load on app init in `+layout.svelte`, same as `collectionStore`.)