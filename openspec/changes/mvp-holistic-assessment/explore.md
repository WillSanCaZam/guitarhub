## Exploration: Holistic MVP Assessment of GuitarHub

### Current State

GuitarHub is a **cross-platform desktop app** (Tauri 2 + Svelte 5 + Rust) for aggregating guitar gear listings from online marketplaces, tracking price drops, and managing personal collections. The project has evolved significantly beyond the "plan-only" status recorded in early Engram context — it now has a working frontend, a fully-implemented Rust backend with 9 SQL migrations, a Python scraper with one adapter (Reverb), CI/CD pipelines, and E2E test scaffolding.

**Data flow**: GitHub Actions runs the Python scraper every 6h → publishes catalog JSON to GitHub Pages → Tauri app fetches and syncs to local SQLite → all search/filter runs locally via FTS5.

#### Architecture Map

```
┌─────────────────────────────────────────────────────────────────┐
│                        FRONTEND (Svelte 5)                       │
│  src/routes/          SvelteKit routes (+page, /collection,      │
│                       /wishlist, /settings)                      │
│  src/lib/components/  7 Svelte components (ProductCard,          │
│                       FilterBar, PriceChart, DashboardCell, etc.)│
│  src/lib/stores/      5 writable stores (collection, filter,     │
│                       sync, wishlist, dashboard)                 │
│  src/lib/types/       4 type files mirroring Rust IPC contracts  │
│  src/lib/utils/       collectionValue.ts                         │
│  IPC: invoke() → Tauri commands                                  │
├─────────────────────────────────────────────────────────────────┤
│                     BACKEND (Rust / Tauri 2)                     │
│  commands/       9 Tauri command handlers (sync, search,         │
│                  collection, wishlist, dashboard, settings,      │
│                  price, image, export)                           │
│  services/       6 services (sync, search, image_cache,          │
│                  price_drop, alert_service, export_service)      │
│  repository/     7 repo traits + sqlite/ implementations         │
│  domain/         product.rs (RawProduct, SyncState, SearchFilters│
│                  SearchResult, CatalogFile, SortOrder)           │
│  migrations/     9 SQL migrations (001→009) with up+down files   │
│  lib.rs          AppState, AppError, initialize_database()       │
│  main.rs         Tauri builder, plugin registration              │
├─────────────────────────────────────────────────────────────────┤
│                     SCRAPER (Python 3.12)                        │
│  domain.py       CatalogProduct, CatalogFile (Pydantic)          │
│  ports.py        ScraperPort protocol, FetchError, ParseError    │
│  adapters/       ReverbAdapter (JSON API, pagination, retry)     │
│  cli.py          argparse CLI (--adapter, --output, --validate)  │
│  tests/          unit/ (test_reverb, test_domain), contract/     │
├─────────────────────────────────────────────────────────────────┤
│                     INFRASTRUCTURE                               │
│  .github/workflows/  ci.yml (PR), scrape.yml (cron 6h),         │
│                      release.yml, e2e.yml                        │
│  Makefile            test, lint, build, audit targets            │
│  tauri.conf.json     CSP, updater, bundle (deb, appimage)        │
│  openspec/           34 spec directories, 3 active changes       │
└─────────────────────────────────────────────────────────────────┘
```

### Affected Areas

| Area | Files | Status |
|------|-------|--------|
| Frontend routes | `src/routes/+page.svelte` (800 lines), `+layout.svelte`, `/collection`, `/wishlist`, `/settings` | Functional but monolithic |
| Frontend stores | `src/lib/stores/*.ts` (5 stores) | Working, Svelte 4 writable pattern (not runes) |
| Rust commands | `src-tauri/src/commands/*.rs` (9 files) | Complete, well-tested |
| Rust services | `src-tauri/src/services/*.rs` (6 files) | Complete, extensive tests |
| Rust repository | `src-tauri/src/repository/` (7 traits + sqlite impls) | Complete |
| Migrations | `src-tauri/src/repository/sqlite/migrations/` (9 up + 9 down) | Complete, well-tested |
| Scraper | `scraper/` (domain, ports, 1 adapter) | Functional, single source |
| CI/CD | `.github/workflows/` (4 workflows) | Active |
| E2E tests | `e2e-tests/specs/` (7 spec files) | Scaffolded, unverified |

