# Design: Store Connections — User-Connected Store Accounts

## Technical Approach

Add a second sync pipeline: user-authenticated Reverb API calls feed into the same `products_meta` table, distinguished by a new nullable `user_id` column (value = `store_connections.id`). A hardcoded **Store Registry** defines supported stores; a **Connection Manager** service handles encrypted token CRUD; a **Reverb API client** + **User Listings Sync** service fetches and normalizes user listings. Public scraped products (`user_id IS NULL`) are untouched.

```
┌──────────────────────────────────────────────────┐
│                   Frontend                         │
│  /stores  ─── StoreCard × N                       │
│  /catalog ─── GearCard + SourceBadge              │
└────────┬────────────┬─────────────────────────────┘
         │ invoke     │ search_products
    ┌────▼────┐  ┌────▼──────────────────┐
    │ store_  │  │ search_command        │
    │ command │  │ product_command       │
    └────┬────┘  └────┬──────────────────┘
         │             │
    ┌────▼────────────▼──────────────────┐
    │         Services                    │
    │  store_registry ── const &[StoreDef]│
    │  connection_manager ── CRUD + crypt │
    │  reverb_api ── HTTP client          │
    │  user_sync ── paginated fetch/upsert│
    │  product_query ── +user_id filter   │
    │  search ── FTS5 +user_id filter     │
    └────┬────────────────────┬───────────┘
         │                    │
    ┌────▼────┐         ┌────▼──────────┐
    │ SQLite  │         │ Reverb API    │
    │ 012 mig │         │ GET /my/      │
    └─────────┘         └───────────────┘
```

## Architecture Decisions

| Decision | Choice | Alternatives | Rationale |
|----------|--------|-------------|-----------|
| Token encryption | `aes-gcm` + `keyring` crate | `tauri-plugin-store` (nonexistent), plaintext SQLite | AES-256-GCM via `aes-gcm`, master key via OS keyring (`keyring` crate). Library for `tauri-plugin-store` is a misnomer in ADR — no such plugin exists. |
| Store Registry | const `&[StoreDef]` module | Settings table, JSON config | Zero runtime cost, type-safe, trivially extensible. Future stores just add a const entry. |
| user_id value | `store_connections.id` as TEXT | Device UUID, separate users table | No user accounts for MVP; connection IS the user context. |
| Catalog integration | `WHERE (user_id IS NULL OR user_id = ?)` param | UNION with separate table | Minimal query change, no new FTS5 rebuild, existing queries untouched when param omitted. |
| PR split | 3 chained PRs | Monolithic PR >400 lines | Review budget: PR1 Core (~200), PR2 Sync (~180), PR3 Frontend (~200) |

## Spec Discrepancies

- **Token storage**: ADR/proposal reference `tauri-plugin-store` which does not exist. Using `aes-gcm` + `keyring` crate instead.
- **user_id semantics**: Proposal says "store_connection.id or device UUID" — spec says store_connection.id. Using connection id (TEXT) for MVP clarity.
- **FTS5 index rebuild**: Spec says FTS5 MUST index user products; migration 002 creates FTS5 triggers on INSERT/UPDATE/DELETE. Adding `user_id` column to `products_meta` does NOT require FTS5 schema change — existing triggers already fire for any row mutation. No rebuild needed.

## Migration 012 SQL

```sql
-- 012_store_connections.sql
CREATE TABLE IF NOT EXISTS store_connections (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    store_id        TEXT NOT NULL,
    label           TEXT NOT NULL DEFAULT '',
    token_encrypted TEXT NOT NULL,
    username        TEXT,
    connected_at    INTEGER NOT NULL,
    synced_at       INTEGER,
    is_active       INTEGER DEFAULT 1,
    UNIQUE(store_id)
);

ALTER TABLE products_meta ADD COLUMN user_id TEXT;
CREATE INDEX idx_products_meta_user_id ON products_meta(user_id);
```

## Component Breakdown Per PR

### PR 1 — Core (Connection Manager + Registry + Migration + Encryption)

| File | Action |
|------|--------|
| `src-tauri/src/repository/sqlite/migrations/012_store_connections.sql` | Create |
| `src-tauri/src/domain/store.rs` | Create — `StoreDef`, `Connection`, `StoreAuthType` |
| `src-tauri/src/services/store_registry.rs` | Create — `fn stores() -> &'static [StoreDef]`, `fn by_id(id) -> Option` |
| `src-tauri/src/services/connection_manager.rs` | Create — `store()` |
| `src-tauri/src/services/reverb_api.rs` | Create — `validate_token()`, `fetch_listings()` |
| `src-tauri/src/services/mod.rs` | Modify — register modules |
| `src-tauri/Cargo.toml` | Modify — add `aes-gcm`, `keyring`, `rand` |

