# Tasks: MVP Completion

## Review Workload Forecast

| Field | Value |
|-------|-------|
| Estimated changed lines | ~1200 |
| 800-line budget risk | High |
| Chained PRs recommended | Yes |
| Delivery strategy | force-chained |
| Chain strategy | stacked-to-main |

Decision needed before apply: No
Chained PRs recommended: Yes
Chain strategy: stacked-to-main
400-line budget risk: High

### Suggested Work Units

| Unit | Goal | PR | Notes |
|------|------|----|-------|
| 1 | SyncService + domain types | PR 1 | ~300 lines, TDD RED→GREEN, base: main |
| 2 | SearchService FTS5 | PR 2 | ~200 lines, TDD, depends on PR 1 domain types |
| 3 | Frontend IPC wiring | PR 3 | ~150 lines, depends on PR 2 commands |
| 4 | Scraper + Reverb + GHA cron | PR 4 | ~400 lines, independent, pytest |
| 5 | README + LICENSE + CHANGELOG | PR 5 | ~150 lines, independent |

## Phase 1: SyncService (PR 1)

- [x] 1.1 Add `SearchFilters`, `SortOrder`, `SearchResult`, `SyncState` types to `src-tauri/src/domain/product.rs`
- [x] 1.2 RED: Write failing tests for `CatalogSyncService` — state transitions, upsert counting, HTTP error handling (httpmock/in-memory SQLite)
- [x] 1.3 Implement `CatalogSyncService` in `src/services/sync.rs` — HTTP fetch, FSM (`idle→downloading→validating→sanitizing→inserting→done|failed`), SQLite upsert
- [x] 1.4 Update `sync_command.rs` — accept `url: String`, wire `CatalogSyncService`
- [x] 1.5 GREEN: `cargo test` passes (state machine lifecycle, concurrency rejection, error cases)

## Phase 2: SearchService (PR 2)

- [ ] 2.1 RED: Write failing tests for `FtsSearchService` — sanitize (strip+quote), filter (category/price/source), paginate (limit/offset), all SortOrder variants
- [ ] 2.2 Implement `FtsSearchService` in `src/services/search.rs` — FTS5 MATCH sanitization, WHERE/ORDER BY/LIMIT/OFFSET generation
- [ ] 2.3 Create `search_command.rs` `#[tauri::command] search_products`, add `pub mod search_command;` and register in `lib.rs`, add `pub mod search;` to `services/mod.rs`
- [ ] 2.4 GREEN: `cargo test` passes (sanitize, all sort orders, filter combos, no-results, pagination boundaries)

## Phase 3: Frontend (PR 3)

- [ ] 3.1 Wire `+page.svelte` — search bar invokes `search_products`, ProductCard grid, loading spinner/empty/error states
- [ ] 3.2 Wire `+layout.svelte` — nav with sync button invoking `sync_catalog`, progress indicator during sync
- [ ] 3.3 Add "Load more" pagination — limit/offset params, button disabled when all results shown

## Phase 4: Scraper (PR 4)

- [ ] 4.1 Create `scraper/` package — `pyproject.toml`, `requirements.txt`, `ports.py` with `ScraperPort` Protocol
- [ ] 4.2 Implement `ReverbAdapter(ScraperPort)` — HTTP with retry+backoff, HTML parse → CatalogProduct field mapping
- [ ] 4.3 Create `cli.py` — `scraper --adapter reverb --output catalog.json --validate` entry point
- [ ] 4.4 Create `scraper/tests/unit/` (HTML fixture extraction) + `tests/contract/` (Protocol conformance via mypy/pytest)
- [ ] 4.5 Create `.github/workflows/scrape.yml` — cron `0 */6 * * *`, pip-audit gate before scrape, schema validation after
- [ ] 4.6 `pytest scraper/tests/unit scraper/tests/contract -v` passes

## Phase 5: Repo Docs (PR 5)

- [ ] 5.1 Create `README.md` — project desc, prerequisites (Rust/Node/Python), `make build`/`make dev`/`make test`, screenshot placeholder, tech stack summary
- [ ] 5.2 Create `LICENSE` — GPL-3.0 full text as published by FSF
- [ ] 5.3 Create `CHANGELOG.md` — Keep a Changelog format, `[Unreleased]` with Added/Changed/Fixed sections
- [ ] 5.4 Add `SPDX-License-Identifier: GPL-3.0-or-later` headers to all new Rust and Python source files
