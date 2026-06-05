# Proposal: MVP Completion

## Intent

Ship the GuitarHub MVP. Backend sync/search are stubs, frontend is disconnected from IPC, no scraper exists, and the repo isn't presentable. Five work units bridge the gap from scaffolding to a functional demo.

## Scope

### In Scope
1. **SyncService** — Real sync state machine: fetch catalog JSON from GitHub Pages → parse → upsert SQLite → update `sync_state`. Full Tauri command.
2. **SearchService** — FTS5 search with sanitization, category/price/source filters, pagination, sorting. Tauri command.
3. **Frontend scaffold** — Wire `+page.svelte`/`+layout.svelte` to existing components + new Tauri commands via `@tauri-apps/api/core`.
4. **Scraper MVP** — `scraper/` directory with Reverb.com adapter (Ports & Adapters), output JSON compatible with SyncService schema. GitHub Action `scrape.yml` runs every 6h.
5. **Repo presentable** — README.md (description, build instructions, screenshot placeholders), LICENSE (GPL-3.0), CHANGELOG.md initial.

### Out of Scope
- Multiple scraper adapters beyond Reverb
- Multi-source aggregated search or complex filter UI (faceted, range sliders)
- E2E or `tauri-driver` tests
- Mobile/desktop packaging beyond what exists
- Landing page (Astro) changes

## Capabilities

### New Capabilities
- `search-service`: FTS5 search, sanitization, filter/pagination/sort via IPC
- `scraper`: Python scraper with Reverb adapter, GitHub Actions cron schedule
- `repo-presentable`: README, LICENSE (GPL-3.0), CHANGELOG

### Modified Capabilities
- `sync-service`: Upgrade from JSON file stub to remote catalog download + upsert state machine
- `frontend-scaffolding`: Upgrade from empty route shells to live IPC-connected search UI

## Approach

Five stacked PRs (auto-chain delivery). Each PR is independently reviewable at ~200–400 lines:

| PR | Focus | Est. LOC |
|----|-------|----------|
| 1 | SyncService real impl + Tauri command | ~300 |
| 2 | SearchService + Tauri command | ~200 |
| 3 | Frontend IPC wiring + search view | ~150 |
| 4 | Scraper + Reverb adapter + GHA cron | ~400 |
| 5 | README, LICENSE, CHANGELOG | ~150 |

Backend PRs (1–2) are TDD-locked: write failing test, implement, pass. Scraper tests run with `pytest`. All PRs gated by `make test` (Rust + Python).

## Affected Areas

| Area | Impact | Description |
|------|--------|-------------|
| `src-tauri/src/services/sync.rs` | Modified | Replace stub with real sync |
| `src-tauri/src/services/search.rs` | New | FTS5 search service |
| `src-tauri/src/commands/sync_command.rs` | Modified | Wire real sync |
| `src-tauri/src/commands/search_command.rs` | New | Search IPC |
| `src-tauri/src/lib.rs` | Modified | Register new commands |
| `src/routes/+page.svelte` | Modified | Search + result list |
| `src/routes/+layout.svelte` | Modified | Nav/wrapper |
| `scraper/` | New | Python scraper package |
| `.github/workflows/scrape.yml` | New | Cron schedule |
| `README.md` | New | Project docs |
| `LICENSE` | New | GPL-3.0 |
| `CHANGELOG.md` | New | Initial entry |

## Risks

| Risk | Likelihood | Mitigation |
|------|------------|------------|
| FTS5 injection via search input | Med | Strict input sanitization before MATCH |
| Reverb site HTML/CSS changes break scraper | Med | Adapter pattern — swap parser, test contract |
| Large catalog JSON strains GitHub Pages | Low | Compress with `--gzip`, keep under 10 MB |
| Tauri complex type serialization | Low | Use `#[derive(Serialize)]` on all response types |

## Rollback Plan

Per stacked PR: `git revert <merge-commit>` on the feature tracker branch. If PR #1 is already merged to `main`, reset chain: revert PR #1 on `main`, abandon feature branch. Individual services (sync/search) are independent — rollback one without the others.

## Dependencies

- GitHub Pages publishing must be enabled in repo settings (manual one-time setup)
- Reverb.com must remain accessible from GHA runners
- `rust-src` component for TDD `cargo test` annotations

## Success Criteria

- [ ] `cargo test` and `pytest scraper/tests/ -v` pass
- [ ] `cargo build` produces working Tauri binary
- [ ] `sync_catalog` downloads real catalog, upserts into SQLite, updates `sync_state`
- [ ] `search_products` returns paginated FTS5 results with filters
- [ ] Frontend shows search bar, executes search, renders ProductCard list
- [ ] Scraper produces valid JSON matching SyncService `CatalogFile` schema
- [ ] `scrape.yml` runs `pip-audit` before scraper and `--validate-input` before publish
- [ ] README builds from scratch work on a clean clone
