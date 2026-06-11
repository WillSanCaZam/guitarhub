# Tasks: Sprint 1 — Backend Alert Refactoring

## Review Workload Forecast

| Field | Value |
|-------|-------|
| Estimated changed lines | ~184 (64 additions + 120 deletions) |
| 400-line budget risk | Low |
| Chained PRs recommended | No |
| Suggested split | Single PR |
| Delivery strategy | ask-always (C1) |
| Chain strategy | pending |

Decision needed before apply: Yes
Chained PRs recommended: No
Chain strategy: pending
400-line budget risk: Low

### Suggested Work Units

| Unit | Goal | Likely PR | Notes |
|------|------|-----------|-------|
| 1 | Extract `dispatch_price_drops()` and refactor both commands | PR 1 | Single commit; tests + clippy verified inline |

## Phase 1: Foundation — Baseline Verification

- [x] 1.1 Run `cargo test` in `src-tauri/` to confirm all existing tests pass before any changes (expect 4 unit tests in `sync_command.rs` + 12 sync tests pass green)
- [x] 1.2 Run `cargo clippy -- -D warnings` in `src-tauri/` to confirm clean baseline (no pre-existing warnings)

## Phase 2: Implementation — Extract and Refactor

- [x] 2.1 Add private async function `dispatch_price_drops()` at line 161 of `src-tauri/src/commands/sync_command.rs` (before `try_build_dispatcher`), with signature: `async fn dispatch_price_drops(result: &mut SyncResult, app: &AppHandle, pool: &sqlx::SqlitePool, http_client: &reqwest::Client)` — body is the exact content currently at lines 22–82
- [x] 2.2 Replace lines 22–82 in `sync_catalog` with: `dispatch_price_drops(&mut result, &app, &state.pool, &state.http_client).await;`
- [x] 2.3 Replace lines 97–157 in `sync_local_catalog` with: `dispatch_price_drops(&mut result, &app, &state.pool, &state.http_client).await;`
- [x] 2.4 Verify both command bodies are now ~5 lines each (service call + dispatch call + Ok return)

## Phase 3: Verification — Regression Check

- [x] 3.1 Run `cargo test` in `src-tauri/` — all existing tests must pass with zero modifications
- [x] 3.2 Run `cargo clippy -- -D warnings` in `src-tauri/` — clean output, no warnings on the new function
- [x] 3.3 Run `cargo build` in `src-tauri/` — confirm clean compilation with no unused import warnings
