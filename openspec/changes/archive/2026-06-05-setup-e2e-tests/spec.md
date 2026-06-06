# Combined Spec + Design: setup-e2e-tests

> **Status**: Draft
> **Change**: setup-e2e-tests
> **Artifact Type**: spec+design (infrastructure/config — combined)

---

## Specs

### Purpose

Introduce end-to-end test infrastructure for the GuitarHub Tauri 2 + SvelteKit 5 desktop app, using the officially supported `tauri-driver` + WebdriverIO stack. This spec covers requirements for the test harness, CI integration, and the initial set of user-journey scenarios.

---

### Requirements

#### REQ-E2E-1: `npm run test:e2e` launches the full E2E pipeline

The system MUST provide an `npm run test:e2e` script that compiles the debug app binary, starts `tauri-driver`, and executes WebdriverIO specs.

#### Scenario: S1 — Smoke test pipeline

- GIVEN a clean working tree with Node dependencies installed
- WHEN `npm run test:e2e` is executed
- THEN the debug binary is built (`cargo tauri build --debug --no-bundle`)
- AND `tauri-driver` starts on port 4444
- AND WebdriverIO discovers and runs all `e2e-tests/specs/**/*.spec.ts`
- AND the process exits with code 0 on success

---

#### REQ-E2E-2: Each E2E test session starts with a fresh seeded SQLite database

The system MUST reset the application database to a known seed state before every test session to prevent cross-test leakage.

#### Scenario: S2 — Database isolation

- GIVEN two consecutive E2E test sessions
- WHEN the first session adds a collection item
- AND the second session reads the collection count
- THEN the second session sees the seeded count, not the first session's mutation

---

#### REQ-E2E-3: CI workflow runs E2E tests on Ubuntu headless

The `.github/workflows/e2e.yml` MUST run E2E tests on `ubuntu-latest` inside `xvfb-run`, triggered on push to `main` and on pull requests.

#### Scenario: S3 — CI gate passes

- GIVEN a pull request is opened
- WHEN the `e2e` job runs in GitHub Actions
- THEN the job installs system deps (webkit2gtk-driver, xvfb)
- AND installs `tauri-driver`
- AND runs `xvfb-run --auto-servernum npm run test:e2e`
- AND the job completes within 15 minutes

---

#### REQ-E2E-4: App launch scenario verifies window and title

The system MUST verify that launching the app produces a visible window with the title "GuitarHub".

#### Scenario: S4 — Window title assertion

- GIVEN the E2E harness is ready
- WHEN the app binary launches via `tauri-driver`
- THEN a window is visible
- AND the window title equals "GuitarHub"

---

#### REQ-E2E-5: Search scenario returns product results

The system MUST verify that typing "Fender" into the search input renders at least one `ProductCard`.

#### Scenario: S5 — Search happy path

- GIVEN the seeded DB contains products with "Fender" in the name
- WHEN the user types "Fender" and submits the search
- THEN at least one element with `data-testid="product-card"` is visible
- AND the results meta shows `> 0` items

---

#### REQ-E2E-6: Sync scenario triggers and completes

The system MUST verify that clicking the Sync button results in a completion toast.

#### Scenario: S6 — Sync completion

- GIVEN a local mock HTTP server is serving a valid catalog JSON
- WHEN the user clicks the Sync button
- THEN a toast/notification with text "sync completed" appears within 10 seconds

---

#### REQ-E2E-7: Collection scenario adds and persists an item

The system MUST verify that adding a product to the collection updates the Collection cell on the dashboard.

#### Scenario: S7 — Add to collection

- GIVEN a search returns at least one product
- WHEN the user clicks "Add to collection" on the first result
- THEN the Collection stats cell shows `1 items`
- AND navigating to `/collection` renders one `.collection-card`

---

#### REQ-E2E-8: Settings scenario changes and persists an alert channel

The system MUST verify that changing the alert channel in Settings and saving persists across app restarts.

#### Scenario: S8 — Settings persistence

- GIVEN the app is on the dashboard with Settings visible
- WHEN the user selects "ntfy" as the alert channel
- AND clicks Save
- THEN a success confirmation is visible
- WHEN the app is restarted
- THEN the alert channel still shows "ntfy"

---

## Design

### Technical Approach

Use `tauri-driver` (official Tauri 2 WebDriver intermediary) with WebdriverIO as the test runner. Tests drive the real compiled debug binary via WebDriver, interacting with the DOM like a standard web app. No app code changes are required.

---

### Architecture Decisions

