# Design: Local Image Cache

## Technical Approach

SQLite BLOB-based image cache with LRU eviction and TTL. Images are stored as raw bytes in an `image_cache` table, accessed via URL hash keys, and served as base64 data URIs to the Svelte frontend. Concurrent requests for the same URL are coalesced into a single HTTP fetch via a shared `oneshot` channel map.

## Architecture Decisions

### Decision: Storage backend

| Option | Tradeoffs | Verdict |
|--------|-----------|---------|
| **SQLite BLOBs** | Single file backup; atomic eviction via SQL; no path/permission management | **Chosen** |
| Filesystem + Tauri asset scope | Complex path handling per-OS; eviction requires dir scan; Tauri 2 asset protocol scope must be declared upfront | Rejected |

**Rationale**: SQLite gives us transactional eviction, a single backup unit, and zero filesystem permission concerns. The spec explicitly excludes "filesystem-level image cache" from scope.

### Decision: Cache key = URL hash

**Choice**: SHA-256 of `image_url`. **Alternatives**: SKU-based key, composite key. **Rationale**: Multiple products can share the same image URL (brand logos, store banners). URL is the invariant — if the URL changes, the old entry is an orphan that TTL will clean.

### Decision: Return base64 data URIs vs file paths

**Choice**: Tauri command returns `data:image/<mime>;base64,...` string. **Alternatives**: Write to temp dir + `convertFileSrc`, return file URL. **Rationale**: Sticking with SQLite BLOBs — no extra filesystem write. Base64 overhead (~33%) is acceptable for typical 50-200KB product images. Can be optimized to asset protocol in a follow-up if list rendering perf becomes an issue.

### Decision: DashMap for request coalescing

**Choice**: `dashmap::DashMap<String, oneshot::Sender<Result>>` to deduplicate in-flight requests. **Alternatives**: `tokio::sync::RwLock<HashMap>`. **Rationale**: DashMap is already a common transitive dep in Tauri projects, no contention on a single lock, and the sharded design is ideal for concurrent image requests from a scrolling list.

## Components

```
src-tauri/src/
├── repository/sqlite/
│   ├── mod.rs                       [MODIFIED] — add image_cache module
│   └── image_cache.rs               [CREATED] — SQL CRUD + eviction queries
├── services/image_cache.rs          [CREATED] — ImageCacheService
├── commands/image_command.rs        [CREATED] — Tauri IPC glue
└── lib.rs                           [MODIFIED] — wire service into AppState
```

## Interfaces

```rust
// services/image_cache.rs
pub struct ImageCacheService {
    pool: sqlx::SqlitePool,
    max_bytes: u64,               // 50 MB default
    default_ttl: Duration,        // 7 days
    http: reqwest::Client,
    in_flight: Arc<DashMap<String, oneshot::Sender<Result<Vec<u8>, Error>>>>,
    failed_urls: Arc<tokio::sync::RwLock<HashSet<String>>>,
}

impl ImageCacheService {
    pub fn new(pool: SqlitePool, max_bytes: u64, default_ttl: Duration) -> Self;

    /// Core method: cached blob → return, miss → fetch → store → return.
    /// Coalesces concurrent requests for the same url.
    /// Stale entries → re-fetch in foreground, fallback to stale on network error.
    pub async fn get(&self, url: &str) -> Result<Vec<u8>>;
}

#[derive(Debug, thiserror::Error)]
pub enum ImageCacheError {
    #[error("invalid image URL")]
    InvalidUrl,
    #[error("download failed: {0}")]
    DownloadFailed(#[from] reqwest::Error),
    #[error("placeholder (download failed, no cache)")]
    Placeholder,
}

// commands/image_command.rs
#[tauri::command]
pub async fn get_product_image(
    image_url: String,
    state: tauri::State<'_, AppState>,
) -> Result<String, String> {
    let bytes = state.image_cache.get(&image_url).await.map_err(|e| e.to_string())?;
    // Detect mime from bytes or default to jpeg
    let mime = infer_mime_type(&bytes);
    Ok(format!("data:{};base64,{}", mime, BASE64.encode(bytes)))
}
```

## SQLite Schema (migration `003_add_image_cache.sql`)

