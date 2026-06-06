# Exploration: setup-e2e-tests

## Current State

The GuitarHub project currently has **zero end-to-end test infrastructure**:

- `.github/workflows/e2e.yml` exists but is a stub — it only checks if `scraper/` is present and exits. No actual E2E job runs.
- `src-tauri/Cargo.toml` has no E2E dependencies. `dev-dependencies` only include `httpmock` and `tempfile` for unit/integration tests.
- `package.json` has no E2E scripts — only Vitest for frontend unit tests (`npm run test`).
- `Makefile` has `test`, `test-app`, `test-frontend`, `test-scraper` — no `test-e2e` target.
- `src-tauri/tests/fixtures/sample_catalog.json` exists, suggesting some preparation for integration tests, but no E2E harness consumes it.
- The frontend is a single-page SvelteKit 5 app with a bento-grid dashboard (`/`) and a `/collection` route. Settings are inline on the main page.
- The app uses Tauri IPC extensively: `search_products`, `sync_catalog`, `export_data`, `add_to_collection`, `get_collection`, `save_setting`, `get_setting`, etc.

## Affected Areas

- `src-tauri/Cargo.toml` — add `tauri-driver` binary dependency (or document installation)
- `package.json` — add `test:e2e` script
- `Makefile` — add `test-e2e` target
- `.github/workflows/e2e.yml` — replace stub with real WebDriver E2E job
- `e2e-tests/` — new directory for WebdriverIO config, specs, and test utilities
- `src-tauri/tests/fixtures/` — reuse `sample_catalog.json` as sync payload; may need a seeded SQLite DB fixture

## Approaches

### 1. tauri-driver + WebdriverIO (Recommended)
Use the official Tauri WebDriver intermediary (`cargo install tauri-driver`) with WebdriverIO as the test runner. This is the approach documented and maintained by the Tauri team for Tauri 2.x.

- **Pros:**
  - Officially supported by Tauri; docs are current (last updated Oct 2025).
  - No app code changes required — tests drive the real compiled binary.
  - WebdriverIO has mature selectors, waits, and assertions.
  - Works on Linux (WebKitWebDriver) and Windows (Edge Driver); CI examples provided.
  - `tauri-driver` handles native WebDriver server lifecycle automatically.
- **Cons:**
  - macOS desktop is **not supported** (no WKWebView WebDriver tool available).
  - Requires compiling the app in debug mode before each test run (`cargo tauri build --debug --no-bundle`).
  - Native dialogs (file save picker for export) cannot be automated via WebDriver.
  - Database state must be managed externally (seed DB, env var `GUITARHUB_DB_PATH`).
- **Effort:** Medium

### 2. tauri-driver + Selenium
Same underlying `tauri-driver`, but use Selenium WebDriver client (e.g., Rust `fantoccini` or Python `selenium`) instead of WebdriverIO.

- **Pros:**
  - Same official `tauri-driver` support.
  - Could write tests in Rust or Python, matching backend/scraper stack.
- **Cons:**
  - Selenium bindings for Tauri are less documented than WebdriverIO.
  - WebdriverIO has better async/await ergonomics and retry logic for flaky webviews.
  - Rust E2E test suites (fantoccini) add compile-time overhead.
- **Effort:** Medium-High

### 3. Mock Runtime Integration Tests (Not E2E)
Use Tauri's mock runtime (`tauri::test::mock_builder`) to test commands without a real webview.

- **Pros:**
  - Fast, no GUI needed, runs in `cargo test`.
  - Good for command-level coverage.
- **Cons:**
  - Does **not** test the frontend Svelte code or user journeys.
  - Not E2E — this is integration testing, which the project already does.
- **Effort:** Low (already partially done)

### 4. Playwright (Not Recommended)
There is no official `playwright-tauri` integration. Playwright is designed for browsers, not Tauri webviews. Community attempts exist but are unmaintained and brittle.

- **Pros:**
  - Familiar API for frontend developers.
- **Cons:**
  - Not supported by Tauri; IPC and native APIs won't work correctly.
  - Would require running the frontend in a browser context, losing all Tauri-specific behavior.
- **Effort:** High (and likely to fail)

## Recommendation

**Use Approach 1: `tauri-driver` + WebdriverIO.**

This is the only officially supported path for Tauri 2 E2E in 2026. The Tauri docs explicitly recommend it, provide working examples, and maintain CI templates. GuitarHub's `openspec/config.yaml` already lists "tauri-driver + WebDriver (E2E, weekly)" as the intended approach.

### How to Start the App for E2E Tests

1. Build the debug binary: `cargo tauri build --debug --no-bundle` (produces `src-tauri/target/debug/guitarhub`).
2. In `wdio.conf.js`, point `tauri:options.application` to that binary.
3. `tauri-driver` spawns the app and proxies WebDriver requests.

### How to Mock Backend Data

- **SQLite Database:** Create a pre-seeded test database (e.g., `e2e-tests/fixtures/test.db`) with products, settings, and collection items. Set `GUITARHUB_DB_PATH` before launching the app in `wdio.conf.js` `beforeSession`.
- **HTTP Sync Mock:** For `sync_catalog`, start a lightweight Node.js HTTP server in `onPrepare` that serves `sample_catalog.json` on `localhost:9999`. The E2E test triggers sync with that URL.
- **Export Dialog:** The export feature uses `tauri-plugin-dialog` (native file picker). WebDriver cannot interact with native dialogs. **Mitigation:** Skip export in pure E2E; verify it in integration tests. Alternatively, add a hidden debug command `export_data_to_path` gated by a compile-time feature flag, but this violates "test the real app."

