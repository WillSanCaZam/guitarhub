# Proposal: Sprint 1 — Backend Alert Refactoring

## Intent

Eliminate 60 lines of byte-for-byte duplicated alert dispatch logic in `sync_command.rs`. Both `sync_catalog` (lines 22–82) and `sync_local_catalog` (lines 97–157) contain identical orchestration: read settings, match channel, dispatch via app notification or external dispatcher, record cooldown. This is high-severity debt that costs ~1 hour to fix and zero risk to ship.

## Scope

### In Scope
- Extract shared `dispatch_price_drops()` function in `sync_command.rs`
- Replace both duplicated blocks with a single call each
- Verify `cargo test` and `cargo clippy` pass clean

### Out of Scope
- Sprint 2–4 work (frontend decomposition, optimization, testing)
- Moving alert logic into the service layer (requires `AppHandle` — architectural constraint)
- Changing alert dispatch behavior or adding new channels
- Modifying `alert_service.rs` or `dispatch_drops` / `try_build_dispatcher` helpers

## Capabilities

> Pure internal refactor — no spec-level behavior changes.

### New Capabilities
None

### Modified Capabilities
None

## Approach

Extract a private async function in `sync_command.rs`:

```rust
async fn dispatch_price_drops(
    result: &mut SyncResult,
    app: &AppHandle,
    pool: &sqlx::SqlitePool,
    http_client: &reqwest::Client,
) { ... }
```

**Parameters chosen**: `&mut SyncResult` (reads `drops`, writes `drops_sent`), `&AppHandle` (required for `tauri_plugin_notification`), `&SqlitePool` + `&reqwest::Client` (for external dispatchers and cooldown repo).

**Why not `alert_service.rs`?** The "app" channel branch calls `app.notification().builder()` which requires `AppHandle` — a Tauri runtime type unavailable in the service layer (documented in `alert_service.rs` lines 80–88). The function stays in the command layer by design.

**Error handling**: Identical to current — individual dispatch failures are logged and non-fatal. `dispatch_price_drops` never returns `Err`.

Both commands reduce to:
```rust
let mut result = service.sync_catalog(&url).await?;
dispatch_price_drops(&mut result, &app, &state.pool, &state.http_client).await;
Ok(result)
```

## Affected Areas

| Area | Impact | Description |
|------|--------|-------------|
| `src-tauri/src/commands/sync_command.rs` | Modified | Extract `dispatch_price_drops()`, replace 2 duplicated blocks (~120 lines → ~10 lines) |

## Risks

| Risk | Likelihood | Mitigation |
|------|------------|------------|
| Behavioral regression | Low | 1:1 extraction — no logic changes. Existing 4 unit tests + 12 sync tests validate. |
| Clippy warnings on new function | Low | Run `cargo clippy -- -D warnings` after refactor. |

## Rollback Plan

Single `git revert` on the refactor commit. The change is self-contained to one file with no external dependencies.

## Dependencies

None — pure internal refactor, no new crates or APIs.

## Success Criteria

- [ ] `dispatch_price_drops()` extracted as a private function in `sync_command.rs`
- [ ] `sync_catalog` and `sync_local_catalog` bodies reduced to ~5 lines each
- [ ] `cargo test` passes (all existing tests, no test modifications needed)
- [ ] `cargo clippy -- -D warnings` clean
- [ ] Zero duplicated alert dispatch lines remain
