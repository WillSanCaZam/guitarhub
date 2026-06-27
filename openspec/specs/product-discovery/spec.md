# Capability: product-discovery

> **Status**: New capability  
> **Change**: product-display-pipeline

## Purpose

Provide three Tauri IPC commands (`get_featured_products`, `get_price_drops`, `get_new_arrivals`) that supply product listing data for the home page discovery feed. All are read-only, query only active products from the SQLite catalog, and MUST return within 50ms.

## Requirements

### Requirement: get_featured_products MUST return N random active products

`get_featured_products(limit: u32, user_id: Option<String>) -> Vec<RawProduct>` — `SELECT ... FROM products_meta WHERE is_active=1 AND (user_id IS NULL OR user_id = ?) ORDER BY RANDOM() LIMIT ?`. The `user_id` parameter is OPTIONAL; when omitted, returns only public products. When provided, includes the user's connected products alongside public ones.

| Case | Precondition | Action | Outcome |
|------|-------------|--------|---------|
| Returns N random public | 50 public active products | `limit=6` | Returns 6 distinct public products |
| Includes user products | 8 user-owned products | `limit=6, user_id="u1"` | May include user products + public products |
| Less than N | 2 public + 3 user products | `limit=6, user_id="u1"` | Returns 5 products |
| Empty catalog | 0 products | invoke | Returns `[]` |
| Frontend renders | 6 products returned | Home page mounts | Renders GearCard grid in "Because You Viewed" section |

### Requirement: get_price_drops MUST return products with largest absolute price drops

`get_price_drops(limit: u32, user_id: Option<String>) -> Vec<RawProduct>` — JOIN `products_meta` with `price_history` subquery, includes public AND user-connected products when `user_id` is provided. Only active products whose current price is strictly less than their first recorded price are included. The query SHALL use the `price_history.sku` index for performance and SHALL be capped at 50 results to mitigate cost.

| Case | Precondition | Action | Outcome |
|------|-------------|--------|---------|
| Biggest drop first | sku-1 dropped $500, sku-2 dropped $200 | invoke with `limit=5` | sku-1 returned before sku-2 |
| Includes user drops | User product dropped $500 | `limit=5, user_id="u1"` | User product in results among public drops |
| No user_id | User product dropped $500 | `limit=5, no user_id` | Only public drops returned |
| No price history | Active product with zero rows in `price_history` | invoke | Product excluded from results |
| No drops exist | All current prices ≥ first recorded | invoke | Returns `[]` |
| Empty catalog | 0 products | invoke | Returns `[]` |
| Frontend renders | 3 drops returned | Home page mounts | Renders GearCard grid in "Price Drops" section |

### Requirement: get_new_arrivals MUST return most recently synced products

`get_new_arrivals(limit: u32, user_id: Option<String>) -> Vec<RawProduct>` — `WHERE is_active=1 AND (user_id IS NULL OR user_id = ?) ORDER BY synced_at DESC LIMIT ?`.

| Case | Precondition | Action | Outcome |
|------|-------------|--------|---------|
| User products first | User's products synced recently | `limit=6, user_id="u1"` | User's newest products appear in results |
| Public only | No user_id given | `limit=6` | Only public products returned |
| Newest first | 20 active products, sku-3 is most recent | invoke with `limit=6` | sku-3 is first, exactly 6 returned |
| Less than N | 2 active products | invoke with `limit=6` | Returns 2 |
| Empty catalog | 0 products | invoke | Returns `[]` |
| Frontend renders | 4 new arrivals | Home page mounts | Renders GearCard grid in "New Arrivals" section |

### Requirement: All discovery commands MUST handle concurrent calls safely

Concurrent invocations via `Promise.all` MUST NOT cause deadlocks or data races.

| Case | Precondition | Action | Outcome |
|------|-------------|--------|---------|
| Concurrent invoke | 100 active products | `Promise.all([featured, drops, arrivals])` | All 3 resolve without error |

### Requirement: Discovery commands MUST return within 50ms

Each command MUST complete within 50ms on local SQLite with warm cache and up to 10,000 products.

| Case | Precondition | Action | Outcome |
|------|-------------|--------|---------|
| Under limit | 10k products, warm cache | Invoke any command | Completes in < 50ms |
| Under limit with user | 10k products + 1k user products | Invoke any command with `user_id` | Completes in < 50ms |
| DB unavailable | Connection lost | Invoke any command | Returns `AppError::Database` with user-friendly message |
