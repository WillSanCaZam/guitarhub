## Exploration: GuitarHub MVP State Assessment (June 2026)

### Current State

GuitarHub v0.3.0 is a **substantially complete MVP** with 4 tagged releases (v0.1.0 → v0.3.0), 34 spec directories, 4 CI/CD workflows, and a working data pipeline: Python scraper → GitHub Pages → Tauri app sync → SQLite FTS5 local search.

**Data flow**: GitHub Actions runs Reverb scraper every 6h → publishes catalog JSON to GitHub Pages → Tauri app fetches and syncs to local SQLite → all search/filter/collection runs locally offline-first.

#### Architecture Summary

```
┌──────────────────────────────────────────────────────────────┐
│                    FRONTEND (Svelte 5 + SvelteKit)            │
│  4 routes: / (dashboard), /collection, /wishlist, /settings  │
│  12 components: SearchPanel, ProductCard, FilterBar, etc.     │
│  5 stores: collection, dashboard, filter, sync, wishlist      │
│  Virtual scrolling via @tanstack/svelte-virtual               │
│  IPC: invoke() → Tauri commands                               │
├──────────────────────────────────────────────────────────────┤
│                    BACKEND (Rust / Tauri 2)                    │
│  9 commands: sync, search, collection, wishlist, dashboard,   │
│              settings, price, image, export                   │
│  7 services: sync, search, image_cache, price_drop,           │
│              alert_service, export_service                    │
│  7 repository traits + SQLite implementations                 │
│  9 SQL migrations with up+down files                          │
│  Batch upserts in single transactions                         │
│  ETag conditional requests for catalog sync                   │
├──────────────────────────────────────────────────────────────┤
│                    SCRAPER (Python 3.12)                       │
│  Ports & Adapters: domain → ports (Protocol) → adapters       │
│  1 adapter: ReverbAdapter (JSON API, pagination, retry)       │
│  CLI: argparse (--adapter, --output, --validate)              │
├──────────────────────────────────────────────────────────────┤
│                    TEST INFRASTRUCTURE                          │
│  Rust: unit tests inline + integration tests (src-tauri/tests)│
│  Python: unit + contract tests (pytest)                        │
│  Frontend: 10 component/store tests (vitest)                   │
│  E2E: 7 spec files (WebDriverIO, unverified)                  │
│  CI: 4 workflows (ci, scrape, release, e2e)                   │
└──────────────────────────────────────────────────────────────┘
```

### Features: Implemented & Working

| Feature | Status | Evidence |
|---------|--------|----------|
| **Full-text search** (FTS5 trigram) | ✅ Complete | `services/search.rs` — 961 lines, input sanitization, pagination, sorting |
| **Catalog sync** (remote + local) | ✅ Complete | `services/sync.rs` — ETag caching, state machine, batch upserts, 304 handling |
| **Price drop detection** | ✅ Complete | `services/price_drop.rs` — pure function, materiality + cooldown, multi-channel dispatch |
| **Alert system** (App/Ntfy/Webhook) | ✅ Complete | `services/alert_service.rs` — trait-based, SSRF prevention, retry |
| **Collection management** | ✅ Complete | `commands/collection_command.rs`, `stores/collection.ts`, UI with stats |
| **Wishlist** | ✅ Complete | `commands/wishlist_command.rs`, `stores/wishlist.ts` |
| **Dashboard bento grid** | ✅ Complete | 9-cell layout with stats, recent searches, featured deal |
| **Image cache** (LRU + coalescing) | ✅ Complete | `services/image_cache.rs` — DashMap, 50MB cap, 7-day TTL |
| **Export** (ZIP of JSON) | ✅ Complete | `commands/export_command.rs`, `services/export_service.rs` |
| **Price history** | ✅ Complete | `repository/price_history.rs`, recorded per sync |
| **Settings persistence** | ✅ Complete | Key-value store in SQLite |
| **Scraper** (Reverb adapter) | ✅ Complete | `scraper/adapters/reverb.py` — pagination, retry, category mapping |
| **CI/CD pipelines** | ✅ Complete | 4 workflows: ci.yml, scrape.yml, release.yml, e2e.yml |
| **Down migrations** | ✅ Complete | 9 down migration files for rollback |
| **In-app updater** | ✅ Complete | tauri-plugin-updater configured |
| **Virtual scrolling** | ✅ Complete | @tanstack/svelte-virtual in SearchPanel |
| **FilterBar** | ✅ Complete | Category, price, condition, currency, sort |

### Features: Partially Implemented or With Known Issues

| Feature | Issue | Severity |
|---------|-------|----------|
| **E2E tests** | 7 spec files scaffolded but never verified passing (requires tauri-driver + debug binary) | Medium |
| **Store pattern** | Mixed: Svelte 4 writable stores (`src/lib/stores/`) alongside Svelte 5 runes in components | Medium |
| **Dark mode** | Only `prefers-color-scheme` media queries, no manual toggle | Low |
| **i18n** | English-only UI despite being listed as a convention | Medium |
| **Accessibility** | Some aria-labels exist, no systematic a11y audit | Medium |
| **Scraper robustness** | Single adapter (Reverb), no rate limiting on 429s, sequential pagination | Medium |

