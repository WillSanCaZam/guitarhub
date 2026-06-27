# Proposal: Store Connections — User-Connected Store Accounts

## Intent

GuitarHub's catalog is entirely public-scraped — users selling on Reverb can't see their own listings. This change lets users connect store accounts (starting with Reverb via Personal Access Token) so their personal listings appear alongside the public catalog in a unified view.

## Scope

### In Scope
- Migration 012: `store_connections` table + nullable `user_id` on `products_meta`
- Store Registry: hardcoded list with metadata (id, name, auth type, icon, website, token URL)
- Connection Manager: Rust service — CRUD for connections, encrypted token storage, validate via Reverb API
- Reverb API client: `GET /api/my/account` (validate), `GET /api/my/listings` (paginated fetch), field mapping to RawProduct
- User Listings Sync: separate pipeline syncing user Reverb listings into `products_meta` with `user_id` set
- Frontend `/stores` page: store grid, connect/disconnect, status badges, paste-and-validate flow
- Catalog integration: home/catalog/search includes user-connected products + source badge per product

### Out of Scope (MVP)
- OAuth redirect flow (eBay, Etsy future)
- GuitarHub user accounts (no login/register — device UUID or connection ID for MVP)
- Orders, watchlist, purchases — listings only
- Store discovery recommendations

## Capabilities

### New Capabilities
- `store-connections`: Store registry + connection manager (CRUD, token validation, encrypted storage) + stores frontend page
- `user-listings-sync`: Reverb API client (auth, listings, pagination, field mapping) + sync pipeline for user-authenticated listings

### Modified Capabilities
- `product-discovery`: Queries MUST include user-connected products; optional `user_id` filter param
- `catalog-browse`: `/catalog` MUST show connected products alongside public; source/store badge per GearCard
- `search-service`: FTS5 MUST search connected products; new `store` source filter; source tag in results

## Approach

1. **Migration 012** — `store_connections(id, store_id, label, token_encrypted, username, connected_at, synced_at)` + `products_meta.user_id TEXT NULL`
2. **Store Registry** — const `&[StoreDef]` module: id, name, auth_type, icon, website, token_url
3. **Connection Manager** — Rust service: `connect/disconnect/validate/list` commands; token via `tauri-plugin-store` (AES-256-GCM, OS keyring)
4. **Reverb API client** — `reqwest`-based module with Bearer auth, paginated listing fetch, field mapper to `RawProduct`
5. **User Listings Sync** — triggered after connect or on schedule; calls Reverb API → normalizes → upserts with `user_id`
6. **Frontend** — `/stores` Svelte 5 route: responsive grid, connect flow (open browser → guide modal → paste → validate → confirm), status per store
7. **Catalog** — queries via UNION or `user_id IS NULL OR user_id = ?`; `<SourceBadge>` component per product card

## Affected Areas

| Area | Impact | Description |
|------|--------|-------------|
| `src-tauri/src/migrations/012_*` | New | store_connections table + user_id column |
| `src-tauri/src/services/store_registry.rs` | New | Static store definitions |
| `src-tauri/src/services/connection_manager.rs` | New | Token CRUD, encrypt, validate |
| `src-tauri/src/services/reverb_api.rs` | New | Reverb HTTP client |
| `src-tauri/src/services/user_sync.rs` | New | User listings sync pipeline |
| `src-tauri/src/commands/store_command.rs` | New | Tauri IPC: list/connect/disconnect/validate/sync |
| `src-tauri/src/lib.rs` | Modified | Register commands + tauri-plugin-store |
| `src-tauri/Cargo.toml` | Modified | Add tauri-plugin-store dep |
| `src/routes/stores/` | New | Stores management route |
| `src/routes/catalog/` | Modified | Filter includes connected products |
| `src/routes/+page.svelte` | Modified | Home discovery includes connected |
| `src/lib/components/` | Modified | New SourceBadge, store icons |

## Risks

| Risk | Likelihood | Mitigation |
|------|------------|------------|
| Token logged or exposed | Low | Structured logging; `Debug` impl redacts token; never log raw token |
| Reverb API rate limits | Med | Per-connection sync interval config; retry with backoff |
| No user auth — user_id semantics unclear | Med | Use store_connection.id or device UUID as user_id for MVP |
| Token storage failure | Low | Fallback to in-memory with user-facing notification |

## Rollback Plan

1. Remove `/stores` route from Svelte router
2. Unregister store Tauri commands from `lib.rs`
3. Migration revert: `ALTER TABLE products_meta DROP COLUMN user_id; DROP TABLE store_connections;`
4. Remove `tauri-plugin-store` from `Cargo.toml`
5. User sync stays dormant — no connections in DB

## Dependencies

- `tauri-plugin-store` (AES-256-GCM encrypted persistence via OS keyring)
- Reverb PAT — user-generated at `reverb.com/settings/api`
- Reverb API: `GET /api/my/account`, `GET /api/my/listings` (requires `public` + `read_listings` scopes)

## Success Criteria

- [ ] User connects Reverb via token paste → status shows "Connected as @username"
- [ ] User's Reverb listings appear in `/catalog` search and home discovery
- [ ] Product detail shows source badge ("via Reverb — your listing")
- [ ] Disconnect removes user's listings from catalog; reconnect restores them
- [ ] Public scraped products remain untouched — zero regressions
- [ ] `make test && make lint` pass

## PR Split Strategy

This exceeds the 400-line review budget. Recommended as 3 chained PRs:

- **PR 1 (Core)**: Migration 012 + Store Registry + Connection Manager + Reverb API client
- **PR 2 (Sync)**: User Listings Sync service + Tauri commands + `lib.rs` wiring
- **PR 3 (Frontend)**: `/stores` page + catalog integration + SourceBadge component
