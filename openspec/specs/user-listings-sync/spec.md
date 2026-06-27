# Capability: user-listings-sync

> **Status**: New capability  
> **Change**: store-connections

## Purpose

Fetch, normalize, and sync user-authenticated Reverb listings into `products_meta` with `user_id` set. Runs after connect and on periodic refresh. Handles pagination, API errors, and cleanup on disconnect.

## Requirements

### Requirement: Reverb API client MUST validate and fetch listings

The system MUST provide a client that calls `GET /api/my/account` (validate token) and `GET /api/my/listings` (paginated fetch, 50 per page) with `Authorization: Bearer <token>`. The client SHALL support cursor-based pagination via the `_links.next` field.

| Case | Precondition | Action | Outcome |
|------|-------------|--------|---------|
| Validate succeeds | Valid PAT | `validate("pat_xxx")` | Returns `Ok("username")` |
| Validate fails | Expired/revoked token | `validate("bad")` | Returns `AppError::InvalidInput` |
| Single page | User has 30 listings | `fetch("pat_xxx")` | Returns 30 listings, no next cursor |
| Multi-page | User has 120 listings | `fetch("pat_xxx")` | Returns all 120 across 3 pages |
| Rate limited | 100+ requests/min | `fetch("pat_xxx")` | Returns `AppError::Network` with retry-after hint |

### Requirement: Field mapping MUST convert Reverb listings to RawProduct

Each Reverb listing SHALL map to `RawProduct` fields: `sku` ← `id` (prefixed `reverb-`), `name` ← `title`, `price` ← `price.amount`, `currency` ← `price.currency`, `url` ← `_links.web.href`, `image_url` ← `photos[0]._links.small.href`.

| Case | Precondition | Action | Outcome |
|------|-------------|--------|---------|
| Complete mapping | Listing has all fields | Map to RawProduct | All fields populated, condition normalized |
| No photos | Listing has 0 photos | Map to RawProduct | `image_url` is empty string |
| Missing price | Draft listing | Map to RawProduct | `price` is 0.0, available as draft |

### Requirement: Sync pipeline MUST upsert with user_id

The sync service SHALL accept a `connection_id` and `user_id`, call the Reverb API, map listings, then upsert into `products_meta` with `user_id` set. Public-scraped products (`user_id IS NULL`) MUST remain untouched.

| Case | Precondition | Action | Outcome |
|------|-------------|--------|---------|
| Full sync | Valid token, 10 listings | `sync_user_listings(conn_id, user_id)` | 10 new rows inserted, all with `user_id = ?` |
| Re-sync | Existing user products | `sync_user_listings(conn_id, user_id)` | Existing rows updated, delisted removed |
| Public untouched | 100 public + 10 user | After sync | Public count remains 100 |

### Requirement: Disconnect MUST delete user's products

`disconnect_store` SHALL set `is_active = 0` for ALL products with matching `user_id`, preserving the rows for audit.

| Case | Precondition | Action | Outcome |
|------|-------------|--------|---------|
| Cleanup | User has 15 products | `disconnect_store("reverb")` | 15 rows set to `is_active = 0` |
| No products | User never synced | `disconnect_store("reverb")` | No rows affected |

### Requirement: Sync MUST handle API errors gracefully

Network failures, timeouts, and auth errors during sync MUST NOT corrupt existing data. Errors SHALL be logged and the sync state set to `failed_network` or `failed_schema`.

| Case | Precondition | Action | Outcome |
|------|-------------|--------|---------|
| Auth failure mid-sync | Token revoked during sync | Sync runs | Existing products preserved, sync state = `failed_network` |
| Timeout | API unresponsive > 30s | Sync runs | Timeout error logged, existing data intact |