```rust
// domain/store.rs
#[derive(Debug, Clone, serde::Serialize)]
pub struct StoreDef {
    pub id: &'static str,       // "reverb"
    pub name: &'static str,     // "Reverb"
    pub auth_type: StoreAuthType,
    pub icon: &'static str,     // svg path or icon name
    pub website: &'static str,  // "https://reverb.com"
    pub token_url: &'static str,// "https://reverb.com/settings/api"
}

#[derive(Debug, Clone, serde::Serialize)]
pub enum StoreAuthType { Pat }

#[derive(Debug, Clone, serde::Serialize)]
pub struct Connection {
    pub id: i64,
    pub store_id: String,
    pub label: String,
    pub username: Option<String>,
    pub connected_at: i64,
    pub synced_at: Option<i64>,
    pub is_active: bool,
}

// Token is NEVER serialized in Debug or IPC responses
#[derive(Clone)]
pub struct EncryptedToken(Vec<u8>);

impl std::fmt::Debug for EncryptedToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "EncryptedToken(REDACTED)")
    }
}
```

```rust
// services/store_registry.rs
pub static STORES: &[StoreDef] = &[StoreDef {
    id: "reverb",
    name: "Reverb",
    auth_type: StoreAuthType::Pat,
    icon: "reverb",
    website: "https://reverb.com",
    token_url: "https://reverb.com/settings/api",
}];
```

```rust
// services/connection_manager.rs
pub struct ConnectionManager {
    pool: SqlitePool,
    cipher: Aes256Gcm, // key derived from keyring
}

impl ConnectionManager {
    pub fn new(pool: SqlitePool) -> Self;

    /// Validate token via Reverb API, encrypt, store connection.
    /// Returns Connection with username from account check.
    pub async fn connect(&self, store_id: &str, token: &str) -> Result<Connection, AppError>;

    /// Remove connection + soft-delete user's products.
    pub async fn disconnect(&self, store_id: &str) -> Result<(), AppError>;

    /// Return all connections (token_encrypted NEVER included).
    pub async fn list(&self) -> Result<Vec<Connection>, AppError>;

    /// Validate token by calling Reverb API. Pure passthrough to reverb_api.
    pub async fn validate_token(&self, store_id: &str, token: &str) -> Result<String, AppError>;

    /// Decrypt token for sync use. Internal method.
    async fn decrypt_token(&self, conn_id: i64) -> Result<String, AppError>;
}
```

### PR 2 — Sync (User Listings Sync + Tauri Commands + Wiring)

| File | Action |
|------|--------|
| `src-tauri/src/services/user_sync.rs` | Create |
| `src-tauri/src/commands/store_command.rs` | Create |
| `src-tauri/src/commands/mod.rs` | Modify — add `store_command` |
| `src-tauri/src/services/mod.rs` | Modify — add `user_sync` |
| `src-tauri/src/lib.rs` | Modify — register commands + AppState fields |
| `src-tauri/src/main.rs` | Modify — register handlers |

**Reverb API — endpoints:**
```
GET /api/my/account
  Auth: Bearer <token>
  Response: { "shop": { "name": "@username" }, "email": "..." }

GET /api/my/listings?page=1&per=50
  Auth: Bearer <token>
  Response: {
    "listings": [{ "id": 123, "title": "Fender Strat", ... }],
    "_links": { "next": { "href": "..." } }
  }
```

**Field mapping (Reverb listing → RawProduct):**
```rust
RawProduct {
    sku: format!("reverb-user-{}", listing.id),
    name: listing.title,
    price: listing.price.amount,
    currency: listing.price.currency,
    url: listing._links.web.href,
    image_url: listing.photos.first().and_then(|p| p._links.small.href).unwrap_or(""),
    condition: normalize_condition(&listing.condition),
    // other fields from defaults
}
```

```rust
// commands/store_command.rs
#[tauri::command]
pub async fn connect_store(
    store_id: String,
    token: String,
    state: State<'_, AppState>,
) -> Result<Connection, AppError>;

#[tauri::command]
pub async fn disconnect_store(
    store_id: String,
    state: State<'_, AppState>,
) -> Result<(), AppError>;

#[tauri::command]
pub async fn list_connections(
    state: State<'_, AppState>,
) -> Result<Vec<Connection>, AppError>;

#[tauri::command]
pub async fn validate_token(
    store_id: String,
    token: String,
    state: State<'_, AppState>,
) -> Result<String, AppError>;

#[tauri::command]
pub async fn sync_user_listings(
    store_id: String,
    state: State<'_, AppState>,
) -> Result<SyncResult, AppError>;
```

```rust
// services/user_sync.rs — follows CatalogSyncService pattern
pub struct UserSyncService {
    pool: SqlitePool,
    http_client: reqwest::Client,
    connection_manager: ConnectionManager,
}

impl UserSyncService {
    /// Fetch all paginated listings → upsert with user_id → delist absent
    pub async fn sync(&self, conn: &Connection) -> Result<SyncResult, AppError>;
}
```

**Data flow — Connect:**
```
"Connect" click → open token_url in browser → user pastes token
  → validate_token("reverb", token)
    → GET /api/my/account (returns @username)
    → if 401: AppError::InvalidInput("token invalid")
    → if 200: OK(@username)
  → connect_store("reverb", token)
    → encrypt token (AES-256-GCM via keyring key)
    → INSERT into store_connections
    → auto-trigger sync_user_listings
      → GET /api/my/listings (paginated)
      → upsert_products with user_id = conn.id
      → return SyncResult
```

