# Design: MVP Completion

## Technical Approach

Upgrade stubs to real implementations across 5 stacked PRs. Backend: replace `JsonFixtureLoader` with a `CatalogSyncService` that fetches remote JSON, runs a state machine against `sync_state` (per-source), and upserts into `products_meta`. Add `SearchService` with FTS5 query sanitization + filter/paginate/sort. Frontend: wire `+page.svelte` and `+layout.svelte` to new `search_products`/`sync_catalog` commands via direct `invoke` (matching existing pattern in `ProductCard.svelte`, `Settings.svelte`). Scraper: Python Ports & Adapters in `scraper/`, Reverb adapter, GHA cron every 6h. Repo: README, LICENSE (GPL-3.0), CHANGELOG.

The design follows Clean Architecture: Tauri commands → services → SQLite. All existing patterns preserved (AppError enum, `#[cfg(test)]` inline tests, `#[async_trait::async_trait]` traits).

## Architecture Decisions

| Decision | Option | Tradeoff | Choice |
|----------|--------|----------|--------|
| Sync state: per-source vs global | Global row vs `sync_state.source_id PK` | Global simpler but blocks parallel source syncs; per-source matches existing schema | **Per-source** — schema already has `source_id PK` |
| Delta vs full sync every time | Track last_run_id and skip unchanged | Saves bandwidth but adds complexity; MVP not multi-source yet | **Full sync always** — minimal complexity, Ok for <10MB catalog |
| Search filters: struct vs separate params | `SearchFilters` struct vs flat fn params | Struct is extensible, flat params match existing Tauri ergonomics | **Struct** (`Option<SearchFilters>`) — cleaner API for filter evolution |
| Search sanitization approach | Strip operators vs escape them | Stripping loses intent (e.g., "NOT" in brand name); escaping is safer for trigram tokenizer | **Strip + double-quote wrap** — spec mandates this for FTS5 injection prevention |
| Scraper: classes vs functions | `ReverbAdapter(ScraperPort)` class vs module-level functions | Protocol ABC enables DI and contract enforcement; functions are simpler but harder to swap | **Class per Protocol** — Ports & Adapters requirement |
| Scraper pagination strategy | Cursor-based vs offset | Reverb uses cursor; offset-based is simpler but misses items if listings change between pages | **Cursor-based** — follows Reverb API pagination |
| Frontend state: stores vs invoke | Svelte writable stores wrapping invoke vs direct invoke in components | Stores enable reactive caching; direct invoke is simpler and matches existing code | **Direct invoke** with `$state()` runes — matches `ProductCard`, `Settings` pattern |

## Data Flow

```
┌──────────────┐     ┌─────────────────────────────┐     ┌──────────────┐
│  GHA Cron    │ ──► │  scraper/reverb_adapter.py  │ ──► │  catalog.json│
│  scrape.yml  │     │  (Ports & Adapters)         │     │  (GitHub Pg) │
└──────────────┘     └─────────────────────────────┘     └──────┬───────┘
                                                                │
                     ┌──────────────────────────────────────────┘
                     ▼
┌─────────────────────────────────────────────┐
│  SyncService::sync_catalog(url)             │
│  ├─ HTTP GET → CatalogFile                  │
│  ├─ sync_state: idle→downloading→validating │
│  │    →sanitizing→inserting→done|failed     │
│  └─ UPSERT INTO products_meta               │
└─────────────────────┬───────────────────────┘
                      │
                      ▼
┌─────────────────────────────────────────────┐
│  SearchService::search_products()           │
│  ├─ Sanitize query (strip+quote)            │
│  ├─ WHERE clause from SearchFilters         │
│  ├─ ORDER BY from SortOrder                 │
│  ├─ LIMIT/OFFSET pagination                 │
│  └─ SELECT FROM products_fts JOIN meta      │
└─────────────────────┬───────────────────────┘
                      │
                      ▼
┌──────────────┐     ┌─────────────────────────────┐
│  Tauri IPC   │ ◄──│  +page.svelte / +layout       │
│  commands    │     │  (invoke → $state runes)     │
└──────────────┘     └─────────────────────────────┘
```

## File Changes

