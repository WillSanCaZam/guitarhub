# Tasks: Setup E2E Tests

## Review Workload Forecast

| Field | Value |
|-------|-------|
| Estimated changed lines | ~440 |
| 400-line budget risk | Low |
| Chained PRs recommended | Yes |
| Suggested split | PR 1 (Infrastructure) → PR 2 (Scenarios) |
| Delivery strategy | auto-chain |
| Chain strategy | stacked-to-main |

Decision needed before apply: No
Chained PRs recommended: Yes
Chain strategy: stacked-to-main
400-line budget risk: Low

### Suggested Work Units

| Unit | Goal | Likely PR | Notes |
|------|------|-----------|-------|
| 1 | E2E harness, CI, Makefile, smoke test | PR 1 | Base: main; includes wdio config, seed util, CI job |
| 2 | User-journey specs (search, sync, collection, settings) | PR 2 | Base: main; depends on PR 1 |

## Phase 1: Infrastructure (PR 1)

- [x] 1.1 Install E2E packages and configure scripts — create `e2e-tests/package.json` with @wdio/cli, @wdio/local-runner, @wdio/mocha-framework, @wdio/spec-reporter, webdriverio; add `test:e2e` to root `package.json`; add `test-e2e` target to `Makefile`. ~30 LoC. Deps: none.
  - Test: `npm run test:e2e` script exists and `make test-e2e` resolves.
- [x] 1.2 Create `e2e-tests/wdio.conf.ts` — WebdriverIO config with local runner, Mocha, tauri service, chrome binary `src-tauri/target/debug/guitarhub`, `beforeSession` seed hook. ~70 LoC. Deps: 1.1.
  - Test: `wdio` parses config without errors.
- [x] 1.3 Create DB seeding utility and fixture — create `e2e-tests/utils/seedDb.ts` to copy `fixtures/seed.db` to app data dir via `GUITARHUB_DB_PATH`; create `e2e-tests/fixtures/seed.db` with 10 products, 5 price-history rows, 1 collection item. ~40 LoC. Deps: 1.1.
  - Test: `seedDb()` runs and copies fixture to target path.
- [x] 1.4 Create smoke test spec — create `e2e-tests/specs/01-app-launch.spec.ts` verifying window title "GuitarHub" and dashboard grid visible (REQ-E2E-4 / S4). ~20 LoC. Deps: 1.2, 1.3.
  - Test: spec passes against debug binary.
- [x] 1.5 Update CI workflow — modify `.github/workflows/e2e.yml` to replace stub with real E2E job: install system deps, `cargo install tauri-driver`, build debug, `xvfb-run npm run test:e2e`. ~40 LoC. Deps: 1.1.
  - Test: workflow YAML is valid and `e2e` job runs on push to main.

## Phase 2: Scenarios (PR 2)

- [x] 2.1 Create shared selectors utility — create `e2e-tests/utils/selectors.ts` with centralized `data-testid` selectors for ProductCard, CollectionView, Settings. ~15 LoC. Deps: 1.2.
  - Test: imported by specs without errors.
- [x] 2.2 Create search scenario — create `e2e-tests/specs/02-search.spec.ts` typing "Fender" and asserting at least one `data-testid="product-card"` is visible (REQ-E2E-5 / S5). ~40 LoC. Deps: 1.4, 2.1.
  - Test: spec passes with seeded DB.
- [x] 2.3 Create sync scenario — create `e2e-tests/specs/03-sync.spec.ts` clicking Sync and asserting completion toast appears within 10s (REQ-E2E-6 / S6). ~50 LoC. Deps: 1.4, 2.1.
  - Test: spec passes with mock sync server.
- [x] 2.4 Create collection scenario — create `e2e-tests/specs/04-collection.spec.ts` adding product to collection, asserting stats update, and navigating to `/collection` (REQ-E2E-7 / S7). ~50 LoC. Deps: 1.4, 2.1.
  - Test: spec passes and DB state resets between sessions.
- [x] 2.5 Create settings scenario — create `e2e-tests/specs/05-settings.spec.ts` changing alert channel to "ntfy", saving, restarting app, and asserting persistence (REQ-E2E-8 / S8). ~60 LoC. Deps: 1.4, 2.1.
  - Test: spec passes and setting survives app restart.
- [x] 2.6 Create dashboard scenario — create `e2e-tests/specs/06-dashboard.spec.ts` asserting bento grid visibility and 9+ cells. ~20 LoC. Deps: 1.4, 2.1.
  - Test: spec passes.
