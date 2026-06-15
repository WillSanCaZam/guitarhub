# Proposal: Community Hub Wiring & Polish

## Intent

The Community Hub Integration (archived `2026-06-11`) implemented 15 Tauri commands across `auth_command.rs`, `community_command.rs`, and `profile_command.rs`, but never registered them in `main.rs` invoke_handler. The frontend stores call these commands via `invoke()` but they silently fail at runtime. Additionally, no `health_check` command exists for server connectivity, the settings page lacks community server URL configuration, and documentation is out of sync.

## Scope

### In Scope
- Register all 15 community Tauri commands in `src-tauri/src/main.rs` invoke_handler
- Implement `health_check` command for server connectivity verification
- Add community server URL field to Settings page
- Verify bundle size stays under 15MB
- Sync AGENTS.md, README.md, CHANGELOG.md with current state

### Out of Scope
- New community features (chat, DMs, real-time)
- Server backend implementation
- Content moderation system
- Mobile app adaptation

## Capabilities

### New Capabilities
- `community-wiring`: Registers all community/auth/profile Tauri commands, adds health_check, and configures server endpoint in settings

### Modified Capabilities
- `wu1-tauri-wiring`: Requirement "Tauri builder with app state and invoke handler" changes — MUST also list community, auth, and profile commands
- `ui`: Settings page gains community server URL field

## Approach

1. Add 15 command entries to `tauri::generate_handler![]` in `main.rs`
2. Implement `health_check` in `community_command.rs` — GET `{server_url}/health` with 5s timeout, returns `bool`
3. Add `community_server_url` input + "Test Connection" button to `Settings.svelte`
4. Run `cargo build --release` and verify binary size
5. Update docs: AGENTS.md (skills index if needed), README.md (features list), CHANGELOG.md

## Affected Areas

| Area | Impact | Description |
|------|--------|-------------|
| `src-tauri/src/main.rs` | Modified | Register 15 commands in invoke_handler |
| `src-tauri/src/commands/community_command.rs` | Modified | Add `health_check` command |
| `src/lib/components/Settings.svelte` | Modified | Add community server URL field + test button |
| `README.md` | Modified | Update features list |
| `CHANGELOG.md` | Modified | Add wiring entry |
| `AGENTS.md` | Modified | Sync if new conventions established |

## Risks

| Risk | Likelihood | Mitigation |
|------|------------|------------|
| Binary size exceeds 15MB | Low | These are wiring-only changes; no new deps |
| Settings test button hits wrong endpoint | Low | Use stored `community_server_url` setting key |
| Health check times out on slow networks | Medium | 5s timeout, show "unreachable" gracefully |

## Rollback Plan

Revert the single commit that adds command registrations. No schema changes, no new dependencies. Settings field is purely additive. Existing offline features unaffected.

## Dependencies

- None new. All command modules already exist.

## Success Criteria

- [ ] `cargo build --release` succeeds with zero errors
- [ ] All 15 community/auth/profile commands callable from frontend
- [ ] `health_check` command returns `true` for valid server, `false` for invalid
- [ ] Settings page shows community server URL field with test button
- [ ] Binary size < 15MB
- [ ] `make lint && make test` pass
- [ ] AGENTS.md, README.md, CHANGELOG.md synced
