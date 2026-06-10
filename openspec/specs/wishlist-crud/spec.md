# Wishlist CRUD Specification

> **Status**: New capability  
> **Change**: mvp-fixes — Batch 4

## Purpose

Provide add, remove, and list operations for the user's wishlist via Tauri commands, backed by the existing `wishlist` table (migration 006). Expose these operations to the Svelte frontend through a reactive store and render a wishlist page in the navigation.

## Requirements

### Requirement: WishlistRepo MUST provide add, remove, and list operations

The `WishlistRepo` struct MUST use the `wishlist` table (already created by migration 006 with columns: `id`, `sku`, `name`, `brand`, `price`, `currency`, `image_url`, `product_url`, `notes`, `added_at`) and implement:

- `add(input: &WishlistItemInput) -> Result<i64>` — insert a row, return autoincrement `id`
- `remove(id: i64) -> Result<()>` — delete by `id`
- `get_all() -> Result<Vec<WishlistItem>>` — return all items ordered by `added_at DESC`

#### Scenario: Add wishlist item

- GIVEN `addToCollection` is NOT called (wishlist only)
- WHEN `add` is called with `WishlistItemInput { sku: "FENDER-TELE", name: "Telecaster", brand: "Fender", price: 1500.0, currency: "USD", image_url: "https://example.com/img.jpg", product_url: None, notes: None }`
- THEN a row is inserted into `wishlist` and the returned `id` is a positive integer

#### Scenario: Remove wishlist item

- GIVEN `wishlist` has a row with `id = 5`
- WHEN `remove(5)` is called
- THEN the row is deleted
- AND subsequent `get_all` excludes it

#### Scenario: List all wishlist items

- GIVEN 3 rows exist in `wishlist`
- WHEN `get_all()` is called
- THEN it returns a vec of 3 `WishlistItem` structs ordered by `added_at DESC`

#### Scenario: Empty wishlist returns empty vec

- GIVEN `wishlist` has zero rows
- WHEN `get_all()` is called
- THEN it returns an empty vec (NOT an error)

### Requirement: Wishlist Tauri commands MUST be registered

The following `#[tauri::command]` functions MUST be registered in the app's `invoke_handler`:

- `add_to_wishlist(input: WishlistItemInput, state: State<AppState>) -> Result<i64, AppError>`
- `remove_from_wishlist(id: i64, state: State<AppState>) -> Result<(), AppError>`
- `get_wishlist(state: State<AppState>) -> Result<Vec<WishlistItem>, AppError>`

Each command MUST delegate to `WishlistRepo` methods on `state.pool`.

#### Scenario: add_to_wishlist IPC round-trip

- GIVEN the Tauri app is running
- WHEN the frontend calls `invoke('add_to_wishlist', { input: { sku: 'X', name: 'Guitar', ... } })`
- THEN a new row appears in `wishlist` and the command returns the new `id`

#### Scenario: remove_from_wishlist IPC round-trip

- GIVEN a wishlist item with `id = 1` exists
- WHEN the frontend calls `invoke('remove_from_wishlist', { id: 1 })`
- THEN the row is deleted and the command returns `Ok(())`

#### Scenario: get_wishlist IPC round-trip

- GIVEN 2 items exist in `wishlist`
- WHEN the frontend calls `invoke('get_wishlist')`
- THEN it returns an array of 2 `WishlistItem` objects sorted by `added_at` descending

### Requirement: Frontend wishlist store MUST wrap all three operations

A new `src/lib/stores/wishlist.ts` MUST export a Svelte writable store with `loadWishlist`, `addToWishlist`, and `removeFromWishlist` functions. Each function MUST call the corresponding Tauri command and update the store state.

#### Scenario: addToWishlist updates store

- GIVEN `wishlistStore` has 0 items
- WHEN `addToWishlist({ sku: 'X', name: 'Guitar', brand: 'Y', price: 100 })` is called
- THEN `invoke('add_to_wishlist', ...)` is called
- AND `wishlistStore` items are refreshed to include the new item

#### Scenario: removeFromWishlist updates store

- GIVEN `wishlistStore` has 2 items
- WHEN `removeFromWishlist(1)` is called
- THEN `invoke('remove_from_wishlist', { id: 1 })` is called
- AND `wishlistStore` items are refreshed to exclude item `1`

### Requirement: TypeScript types for wishlist MUST be defined

A new `src/lib/types/wishlist.ts` MUST export `WishlistItem` and `WishlistItemInput` interfaces matching the Rust structs. `WishlistItem` MUST include all columns from the `wishlist` table. `WishlistItemInput` MUST include the fields callers provide (sku, name, brand, price, currency, image_url, product_url, notes) — `id` and `added_at` are server-generated.

#### Scenario: WishlistItem type shape

- GIVEN `WishlistItem` is imported
- WHEN used in TypeScript strict mode
- THEN it compiles with fields: `id: number`, `sku: string | null`, `name: string | null`, `brand: string | null`, `price: number | null`, `currency: string | null`, `image_url: string | null`, `product_url: string | null`, `notes: string | null`, `added_at: number | null`

#### Scenario: WishlistItemInput type shape

- GIVEN `WishlistItemInput` is imported
- WHEN used in TypeScript strict mode
- THEN it compiles with the input fields — all optional where appropriate, no `id` or `added_at`

### Requirement: Wishlist page MUST be wired into navigation

A new route at `src/routes/wishlist/+page.svelte` MUST render the wishlist items from `wishlistStore`. The navigation sidebar MUST include a link to `/wishlist` with a count badge showing `wishlistStore.items.length`.

#### Scenario: Navigation shows wishlist link with count

- GIVEN 3 items exist in the wishlist
- WHEN the user views the sidebar
- THEN a "Wishlist" link is visible pointing to `/wishlist`
- AND a badge shows "3"

#### Scenario: Empty wishlist shows empty state

- GIVEN 0 items exist in the wishlist
- WHEN the user navigates to `/wishlist`
- THEN the page renders an empty state message (e.g., "Your wishlist is empty")

#### Scenario: Wishlist displays items

- GIVEN 2 items exist in the wishlist
- WHEN the user navigates to `/wishlist`
- THEN the page renders 2 wishlist items with name, brand, price, and a remove button