```sql
CREATE TABLE image_cache (
    url_hash      TEXT PRIMARY KEY,
    image_url     TEXT NOT NULL,
    blob          BLOB NOT NULL,
    mime_type     TEXT NOT NULL DEFAULT 'image/jpeg',
    size_bytes    INTEGER NOT NULL,
    created_at    INTEGER NOT NULL,
    last_accessed INTEGER NOT NULL,
    ttl_seconds   INTEGER NOT NULL DEFAULT 604800
);

CREATE INDEX idx_image_cache_last_accessed ON image_cache(last_accessed);
```

## Data Flow

```
Svelte: <ProductCard image_url="https://img.reverb.com/x.jpg">
  │
  └── invoke("get_product_image", { imageUrl })
         │
         ▼
  ImageCacheService::get(url)
    │
    ├─ 1. hash = sha256(url)
    │
    ├─ 2. in_flight.get(hash) ?  ← DashMap check
    │     YES → await existing receiver (coalesce, no new HTTP)
    │     NO  → insert new oneshot channel
    │
    ├─ 3. SELECT blob WHERE url_hash = ?
    │     │
    │     ├─ MISS → HTTP GET url ──── success → INSERT blob, enforce_size_limit()
    │     │         ↓ fail OR 404     failure → failed_urls.add(url); return PLACEHOLDER
    │     │
    │     ├─ HIT + FRESH (now - last_accessed < TTL)
    │     │     → UPDATE last_accessed = now
    │     │     → return blob
    │     │
    │     └─ HIT + STALE
    │           → HTTP GET url ──── success → UPDATE blob + last_accessed
    │           ↓ fail (offline)    failure → UPDATE last_accessed (touch)
    │           → return (possibly stale) blob
    │
    └─ 4. remove oneshot channel from in_flight (complete)
```

## Eviction Rules

- **On insert**: `SELECT COALESCE(SUM(size_bytes), 0) FROM image_cache` → if total > `max_bytes`, `DELETE FROM image_cache ORDER BY last_accessed ASC LIMIT N` where N = enough rows to fit new blob
- **Oversized image** (single entry > `max_bytes`): skip cache, serve in-memory only
- **TTL**: checked on access, not background. Stale + online → re-fetch; stale + offline → serve stale

## Concurrency

- `in_flight: DashMap` — insert before HTTP fetch, remove after. Other requests for the same URL wait on the same `oneshot::Receiver`.
- `failed_urls: RwLock<HashSet>` — checked before HTTP to skip known-404 URLs; low contention.
- SQLite writes are serialized by `sqlx` pool — no concurrent INSERT/EVICT race (eviction runs inside the `get()` hot path, locked per-connection).

## Security

- Image URLs already sanitized to `https://` in `SyncService::map_and_sanitize_vec()`.
- MIME type validated on download: reject non-image types (`text/html`, `application/xml`) before storage.
- Blobs stored as-is — no SVG parsing (prevents XSS vector in `<img>`).
- CSP `img-src https: data:` covers both remote URLs and base64 data URIs.

## Testing Strategy

| Layer | What | Approach |
|-------|------|----------|
| Unit | Cache hit returns bytes | Insert blob, call `get()` → same bytes, no HTTP call |
| Unit | Cache miss fetches + stores | Mock HTTP 200 → verify INSERT happened |
| Unit | LRU eviction order | Insert 3 entries (30 MB total), set limit 20 MB, access middle → verify oldest evicted |
| Unit | Concurrent coalesce | 10 concurrent tokio tasks for same URL → count HTTP calls = 1 |
| Unit | Failed URL dedup | Request 404 twice → only 1 HTTP call |
| Unit | Stale served offline | Set TTL=0, mock HTTP fail → stale blob returned |
| Integration | Persistence across restart | Insert blob, recreate service, `get()` → same bytes |
| Integration | Size limit enforcement | Insert 60 MB with 50 MB limit → verify SUM ≤ 50 MB |

## Implementation Order

1. Create `003_add_image_cache.sql` migration
2. Create `repository/sqlite/image_cache.rs` — raw SQL CRUD
3. Create `services/image_cache.rs` — `ImageCacheService` with coalescing + eviction
4. Create `commands/image_command.rs` — Tauri command
5. Wire into `AppState` in `lib.rs`
6. Update Svelte `ProductCard.svelte` to call `get_product_image` instead of direct `<img src>`
7. Write tests
