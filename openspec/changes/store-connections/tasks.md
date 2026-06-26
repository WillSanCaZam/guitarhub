# Tasks: Store Connections — User-Connected Store Accounts

## Review Workload Forecast

| Field | Value |
|-------|-------|
| Estimated changed lines | ~580–700 across 3 PRs |
| 400-line budget risk | High |
| Chained PRs recommended | Yes |
| Suggested split | PR 1 (Core) → PR 2 (Sync) → PR 3 (Frontend) |
| Delivery strategy | ask-on-risk |
| Chain strategy | stacked-to-main |

Decision needed before apply: Yes
Chained PRs recommended: Yes
Chain strategy: stacked-to-main
400-line budget risk: High

### Suggested Work Units

| Unit | Goal | Likely PR | Notes |
|------|------|-----------|-------|
| 1 | Core: migration, domain, registry, connection manager, encryption, Reverb client | PR 1 | Base = `store-connections` tracker branch. ~200–250 lines |
| 2 | Sync: user sync service, Tauri commands, AppState wiring, lib.rs/main.rs | PR 2 | Base = PR 1 branch. ~180–200 lines. Depends on PR 1 |
| 3 | Frontend: /stores route, StoreCard, ConnectModal, SourceBadge, catalog integration | PR 3 | Base = PR 2 branch. ~200–250 lines. Depends on PR 1 + PR 2 |

**Chain strategy**: `stacked-to-main` — each PR merges independently to main. PR2 targets main after PR1 merges, PR3 targets main after PR2 merges.

---

## PR 1 — Core (Connection Manager + Registry + Migration + Encryption)

### Phase 1: Foundation

- [x] 1.1 Create `src-tauri/src/repository/sqlite/migrations/012_store_connections.sql` — `store_connections` table + `products_meta.user_id` + index
- [x] 1.2 Create `src-tauri/src/domain/store.rs` — `StoreDef`, `Connection`, `StoreAuthType`, `EncryptedToken` (Debug redacted)
- [x] 1.3 Create `src-tauri/src/services/store_registry.rs` — `STORES: &[StoreDef]` + `fn by_id()`

### Phase 2: Core Implementation

- [x] 2.1 Modify `src-tauri/Cargo.toml` — add `aes-gcm`, `keyring`, `rand` crates
- [x] 2.2 Create `src-tauri/src/services/reverb_api.rs` — `validate_token()`, `fetch_listings()` with Bearer auth + pagination
- [x] 2.3 Create `src-tauri/src/services/connection_manager.rs` — `connect/disconnect/list/validate` with AES-256-GCM + keyring
- [x] 2.4 Modify `src-tauri/src/services/mod.rs` — register `store_registry`, `connection_manager`, `reverb_api`

### Phase 3: Testing (TDD)

- [x] 3.1 RED: Write failing test for `StoreDef` registry returns correct metadata
- [x] 3.2 GREEN: Implement registry — test passes
- [x] 3.3 RED: Write failing test for encryption round-trip + Debug redaction
- [x] 3.4 GREEN: Implement `EncryptedToken` + cipher — test passes
- [x] 3.5 RED: Write failing test for `reverb_api.validate_token` (httpmock 200/401)
- [x] 3.6 GREEN: Implement `validate_token` — test passes
- [x] 3.7 RED: Write failing test for `connection_manager` CRUD (in-memory SQLite)
- [x] 3.8 GREEN: Implement `connect/list/disconnect` — test passes

---

## PR 2 — Sync (User Listings Sync + Tauri Commands + Wiring)

### Phase 1: Core Implementation

- [ ] 4.1 Create `src-tauri/src/services/user_sync.rs` — paginated fetch → upsert with `user_id` → delist absent
- [ ] 4.2 Modify `src-tauri/src/services/mod.rs` — register `user_sync`

### Phase 2: Integration / Wiring