| File | Action | Description |
|------|--------|-------------|
| `src-tauri/src/services/sync.rs` | Modify | Replace `JsonFixtureLoader` with `CatalogSyncService` — HTTP fetch, state machine, upsert |
| `src-tauri/src/services/search.rs` | Create | `SearchService` trait + `FtsSearchService` impl: sanitize, filter, paginate, sort |
| `src-tauri/src/services/mod.rs` | Modify | Add `pub mod search;` |
| `src-tauri/src/commands/sync_command.rs` | Modify | Accept `url: String` instead of `path`, use `CatalogSyncService` |
| `src-tauri/src/commands/search_command.rs` | Create | `#[tauri::command] search_products` with struct params |
| `src-tauri/src/commands/mod.rs` | Modify | Add `pub mod search_command;` |
| `src-tauri/src/lib.rs` | Modify | Register `search_products` in invoke_handler |
| `src-tauri/src/domain/product.rs` | Modify | Add `SearchFilters`, `SortOrder`, `SearchResult` types |
| `src/routes/+page.svelte` | Modify | Search bar + result grid, `load more` pagination, loading/empty/error states |
| `src/routes/+layout.svelte` | Modify | Nav bar with sync button, progress indicator |
| `scraper/` | Create | Package dir: `pyproject.toml`, `requirements.txt` |
| `scraper/ports.py` | Create | `ScraperPort` Protocol ABC |
| `scraper/reverb_adapter.py` | Create | `ReverbAdapter(ScraperPort)` — HTTP + HTML/CSS parsing |
| `scraper/cli.py` | Create | `scraper --adapter reverb --output catalog.json` entry point |
| `scraper/tests/` | Create | `unit/` + `contract/` test dirs |
| `.github/workflows/scrape.yml` | Create | Cron every 6h, `pip-audit`, validate JSON schema |
| `README.md` | Create | Project desc, build/dev/test instructions, screenshot placeholder |
| `LICENSE` | Create | GPL-3.0 full text |
| `CHANGELOG.md` | Create | Keep a Changelog format, Unreleased section |

## Interfaces / Contracts

### SyncService trait (modified)

```rust
#[async_trait::async_trait]
pub trait SyncService: Send + Sync {
    async fn sync_catalog(&self, url: &str) -> Result<SyncResult, AppError>;
}

pub struct SyncResult {
    pub source_id: String,
    pub products_loaded: u32,
    pub products_updated: u32,
    pub state: SyncState,    // current state after sync
    pub progress: f32,       // 0.0–1.0
}
```

### SearchService trait (new)

```rust
#[async_trait::async_trait]
pub trait SearchService: Send + Sync {
    async fn search_products(
        &self,
        query: &str,
        filters: Option<SearchFilters>,
        sort: Option<SortOrder>,
        limit: Option<u32>,
        offset: Option<u32>,
    ) -> Result<SearchResult, AppError>;
}

pub struct SearchFilters {
    pub category: Option<String>,
    pub price_min: Option<f64>,
    pub price_max: Option<f64>,
    pub source: Option<String>,
}

pub enum SortOrder { Relevance, PriceAsc, PriceDesc, NameAsc, NameDesc }

pub struct SearchResult {
    pub products: Vec<RawProduct>,
    pub total: u64,
    pub offset: u32,
    pub limit: u32,
}
```

### ScraperPort Protocol (new)

```python
class ScraperPort(Protocol):
    def scrape(self, url: str) -> CatalogFile: ...
```

## Testing Strategy

| Layer | What | Approach |
|-------|------|----------|
| Unit (Rust) | `CatalogSyncService` state transitions, upsert counting, error cases | In-memory SQLite, mock HTTP with `wiremock` or trait abstraction |
| Unit (Rust) | `FtsSearchService` sanitization, filtering, sorting, pagination | In-memory SQLite with seeded products_meta, test all SortOrder variants |
| Unit (Rust) | Tauri command serialization/deserialization | Test `#[tauri::command]` fn signatures compile with `SearchFilters`/`SortOrder` |
| Unit (Python) | `ReverbAdapter` data extraction | Local HTML fixtures, verify `CatalogProduct` field mapping |
| Contract (Python) | Adapter conforms to `ScraperPort` Protocol | `mypy --strict` + `pytest` protocol conformance test |
| Integration (Rust) | Full sync-then-search pipeline | Sync fixture JSON, search by name/category/price, verify round-trip |
| Integration (Python) | CLI end-to-end | `scraper --adapter test --output /tmp/out.json`, validate output schema |

## Migration / Rollout

5 stacked PRs per proposal. Chain: PR #1 (SyncService) → PR #2 (SearchService) → PR #3 (Frontend) → PR #4 (Scraper) → PR #5 (Repo docs). Each targets the feature tracker branch. No data migration needed — `sync_state` entries are created fresh on first sync.

## Open Questions

- [ ] What is the actual GitHub Pages URL for catalog hosting? (needed for default URL in frontend sync button and GHA scrape.yml publish step)
- [ ] Should `CatalogSyncService` implement retry with backoff for HTTP 5xx? (spec doesn't mention, but scraper spec does)
