# Tasks: Sprint 4 — Testing & Final Polish

## Review Workload Forecast

| Field | Value |
|-------|-------|
| Estimated changed lines | 250–400 |
| 400-line budget risk | Medium |
| Chained PRs recommended | Yes |
| Suggested split | PR 1: Rust integration tests → PR 2: E2E + frontend verification + cleanup |
| Delivery strategy | ask-on-risk |
| Chain strategy | pending |

Decision needed before apply: Yes
Chained PRs recommended: Yes
Chain strategy: pending
400-line budget risk: Medium

### Suggested Work Units

| Unit | Goal | Likely PR | Notes |
|------|------|-----------|-------|
| 1 | Rust integration test suite with ≥4 test functions | PR 1 | New `src-tauri/tests/integration_test.rs`, visibility changes, self-contained |
| 2 | E2E + frontend test verification, CI workflow, cleanup | PR 2 | Base: main; depends on PR 1 only for `make test` completeness |

## Phase 1: Rust Integration Tests

- [x] 1.1 Create `src-tauri/tests/integration_test.rs` with `use guitarhub_lib::services::*` and `use guitarhub_lib::repository::*` imports; add helper `async fn test_pool() -> SqlitePool` using `tempfile::NamedTempFile` + `SqlitePoolOptions`
- [x] 1.2 Add `pub` visibility to service constructor `new()` methods in `src-tauri/src/services/sync.rs`, `search.rs`, `price_drop.rs`, `alert_service.rs` if not already public — only the minimum needed for integration test imports
- [x] 1.3 Write `test_sync_upsert_products` — load `tests/fixtures/sample_catalog.json`, call `SyncService` upsert, assert product count in DB matches fixture
- [x] 1.4 Write `test_search_by_brand` — upsert fixture data, call `SearchService::search("Fender")`, assert results contain expected products
- [x] 1.5 Write `test_price_drop_detection` — upsert products at price A, re-sync with lower price B, call `PriceDropService`, assert detected drops with correct delta
- [x] 1.6 Write `test_notification_on_price_drop` — chain sync → price drop → `AlertService`, assert notification record created in DB
- [x] 1.7 Run `cd src-tauri && cargo test --test integration_test` and verify all ≥4 tests pass

## Phase 2: E2E Test Verification

- [~] 2.1 tauri-driver NOT INSTALLED — `tauri-driver` binary not found in PATH. Requires `cargo install tauri-driver` and system packages `webkit2gtk-driver` + `xvfb`.
- [~] 2.2 Debug binary NOT BUILT — depends on 2.1 being resolved.
- [~] 2.3–2.6 E2E tests NOT RUN — blocked by missing tauri-driver infrastructure. All unit/integration tests (Rust 343 + Python 49 + Vitest 75) pass.

## Phase 3: Frontend Vitest Verification

- [x] 3.1 Run `npx vitest run` and capture full output — document which of the 11 test files pass and which fail
- [x] 3.2 Fix any broken imports caused by Sprint 2 component extraction — update `$lib` alias paths in test files if component locations changed
- [x] 3.3 Fix any mock/stub issues from Svelte 5 runes migration — ensure `$state`/`$derived` components are testable via `@testing-library/svelte` render
- [x] 3.4 Re-run `npx vitest run` — all 11 test files must pass with 0 failures

## Phase 4: Final Cleanup

- [x] 4.1 `make test` confirms: Rust 343 ✅ + Python 49 ✅ + Vitest 75 ✅ (E2E skipped — tauri-driver not available)
- [x] 4.2 No `#[ignore]` markers, debug prints, or commented-out test code found in integration test files
- [~] 4.3 `specs/testing.yaml` does not exist — no update needed