### Approaches

#### 1. Architecture Mapping — Findings

**Strengths:**
- Clean Architecture in Rust backend: commands → services → repository traits ← sqlite implementations
- Ports & Adapters in scraper: domain → ports (Protocol) → adapters
- Type-safe IPC contracts mirrored in TypeScript (`src/lib/types/`)
- Custom migration runner with up/down support, gap detection, trigger-aware SQL splitting
- FTS5 with trigram tokenizer + sanitization against injection
- Image cache with request coalescing (DashMap + watch channels), LRU eviction, 50MB cap
- Price drop detection: pure function, materiality + cooldown layered
- Alert system: trait-based dispatchers (App, Ntfy, Webhook) with retry
- ETag-based conditional requests for catalog sync
- CSP configured, SSRF prevention in webhook URL validation

**Patterns in use:**
- Svelte 5 runes ($state, $derived, $props) in components
- Svelte 4 writable stores for state management (mixed approach)
- Tauri State injection for AppState
- async_trait for service/repo abstractions
- Pydantic v2 for scraper domain models

#### 2. Structural Irregularities

| # | Issue | Location | Severity |
|---|-------|----------|----------|
| 1 | **Version mismatch**: scraper is v0.2.0, app is v0.3.0 | `scraper/pyproject.toml:7` vs `package.json:4` / `Cargo.toml:3` | Medium |
| 2 | **HTTP User-Agent stale version**: "GuitarHub/0.2.0" but app is 0.3.0 | `src-tauri/src/lib.rs:67` | Low |
| 3 | **Double tracing init**: `tracing_subscriber` initialized in both `lib.rs:29-35` and `main.rs:7` | `src-tauri/src/lib.rs` + `main.rs` | Medium |
| 4 | **Massive code duplication** in sync_command.rs: alert dispatch logic copy-pasted between `sync_catalog` and `sync_local_catalog` (~80 lines duplicated) | `src-tauri/src/commands/sync_command.rs:22-82` vs `:97-157` | High |
| 5 | **Monolithic page component**: +page.svelte is 800 lines with inline styles, business logic, and UI | `src/routes/+page.svelte` | High |
| 6 | **Mixed state management**: Svelte 5 runes in components but Svelte 4 writable stores for shared state | `src/lib/stores/*.ts` vs `src/routes/*.svelte` | Medium |
| 7 | **Unused dependency**: `beautifulsoup4` in requirements.txt but never imported | `scraper/requirements.txt:3` | Low |
| 8 | **Python version discrepancy**: pyproject.toml says `>=3.12`, openspec config says `3.14`, CI uses `3.12` | `scraper/pyproject.toml:9` vs `openspec/config.yaml:13` vs `.github/workflows/ci.yml:16` | Low |
| 9 | **mypy overrides suppressing errors** in adapter and domain modules | `scraper/pyproject.toml:25-31` | Medium |
| 10 | **No i18n implementation** despite being listed as a Phase 1 convention | N/A | Medium |
| 11 | **No dark mode toggle** — only `prefers-color-scheme` media queries | `src/routes/+page.svelte:701-779` | Low |
| 12 | **Scraper version not synced** with app versioning scheme | `scraper/pyproject.toml:7` | Low |
| 13 | **Dependabot pip ecosystem commented out** with TODO | `.github/dependabot.yml:7-10` | Low |

#### 3. Technical Debt Catalog

