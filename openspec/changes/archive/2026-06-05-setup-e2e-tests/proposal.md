# Proposal: setup-e2e-tests

## Intent

Unit and integration tests cannot catch frontend-backend integration failures, IPC contract drift, or Tauri webview regressions. GuitarHub currently has zero E2E coverage; this change introduces end-to-end validation through the real compiled binary to protect the critical user journeys (search, sync, collection, settings) from shipping broken.

## Scope

### In Scope
- PR 1 (Infrastructure): tauri-driver + WebdriverIO harness, seeded DB fixture, mock HTTP server, CI workflow, `Makefile` target, one smoke test.
- PR 2 (Scenarios): 6 user-journey E2E specs covering search, sync, collection add/view, settings, and export UI presence.
- Replace `.github/workflows/e2e.yml` stub with a real E2E job running on `ubuntu-latest` via `xvfb`.

### Out of Scope
- macOS E2E (tauri-driver does not support WKWebView WebDriver).
- Full native-dialog automation for file export (verified at integration-test level instead).
- Mobile E2E (separate concern).

## Capabilities

### New Capabilities
- `e2e-testing-infrastructure`: WebdriverIO config, tauri-driver wiring, seeded SQLite fixtures, mock sync server, CI job, and smoke test.
- `e2e-user-journeys`: DOM-driven E2E specs for app launch, search, sync, collection add/view, settings persistence, and export button presence.

### Modified Capabilities
- `frontend-testing`: Add `test-e2e` to `Makefile` and `test:e2e` to `package.json` so `make test` can optionally gate E2E. Existing Vitest behavior unchanged.

## Approach

1. **Stack**: `tauri-driver` (official Tauri 2 WebDriver intermediary) + WebdriverIO. Tests drive the real debug binary; no app code changes required.
2. **Seeding**: Each session starts from a fresh pre-seeded SQLite DB (`e2e-tests/fixtures/test.db`) via `GUITARHUB_DB_PATH`. Per-spec cleanup via `beforeSession`.
3. **Sync mock**: A lightweight Node.js HTTP server serves `sample_catalog.json` on `localhost:9999` during `onPrepare`; tests trigger sync against it.
4. **Export dialog**: WebDriver cannot automate native file pickers. E2E asserts the Export button is present and clickable; ZIP content is verified in Rust integration tests.

## Affected Areas

| Area | Impact | Description |
|------|--------|-------------|
| `e2e-tests/` | New | WebdriverIO config, specs, utilities, fixtures |
| `.github/workflows/e2e.yml` | Modified | Replace stub with real xvfb + tauri-driver job |
| `Makefile` | Modified | Add `test-e2e` target |
| `package.json` | Modified | Add `test:e2e` script |
| `src-tauri/tests/fixtures/` | Reuse | `sample_catalog.json` becomes sync payload |

## Risks

| Risk | Likelihood | Mitigation |
|------|------------|------------|
| Native dialog automation gap | Med | Export verified in integration tests; E2E checks UI presence only |
| Flaky WebDriver on CI | Med | 60s timeouts, wdio retry logic, weekly cadence (not every push) |
| DB state leakage between specs | Med | Fresh seeded DB per session via env var |
| macOS developers cannot run E2E locally | Low | Document Linux CI as canonical; macOS mobile E2E is separate |
| Build time (+2–4 min per run) | Low | Debug build cached by `rust-cache`; run weekly not per-PR |

## Rollback Plan

1. Revert the PR(s) — all changes are additive (`e2e-tests/` directory, workflow file, Makefile lines, package.json script).
2. If CI becomes flaky, disable the E2E job by commenting out the `e2e` job block in `.github/workflows/e2e.yml` while keeping the file and artifacts in repo.
3. Zero cost: no runtime dependencies are added to the shipped app; tauri-driver is a dev-only binary.

## Dependencies

- `cargo install tauri-driver` (dev binary, CI installs it fresh; no lockfile change).
- `webkit2gtk-driver` + `xvfb` on CI runners (Ubuntu packages, zero cost).

## Success Criteria

- [ ] `make test-e2e` passes locally on Linux.
- [ ] `.github/workflows/e2e.yml` runs weekly and on `main` push, exiting green.
- [ ] All 7 scenarios (1 smoke + 6 journeys) pass against the debug binary.
- [ ] No changes to shipped app artifacts or runtime dependencies.
