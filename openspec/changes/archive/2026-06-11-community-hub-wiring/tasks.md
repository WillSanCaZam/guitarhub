# Tasks: Community Hub Wiring & Polish

## Review Workload Forecast

| Field | Value |
|-------|-------|
| Estimated changed lines | 80–120 |
| 400-line budget risk | Low |
| Chained PRs recommended | No |
| Suggested split | Single PR |
| Delivery strategy | ask-on-risk |
| Chain strategy | pending |

Decision needed before apply: No
Chained PRs recommended: No
Chain strategy: pending
400-line budget risk: Low

## Phase 1: Command Registration

- [x] 1.1 Add 15 command imports to `src-tauri/src/main.rs`: `auth_command::{register, login, get_current_user, logout, refresh_token}`, `community_command::{get_feed, create_lesson, get_lesson, like_content, add_comment, get_comments}`, `profile_command::{get_profile, update_profile, get_streak, add_gear_to_list}`
- [x] 1.2 Register all 15 in the `tauri::generate_handler![]` macro after the existing `wishlist_command::get_wishlist` entry
- [x] 1.3 Verify: `make lint-rust` passes with zero clippy warnings

## Phase 2: Health Check Command

- [x] 2.1 Add `health_check` function in `src-tauri/src/commands/community_command.rs` — GET `{server_url}/health` with 5s reqwest timeout, returns `Result<bool, AppError>`
- [x] 2.2 Register `community_command::health_check` in `main.rs` invoke_handler (same file as Phase 1)
- [x] 2.3 Verify: `cargo check` succeeds, function compiles with correct `#[tauri::command]` attribute

## Phase 3: Settings UI Wiring

- [x] 3.1 Add `communityServerUrl` state + load from `get_setting('community_server_url')` in `onMount` block of `src/lib/components/Settings.svelte`
- [x] 3.2 Add "Community Server" `<fieldset>` with text input bound to `communityServerUrl` and a "Test Connection" button that invokes `health_check`
- [x] 3.3 Wire `saveAll()` to also persist `community_server_url`
- [x] 3.4 Show test result indicator (green/red) below the input — mirrors existing `testResult` pattern

## Phase 4: Bundle Size & Final Verification

- [x] 4.1 Run `cargo build --release` and verify binary size < 15MB
- [x] 4.2 Run `make test && make lint` — all checks must pass
- [x] 4.3 Update `CHANGELOG.md` with community-hub-wiring entry under Unreleased