**Data flow — Disconnect:**
```
disconnect_store("reverb")
  → DELETE FROM store_connections WHERE store_id = ?
  → UPDATE products_meta SET is_active=0 WHERE user_id = ?
  → return Ok(())
```

**AppState additions:**
```rust
pub struct AppState {
    pub pool: SqlitePool,
    pub image_cache_service: ImageCacheService,
    pub http_client: reqwest::Client,
    pub product_query: ProductQueryService,
    pub connection_manager: ConnectionManager,  // NEW
}
```

### PR 3 — Frontend (/stores Route + Catalog Integration + SourceBadge)

| File | Action |
|------|--------|
| `src/routes/stores/+page.svelte` | Create |
| `src/routes/stores/+page.ts` | Create — load function |
| `src/lib/components/stores/StoreCard.svelte` | Create |
| `src/lib/components/stores/ConnectModal.svelte` | Create |
| `src/lib/components/stores/StoresGrid.svelte` | Create |
| `src/lib/components/SourceBadge.svelte` | Create |
| `src/lib/components/discovery/StoreIcon.svelte` | Create |
| `src/lib/types/stores.ts` | Create |
| `src/routes/catalog/+page.svelte` | Modify — pass user_id to SearchPanel |
| `src/routes/+page.svelte` | Modify — pass user_id to discovery invocations |
| `src/lib/components/GearCard.svelte` | Modify — add SourceBadge |
| `src/lib/components/SearchPanel.svelte` | Modify — source filter includes user |

```
Component tree (/stores):
+page.svelte
  └─ StoresGrid
       └─ StoreCard × N
            ├─ store icon + name + status badge
            ├─ "Connect" button → ConnectModal
            └─ "Disconnect" button → confirm

ConnectModal:
  Step 1: "Open Reverb settings" → opens browser
  Step 2: Inline guide with numbered steps
  Step 3: Token input field (type=password)
  Step 4: Validate button → calls validate_token
  Step 5: Success → "Connected as @username" / Error → inline error
```

**Catalog integration — user_id param:**
```typescript
// All discovery commands now accept optional user_id
const featured = await invoke('get_featured_products', {
    limit: 6,
    userId: connections.length > 0 ? connections[0].id : null,
});
```

**SearchFilters additions:**
```typescript
export interface SearchFilters {
  // ...existing fields...
  store_connection_id: string | null; // NEW — "public" excludes user products
}
```

## Error Handling

New `AppError` variants:

| Variant | When | Message |
|---------|------|---------|
| `TokenInvalid` | Validated token returns 401 | "token invalid" |
| `TokenStorage` | Keyring/AES failure | "failed to store token" |
| `StoreNotFound` | Unknown store_id in registry | "unknown store: {id}" |
| `ConnectionExists` | Already connected (upsert flow) | (handled as re-connect) |
| `RateLimited` | 429 from Reverb | "rate limited, retry in {secs}s" |

## Testing Strategy

| Layer | What | Approach |
|-------|------|----------|
| Unit — registry | All store defs return correct metadata | Pure fn tests |
| Unit — encrypt | Round-trip encrypt→decrypt; Debug redacts | AES-GCM with test key |
| Unit — reverb_api | Validate + fetch_listings with httpmock | Mocked HTTP responses |
| Unit — connection_manager | CRUD via in-memory SQLite | Test pool with 012 schema |
| Unit — user_sync | Paginated fetch → upsert with user_id | httpmock + in-memory pool |
| Unit — product_query | user_id filter returns correct subset | In-memory pool tests |
| Integration — command | connect→validate→list→disconnect lifecycle | Full in-memory DB |
| Frontend — StoreCard | Renders all states (disconnected/connected/loading/error) | Vitest + svelte-testing |
| Frontend — SourceBadge | Shows correct label per product type | Vitest |

## Key Learnings

- FTS5 index needs NO rebuild for `user_id` column — existing insert/update/delete triggers fire for any row mutation covering the indexed columns (sku, name, etc.). The `user_id` column is NOT in the FTS5 index — it's filtered at the `products_meta` WHERE clause during the JOIN.
- `tauri-plugin-store` does not exist as a crate — using `aes-gcm` + `keyring` crates instead. This is a design correction from the ADR.
- The existing `ProductQueryRow` struct in `product_query.rs` does NOT include `user_id` — need to add it for the SourceBadge to know whether a product is user-connected. Alternatively, derive it by checking `user_id IS NOT NULL` in the query.

## Open Questions

- [ ] Should `user_id` be added to the `ProductQueryRow` / `RawProduct` struct so frontend can distinguish "your listing" from public?
- [ ] Rate limit handling: retry with backoff or surface to user?
- [ ] Should we add a `source_store` field to `SearchResult` or derive it from `source_id` on frontend?