| Decision | Choice | Alternatives | Rationale |
|----------|--------|--------------|-----------|
| Runner | WebdriverIO + Mocha | Selenium, Playwright | Official Tauri docs recommend WebdriverIO; mature selectors and retry logic |
| Browser | Chrome via `tauri:options` | Safari, Edge | Tauri 2 on Linux uses WebKit; `tauri-driver` abstracts the binary path |
| DB Seed | Pre-built SQLite fixture copied per session | SQL script generation, in-memory | Fastest reset; no schema drift risk |
| CI OS | Ubuntu only | macOS, Windows | `tauri-driver` does not support macOS; Linux is cheapest headless CI target |
| Build | Debug binary (`--no-bundle`) | Release binary | Debug builds are faster; bundle step is unnecessary for E2E |

---

### Data Flow

```
+------------+     +----------------+     +------------------+
|  wdio.conf |---->| tauri-driver   |---->|  GuitarHub debug |
|  (Node)    |     | (WebDriver)    |     |  binary (Rust)   |
+------------+     +----------------+     +------------------+
       |                                          |
       |                                          v
       |                                   +-------------+
       |                                   |  SQLite DB  |
       |                                   |  (seeded)   |
       |                                   +-------------+
       v
+------------+
|  seedDb.ts |
|  (copies   |
|  fixture)  |
+------------+
```

---

### File Changes

| File | Action | Description |
|------|--------|-------------|
| `e2e-tests/wdio.conf.ts` | Create | WebdriverIO config: local runner, Mocha, tauri service, chrome binary pointing to debug build, `beforeSession` seed hook |
| `e2e-tests/utils/seedDb.ts` | Create | Copies `fixtures/seed.db` to app data dir before each session; reads `GUITARHUB_DB_PATH` env var |
| `e2e-tests/utils/selectors.ts` | Create | Centralized `data-testid` selectors for ProductCard, CollectionView, Settings, etc. |
| `e2e-tests/fixtures/seed.db` | Create | Pre-built SQLite DB with 10 products, 5 price-history rows, 1 collection item |
| `e2e-tests/specs/01-app-launch.spec.ts` | Create | REQ-E2E-4 / S4 |
| `e2e-tests/specs/02-search.spec.ts` | Create | REQ-E2E-5 / S5 |
| `e2e-tests/specs/03-sync.spec.ts` | Create | REQ-E2E-6 / S6 |
| `e2e-tests/specs/04-collection.spec.ts` | Create | REQ-E2E-7 / S7 |
| `e2e-tests/specs/05-settings.spec.ts` | Create | REQ-E2E-8 / S8 |
| `package.json` | Modify | Add `"test:e2e": "wdio run e2e-tests/wdio.conf.ts"` to scripts |
| `Makefile` | Modify | Add `test-e2e` target; append it to `test` dependency list |
| `.github/workflows/e2e.yml` | Modify | Replace stub with real E2E job: install deps, `cargo install tauri-driver`, build debug, `xvfb-run npm run test:e2e` |

---

### Interfaces / Contracts

#### seedDb.ts

```typescript
export async function seedDb(): Promise<void>;
// Copies e2e-tests/fixtures/seed.db to the path defined by
// process.env.GUITARHUB_DB_PATH (or default app data dir).
// Must be called in wdio.conf.ts beforeSession.
```

#### wdio.conf.ts — capabilities snippet

```typescript
capabilities: [{
  browserName: 'chrome',
  'tauri:options': {
    application: './src-tauri/target/debug/guitarhub'
  }
}]
```

---

### Testing Strategy

| Layer | What to Test | Approach |
|-------|-------------|----------|
| Unit | Rust commands, Svelte components | Existing `cargo test`, `vitest` |
| Integration | Tauri command mocks, scraper contracts | Existing `cargo test --ignored`, `pytest` |
| E2E | Full user journeys (launch → search → sync → collect → settings) | `tauri-driver` + WebdriverIO driving real binary |

---

### Migration / Rollout

No migration required. This is a net-new test harness. Existing unit and integration tests remain unchanged. The `make test` target will now include E2E; developers who do not have `tauri-driver` installed will see the E2E suite fail locally until they run `cargo install tauri-driver`.

---

### Out of Scope

- macOS E2E (`tauri-driver` does not support WKWebView WebDriver)
- Export ZIP E2E (native dialog automation gap; verified at integration level)
- Windows E2E in CI (Linux only for now)

---

### Open Questions

- [ ] Should `seed.db` be committed as a binary blob, or generated from `sample_catalog.json` via a build script?
- [ ] Do we gate `make test-e2e` behind an env var (e.g., `CI=true`) so local `make test` skips it by default?