### Handling Tauri IPC in E2E Context

WebdriverIO interacts with the DOM only. IPC calls (`invoke()`) are triggered by frontend interactions (clicks, form submissions). No special handling needed — the tests click buttons and assert on DOM changes, just like any web app. The difference is that the backend is real Rust code, not a mock server.

### Test Scenarios (7 User Journeys)

1. **App launches successfully**
   - Launch binary via `tauri-driver`.
   - Assert window title contains "GuitarHub".
   - Assert dashboard grid is visible.

2. **Search for "Fender" shows results**
   - Seed DB with products (including Fender SKUs).
   - Type "Fen" into search input, click Search.
   - Assert product cards appear with "Fender" text.
   - Assert results meta shows `> 0` results.

3. **Sync catalog completes**
   - Start local HTTP server serving `sample_catalog.json`.
   - Trigger sync with localhost URL.
   - Assert Sync Status cell shows products updated / price drops detected.

4. **Add item to collection**
   - Search for a product.
   - Click "Add to collection" button on a ProductCard.
   - Assert Collection cell stats update (total_items increases).

5. **View collection page**
   - Click Collection cell link (navigates to `/collection`).
   - Assert collection items are rendered.
   - Assert total value stat is visible.

6. **Settings page loads and saves**
   - Scroll to Settings section.
   - Select "Webhook POST" radio button.
   - Enter webhook URL.
   - Click "Test Notification".
   - Assert test result shows success (or appropriate error for invalid URL).
   - Reload app; assert setting persists.

7. **Export data creates ZIP**
   - **Blocked by native dialog.** Propose: verify export integration in Rust tests (`export_data_cmd` already tested). In E2E, assert the Export button is present and clickable; skip file-system verification unless a test-mode dialog bypass is added.

### CI Integration (GitHub Actions)

Replace `.github/workflows/e2e.yml` with a real job:

```yaml
name: E2E
on:
  workflow_dispatch:
  schedule:
    - cron: '0 0 * * 0'  # weekly
  push:
    branches: [main]

jobs:
  e2e:
    runs-on: ubuntu-latest
    timeout-minutes: 15
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
        with:
          workspaces: src-tauri
      - uses: actions/setup-node@v4
        with:
          node-version: '22'
          cache: 'npm'
      - name: Install Tauri system deps
        run: |
          sudo apt-get update
          sudo apt-get install -y libwebkit2gtk-4.1-dev libayatana-appindicator3-dev webkit2gtk-driver xvfb
      - run: npm ci
      - run: cargo install tauri-driver --locked
      - run: cargo tauri build --debug --no-bundle
        working-directory: src-tauri
      - run: npm ci
        working-directory: e2e-tests
      - name: Run E2E tests
        run: xvfb-run npm test
        working-directory: e2e-tests
```

### Scope Estimate

Rough line count for the full setup:
- `e2e-tests/package.json` — 15 lines
- `e2e-tests/wdio.conf.js` — 80 lines
- `e2e-tests/test/specs/*.e2e.js` (7 scenarios) — ~250 lines
- `e2e-tests/test/utils/db-seed.js` — 40 lines
- `e2e-tests/test/utils/mock-server.js` — 30 lines
- `.github/workflows/e2e.yml` — 40 lines
- `Makefile` additions — 5 lines
- `package.json` script addition — 1 line

**Total: ~460 lines** — this exceeds the 400-line review budget. Recommend splitting into two chained work units:
1. **WU1:** Infrastructure (wdio config, CI, Makefile, fixtures, 1 smoke test).
2. **WU2:** Test scenarios (remaining 6 journeys).

## Risks

1. **Native dialog automation gap** — Export and any future file-picker flows cannot be fully E2E-tested without app modifications. Risk: medium.
2. **Flaky WebDriver on CI** — WebKitWebDriver + xvfb can be flaky in headless environments. Risk: medium. Mitigation: generous timeouts (60s), retry logic in wdio config.
3. **macOS exclusion** — E2E tests will not run on macOS desktop. macOS developers cannot run E2E locally. Risk: low (CI runs on Linux; macOS mobile E2E is a separate concern).
4. **Database state leakage** — Tests share the same app process. If a test crashes mid-run, the DB may be dirty for subsequent tests. Mitigation: use a fresh seeded DB per test session or per spec.
5. **Build time** — `cargo tauri build --debug --no-bundle` adds ~2-4 minutes to every E2E run. Risk: low.
6. **Scraper dependency** — The existing e2e.yml stub is gated on `scraper/` existence. The real E2E workflow does NOT depend on the scraper, so this gating should be removed.

## Ready for Proposal

**Yes.** The exploration is complete. The orchestrator should tell the user:

- The recommended tool stack is `tauri-driver` + WebdriverIO (official Tauri 2 support).
- The work should be split into two chained PRs because the total scope (~460 lines) exceeds the 400-line review budget.
- The export-ZIP scenario is partially blocked by native dialogs; the proposal should note that export will be verified at the integration level, with E2E covering UI presence only.
