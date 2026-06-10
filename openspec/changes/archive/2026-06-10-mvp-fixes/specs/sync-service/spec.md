# Delta for sync-service

> **Change**: mvp-fixes — Panic fixes C1–C3

## MODIFIED Requirements

### Requirement: SyncService trait MUST be defined

The system MUST define a `SyncService` trait with method `async fn sync_catalog(&self, url: &str) -> Result<SyncResult, AppError>`. `SyncResult` MUST contain `products_loaded: u32`, `products_updated: u32`, `state: SyncState`, and `progress: f32`. All `SystemTime::now().duration_since(UNIX_EPOCH)` calls in `sync_command.rs` and `sync.rs` MUST use `unwrap_or_default()` instead of `unwrap()` to prevent panics on clock anomalies.

(Previously: Requirement allowed `unwrap()` on `SystemTime::now().duration_since(UNIX_EPOCH)` at `sync_command.rs:30` and `sync.rs:55,112-115`.)

#### Scenario: Sync completes despite clock being before UNIX_EPOCH

- GIVEN the system clock returns a time before `UNIX_EPOCH`
- WHEN `sync_catalog` or `sync_command` computes a timestamp
- THEN the timestamp defaults to `0` instead of panicking
- AND sync proceeds without crash

#### Scenario: Sync completes normally with valid system clock

- GIVEN the system clock returns a normal time
- WHEN `sync_catalog` or `sync_command` computes a timestamp
- THEN the timestamp is the correct number of seconds since `UNIX_EPOCH`
- AND behavior is identical to the previous `unwrap()` path

#### Scenario: Alert dispatch timestamp is safe

- GIVEN drops are detected and `alert_channel` is configured
- WHEN `sync_command` builds the `now` timestamp for cooldown recording
- THEN `now` is computed with `unwrap_or_default()` — never panicking

#### Scenario: State machine transitions use safe timestamps

- GIVEN the sync state machine transitions through `downloading → validating → sanitizing → inserting → done`
- WHEN `set_state` is called to record `last_synced`
- THEN `last_synced` is computed with `unwrap_or_default()` — never panicking