| # | Item | Severity | Impact |
|---|------|----------|--------|
| 1 | **sync_command.rs duplication**: ~80 lines of alert dispatch logic duplicated between two commands. Extract to a shared function. | High | Maintenance burden, bug divergence risk |
| 2 | **+page.svelte monolith**: 800 lines mixing search logic, dashboard rendering, collection stats, settings, and 400+ lines of CSS. Should be decomposed into sub-components. | High | Unmaintainable, blocks parallel work |
| 3 | **No integration tests in Rust**: `src-tauri/tests/` directory is empty. All Rust tests are inline `#[cfg(test)]`. | Medium | No end-to-end backend verification |
| 4 | **E2E tests unverified**: 7 spec files exist but depend on `tauri-driver` which may not be installed. Never confirmed passing. | Medium | False confidence |
| 5 | **mypy strict overrides**: adapter and domain modules have disabled error codes, hiding potential type issues | Medium | Type safety erosion |
| 6 | **No i18n**: English-only UI despite i18n being a stated Phase 1 convention | Medium | Blocks Spanish-speaking users |
| 7 | **No accessibility audit**: aria-labels exist in some components but no systematic a11y testing | Medium | Excludes assistive tech users |
| 8 | **Store pattern inconsistency**: writable stores (Svelte 4) mixed with runes (Svelte 5). Should migrate to `$state`-based stores or runes-based state. | Medium | Confusing for contributors |
| 9 | **Double tracing init**: may cause duplicate log output or panics in certain configurations | Medium | Runtime instability |
| 10 | **No landing page**: Astro landing page mentioned in config but not present | Low | Missing marketing surface |
| 11 | **beautifulsoup4 unused**: dead dependency in scraper | Low | Confusing, minor bloat |
| 12 | **No batch inserts**: products upserted one-by-one in a loop in `sync.rs:132-216` | Medium | Slow sync for large catalogs |

#### 4. Bottlenecks

| # | Bottleneck | Location | Impact |
|---|-----------|----------|--------|
| 1 | **SQLite max_connections=1** | `lib.rs:37` | Single writer at a time; concurrent reads OK in WAL mode but writes serialize |
| 2 | **Sequential product upsert** | `services/sync.rs:132-216` | Each product is INSERT OR REPLACE'd individually in a loop. For 1000+ products, this is slow. Should use batch transactions. |
| 3 | **No FTS query caching** | `services/search.rs` | Every search hits SQLite directly. For repeated queries, a DashMap cache would help. |
| 4 | **Image cache: no concurrent download limit** | `services/image_cache.rs` | DashMap coalesces same-URL requests but no global semaphore limits parallel downloads |
| 5 | **Scraper: single-threaded pagination** | `scraper/adapters/reverb.py:96-132` | Pages fetched sequentially with `time.sleep()`. Could use async + concurrent pages. |
| 6 | **No rate limiting on scraper** | `scraper/adapters/reverb.py` | Only `time.sleep(delay)` between pages. No exponential backoff on 429s. |
| 7 | **Frontend: no virtual scrolling** | `src/routes/+page.svelte:186-189` | All search results rendered in DOM. For 100+ results, performance degrades. |

### Risks

1. **Data loss on migration failure**: The custom migration runner rolls back per-migration but has no backup mechanism before applying. A failed migration mid-chain could leave the DB in an inconsistent state.
2. **Scraper fragility**: Only one adapter (Reverb). If Reverb changes their API, the entire catalog pipeline breaks. No fallback sources.
3. **No offline fallback for sync**: If GitHub Pages is down, the app can't sync. Local sync exists but requires manual file selection.
4. **CSP may be too restrictive**: `connect-src: ipc: http://ipc.localhost` — if any external API call is needed from the frontend (not through Tauri), it will be blocked.
5. **Updater endpoint hardcoded**: `tauri.conf.json:39` points to `willsancazam.github.io` — needs to match the actual deployment target.

### Ready for Proposal

**Yes** — the codebase is substantial enough for a comprehensive MVP briefing. The orchestrator should:

1. Use the architecture map and irregularity table as the foundation for `MVP_BRIEFING.md`
2. Prioritize the **High** severity items (sync_command duplication, +page.svelte monolith) as immediate roadmap items
3. Include the batch insert optimization and store migration as performance/quality improvements
4. Note that the CI/CD pipeline is active and functional (ci.yml, scrape.yml, release.yml, e2e.yml)
5. Flag that E2E tests need verification before being considered part of the test suite
