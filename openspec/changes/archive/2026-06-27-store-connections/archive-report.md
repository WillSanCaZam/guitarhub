# Archive Report: store-connections

**Archived**: 2026-06-27
**Change**: Store Connections — User-Connected Store Accounts
**Delivery**: 3 chained PRs (stacked-to-main)

## Change Summary

Allow users to connect external store accounts (starting with Reverb via Personal Access Token), manage connection lifecycle, and have personal listings appear in the unified catalog alongside public-scraped products. Token storage encrypted via AES-256-GCM + OS keyring.

### New Capabilities

- `store-connections`: Store registry + connection manager (CRUD, token validation, encryption) + `/stores` frontend page
- `user-listings-sync`: Reverb API client (auth, paginated fetch, field mapping) + sync pipeline for user-authenticated listings

### Modified Capabilities

- `product-discovery`: Three discovery commands accept optional `user_id` to include user's products
- `catalog-browse`: Catalog renders user-connected products with SourceBadge
- `search-service`: FTS5 indexes user products; `store_connection_id` filter in SearchFilters

## What Was Delivered

### PR #37 (1/3) — Core [Merged]

| Artifact | Description |
|----------|-------------|
| Migration 012 | `store_connections` table + `products_meta.user_id` column + index |
| `domain/store.rs` | `StoreDef`, `Connection`, `StoreAuthType`, `EncryptedToken` (Debug redacted) |
| `services/store_registry.rs` | `STORES: &[StoreDef]` + `fn by_id()` with Reverb entry |
| `services/reverb_api.rs` | `validate_token()`, `fetch_listings()` with Bearer auth + pagination |
| `services/connection_manager.rs` | `connect/disconnect/list/validate` with AES-256-GCM + keyring |
| Cargo.toml deps | `aes-gcm`, `keyring`, `rand` |

### PR #38 (2/3) — Sync [Merged]

| Artifact | Description |
|----------|-------------|
| `services/user_sync.rs` | Paginated fetch → upsert with `user_id` → delist absent |
| `commands/store_command.rs` | `connect_store`, `disconnect_store`, `list_connections`, `validate_token`, `sync_user_listings` |
| Wiring | `lib.rs` AppState, `main.rs` handler registration, mod files |

### PR #39 (3/3) — Frontend [Merged]

| Artifact | Description |
|----------|-------------|
| `src/routes/stores/` | Stores management route with +page.ts load function |
| Store components | `StoreIcon.svelte`, `StoreCard.svelte`, `StoresGrid.svelte` |
| `ConnectModal.svelte` | Token input, inline guide, validate, success/error states |
| `SourceBadge.svelte` | Badge showing "via Reverb — Your listing" for user products |
| Catalog integration | `GearCard` SourceBadge, `SearchPanel` store filter, home page `user_id` pass-through |
| TypeScript types | `stores.ts`, `SearchFilters.store_connection_id`, `FilterState` |

## Verification

All 3 PRs verified by orchestrator:

- [x] `make test` passes
- [x] `make lint` passes (clippy + ruff + mypy + svelte-check)
- [x] All Rust unit + integration tests pass
- [x] All frontend Vitest tests pass
- [x] No regressions in public-scraped product queries

## Specs Merged

| Domain | Action | Details |
|--------|--------|---------|
| catalog-browse | Updated | 1 ADDED (SourceBadge), 2 MODIFIED (route includes user products, FTS5 searches user products) |
| product-discovery | Updated | 3 MODIFIED (featured/drops/arrivals accept `user_id`), 1 new case for 50ms with user_id |
| search-service | Updated | 2 ADDED (FTS5 indexes user products, store connection filter), 2 MODIFIED (SearchFilters gains `store_connection_id`) |

## Source of Truth Updated

The following main specs now reflect the new behavior:
- `openspec/specs/catalog-browse/spec.md`
- `openspec/specs/product-discovery/spec.md`
- `openspec/specs/search-service/spec.md`
- `openspec/specs/store-connections/spec.md` (unchanged — was already a full spec)
- `openspec/specs/user-listings-sync/spec.md` (unchanged — was already a full spec)

## Open Items

1. **Tasks not updated**: `tasks.md` still shows unchecked `[ ]` for PR2 and PR3 tasks. The `sdd-apply` phase did not update task completion markers. This does not affect functionality — all code was delivered and verified.
2. **Verify report not generated**: No `verify-report.md` was created by `sdd-verify`. Verification was confirmed by orchestrator inline.

## Risks Discovered

None. No destructive deltas were merged. All changes are additive or backward-compatible expansions.

## Architecture Decisions Confirmed

- Token uses `aes-gcm` + `keyring` (not `tauri-plugin-store` — that crate doesn't exist)
- `user_id` = `store_connections.id` as TEXT for MVP
- FTS5 needs NO rebuild — existing triggers fire on any row mutation