- [ ] 5.1 Create `src-tauri/src/commands/store_command.rs` — `connect_store`, `disconnect_store`, `list_connections`, `validate_token`, `sync_user_listings`
- [ ] 5.2 Modify `src-tauri/src/commands/mod.rs` — register `store_command`
- [ ] 5.3 Modify `src-tauri/src/lib.rs` — add `connection_manager: ConnectionManager` to `AppState`, add `ConnectionManager::new()` to `initialize_database`
- [ ] 5.4 Modify `src-tauri/src/main.rs` — register store commands in `invoke_handler`

### Phase 3: Testing (TDD)

- [ ] 6.1 RED: Write failing test for `user_sync.sync` with httpmock pagination → upserts with `user_id`
- [ ] 6.2 GREEN: Implement `user_sync.sync` — test passes
- [ ] 6.3 RED: Write failing integration test for `connect_store` → `validate` → `list` → `disconnect` lifecycle
- [ ] 6.4 GREEN: Wire commands — integration test passes

---

## PR 3 — Frontend (/stores Route + Catalog Integration + SourceBadge)

### Phase 1: Foundation

- [ ] 7.1 Create `src/lib/types/stores.ts` — `StoreDef`, `Connection`, `ConnectionStatus` TS types
- [ ] 7.2 Modify `src/lib/types/search.ts` — add `store_connection_id: string | null` to `SearchFilters`
- [ ] 7.3 Modify `src/lib/stores/filter.svelte.ts` — add `store_connection_id` to `FilterState`

### Phase 2: Components

- [ ] 8.1 Create `src/lib/components/stores/StoreIcon.svelte` — SVG icon per store
- [ ] 8.2 Create `src/lib/components/stores/StoreCard.svelte` — icon, name, status, connect/disconnect buttons
- [ ] 8.3 Create `src/lib/components/stores/StoresGrid.svelte` — responsive grid layout
- [ ] 8.4 Create `src/lib/components/stores/ConnectModal.svelte` — token input, validate, success/error states
- [ ] 8.5 Create `src/lib/components/SourceBadge.svelte` — "via Reverb — Your listing" label with icon

### Phase 3: Routes / Integration

- [ ] 9.1 Create `src/routes/stores/+page.ts` — load function: `list_connections`
- [ ] 9.2 Create `src/routes/stores/+page.svelte` — renders `StoresGrid` with `StoreCard`s
- [ ] 9.3 Modify `src/routes/+page.svelte` — pass `userId` from connections to discovery invocations
- [ ] 9.4 Modify `src/lib/components/GearCard.svelte` — add `SourceBadge` for user-connected products
- [ ] 9.5 Modify `src/routes/catalog/+page.svelte` — wire `store_connection_id` filter to `SearchPanel`
- [ ] 9.6 Modify `src/lib/components/SearchPanel.svelte` — source filter includes user product toggle

### Phase 4: Frontend Testing

- [ ] 10.1 Vitest: `StoreCard` renders all states (disconnected/connected/loading/error)
- [ ] 10.2 Vitest: `SourceBadge` shows correct label per product type (user vs public)

---

## Dependency Graph

```
PR 1 (Core)
  ├── 1.1 Migration SQL
  ├── 1.2 Domain types (StoreDef, Connection, EncryptedToken)
  ├── 1.3 Store registry
  ├── 2.1 Cargo deps
  ├── 2.2 Reverb API client (validate_token, fetch_listings)
  ├── 2.3 Connection manager (CRUD + encryption)
  ├── 2.4 Services mod registration
  └── 3.x Tests
        ↓
PR 2 (Sync)
  ├── 4.1 User sync service (uses Reverb API + ConnectionManager)
  ├── 5.1 Store Tauri commands
  ├── 5.2–5.4 Wiring (mod, lib.rs, main.rs)
  └── 6.x Tests
        ↓
PR 3 (Frontend)
  ├── 7.x Types + search filter extensions
  ├── 8.x Components (StoreCard, ConnectModal, SourceBadge)
  ├── 9.x Routes + catalog integration
  └── 10.x Frontend tests
```

**Chain rule**: PR #2 base = PR #1 branch, PR #3 base = PR #2 branch. Each child diff only shows its own delta.
