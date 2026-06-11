# Proposal: Sprint 4 — Testing & Final Polish

## Intent

Close the testing gaps before MVP release. The backend has 341 inline unit tests but zero integration tests. E2E tests exist (7 specs) but have never been confirmed passing. Frontend vitest suite (11 files) needs verification. This sprint establishes confidence that the full stack works end-to-end.

## Scope

### In Scope
- Rust integration tests in `src-tauri/tests/` covering sync, search, collection, and wishlist services
- E2E test verification: run all 7 WebdriverIO specs locally, fix failures, confirm CI workflow
- Frontend test verification: confirm all 11 vitest files pass, add missing coverage if needed
- Makefile target for full test orchestration (`make test` runs all three suites)

### Out of Scope
- New E2E test scenarios (7 existing specs cover the critical paths)
- Performance/load testing
- macOS or Windows E2E (tauri-driver Linux-only)
- Frontend component tests beyond existing 11 files (unless gaps found)

## Capabilities

> Contract with sdd-spec phase.

### New Capabilities
- `rust-integration-tests`: Integration test suite in `src-tauri/tests/` exercising service layer with a test SQLite database

### Modified Capabilities
- `e2e-testing`: Verify existing specs pass; fix flaky waits or tauri-driver compatibility issues
- `frontend-testing`: Verify existing vitest suite passes; fix any Svelte 5 compatibility issues

## Approach

| Area | Strategy |
|------|----------|
| Rust integration | `#[tokio::test]` with ephemeral SQLite DB; exercise `SyncService`, `SearchService`, collection/wishlist CRUD directly |
| E2E verification | Install `tauri-driver`, run `npm run test:e2e`, fix failures iteratively |
| Frontend verification | Run `npm run test`, fix any failures, confirm CI workflow includes frontend job |

### Rust Integration Test Design
- Create `src-tauri/tests/integration_test.rs`
- Use `tempfile` for isolated test databases
- Test the service layer (not Tauri commands) to avoid IPC overhead
- May need `pub` visibility changes on service structs in `lib.rs`

## Affected Areas

| Area | Impact | Description |
|------|--------|-------------|
| `src-tauri/tests/` | New | Integration test files |
| `src-tauri/src/lib.rs` | Modified | Potential `pub` visibility for test imports |
| `src-tauri/Cargo.toml` | Modified | Add `tempfile` dev-dependency |
| `e2e-tests/specs/` | Modified | Fix any failing specs |
| `.github/workflows/e2e.yml` | Modified | Ensure workflow is correct |
| `Makefile` | Modified | Verify `make test` orchestrates all suites |

## Risks

| Risk | Likelihood | Mitigation |
|------|------------|------------|
| `pub` visibility changes break encapsulation | Low | Only expose what integration tests need; use `#[cfg(test)]` where possible |
| tauri-driver version mismatch | Med | Pin version in CI; document local install steps |
| E2E flaky waits | Med | Use `waitForExist` with explicit timeouts; avoid `browser.pause()` |
| Svelte 5 + testing-library issues | Low | Already working in existing tests; fix incrementally |

## Rollback Plan

- Integration tests are additive — delete `src-tauri/tests/*.rs` to revert
- E2E fixes are isolated to spec files — `git checkout` original specs
- No production code changes expected beyond visibility modifiers

## Dependencies

- `tauri-driver` must be installed locally (`cargo install tauri-driver`)
- System packages: `webkit2gtk-driver`, `xvfb` (for headless E2E)

## Success Criteria

- [ ] `cargo test --test integration_test` passes with ≥4 test functions
- [ ] `npm run test:e2e` passes all 7 specs locally
- [ ] `.github/workflows/e2e.yml` passes in CI
- [ ] `npm run test` passes all 11 frontend test files
- [ ] `make test` runs all three suites and exits 0 on success