### Architecture Quality Assessment

**Strengths (well-structured for MVP):**
- Clean Architecture in Rust: commands (IPC glue) → services (business logic) → repository traits ← SQLite impls
- Ports & Adapters in scraper: domain → ports (Protocol) → adapters
- Type-safe IPC contracts mirrored in TypeScript (`src/lib/types/`)
- Custom migration runner with up/down support, gap detection, trigger-aware SQL splitting
- FTS5 with trigram tokenizer + sanitization against injection
- Batch upserts in single transactions (atomic, performant)
- ETag conditional requests to skip unchanged catalogs
- Price drop detection as pure function with layered materiality + cooldown
- Trait-based alert dispatchers (App, Ntfy, Webhook) with SSRF prevention
- Image cache with request coalescing (DashMap + watch channels)

**Structural Issues Remaining:**

| # | Issue | Severity | Status |
|---|-------|----------|--------|
| 1 | sync_command.rs duplication | High | ✅ FIXED — `dispatch_price_drops` extracted |
| 2 | +page.svelte monolith | High | ⚠️ Reduced (323 lines) but still mixes concerns |
| 3 | No Rust integration tests | Medium | ✅ FIXED — added in Sprint 4 |
| 4 | E2E tests unverified | Medium | ⚠️ Still unverified |
| 5 | mypy strict overrides | Medium | ⚠️ Reduced but still present |
| 6 | No i18n | Medium | ❌ Not implemented |
| 7 | No accessibility audit | Medium | ❌ Not done |
| 8 | Store pattern inconsistency | Medium | ❌ Still mixed |
| 9 | Double tracing init | Medium | Need to verify |
| 10 | Version mismatches | Low | ⚠️ Partially fixed (user-agent updated) |

### Technical Debt Catalog

| # | Item | Severity | Impact |
|---|------|----------|--------|
| 1 | **Store migration**: writable stores (Svelte 4) → runes-based state | Medium | Confusing for contributors, inconsistent patterns |
| 2 | **+page.svelte decomposition**: extract SearchPanel, DashboardGrid, etc. into sub-components | Medium | 323 lines still mixes dashboard + search + settings |
| 3 | **E2E verification**: run specs against debug build to confirm they pass | Medium | False confidence in test coverage |
| 4 | **i18n foundation**: even basic English-only i18n setup for future localization | Medium | Blocks non-English users |
| 5 | **a11y audit**: WCAG 2.2 AA compliance check | Medium | Excludes assistive tech users |
| 6 | **beautifulsoup4**: check if still unused in requirements.txt | Low | Dead dependency |
| 7 | **Python version**: pyproject.toml says >=3.12, openspec says 3.14, CI uses 3.12 | Low | Confusion |

### Test Coverage Summary

| Layer | Tests | Status |
|-------|-------|--------|
| Rust unit tests | ~341 inline `#[cfg(test)]` | ✅ Passing |
| Rust integration tests | 4 test functions in `src-tauri/tests/` | ✅ Added Sprint 4 |
| Python unit tests | `test_reverb.py`, `test_domain.py` | ✅ Passing |
| Python contract tests | `test_protocol.py` | ✅ Passing |
| Frontend component tests | 7 test files (DashboardCell, FilterBar, PriceBadge, PriceChart, ProductCard, Settings, CollectionView) | ✅ Passing |
| Frontend store tests | 2 test files (collection, filter) | ✅ Passing |
| Frontend page tests | 1 test file (+page) | ✅ Passing |
| E2E tests | 7 spec files (app-launch, search, sync, collection, settings, dashboard, filters) | ⚠️ Unverified |

### Risks

1. **Single scraper source**: Only Reverb adapter. If Reverb changes API, entire catalog pipeline breaks. No fallback.
2. **E2E test gap**: 7 spec files exist but have never been confirmed passing. Could hide regressions.
3. **No offline catalog import UX**: `sync_local_catalog` exists in backend but no UI to trigger it — users must use the URL sync.
4. **GitHub Pages dependency**: Catalog hosted on GitHub Pages. If Pages is down, sync fails. No offline fallback.
5. **Updater endpoint**: Points to `willsancazam.github.io` — needs verification this matches deployment.

### Ready for Proposal

**Yes** — the MVP is substantially complete and functional. The codebase has:
- Working sync pipeline (scraper → GitHub Pages → app)
- Full-text search with FTS5
- Price drop detection and alerts
- Collection and wishlist management
- Dashboard with bento grid layout
- CI/CD with 4 workflows
- 4 tagged releases

**Recommended next steps for the orchestrator:**
1. **Verify E2E tests pass** — run the 7 specs against a debug build
2. **Run full test suite** — `make test` to confirm everything passes
3. **Address remaining Medium debt** — store migration, +page decomposition, i18n foundation
4. **Consider second scraper adapter** — reduce single-source risk
5. **Ship v0.4.0** with verified E2E tests and remaining debt fixes
