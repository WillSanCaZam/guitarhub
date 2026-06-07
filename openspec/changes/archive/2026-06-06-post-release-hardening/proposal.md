# Proposal: Post-Release Hardening

## Intent

v0.1.0 shipped but exposed critical gaps in the release pipeline and update infrastructure: no in-app updater, runaway CI blocking on scarce macOS/Windows runners, no concurrency guard on gh-pages (removed during release-ci), implicit bundle config, and 4 Dependabot major-version bumps. This change hardens the release path so the next tag push produces a usable release without manual intervention.

## Scope

### In Scope
1. Add tag-scoped concurrency group to release.yml (group: ${{ github.ref_name }}, cancel-in-progress: true)
2. Wire tauri-plugin-updater — add Rust dep, register plugin, configure endpoints
3. Add explicit `bundle` config to tauri.conf.json
4. Reduce release matrix: Linux only for now, drop macOS/Windows until runners confirm
5. Merge one Dependabot PR (httpmock 0.7.0 → 0.8.3, dev-dep only) with test validation

### Out of Scope
- macOS/Windows codesigning (Apple cert + EV cert procurement deferred)
- Cross-compilation setup
- Local macOS/Windows build testing
- Full E2E test enablement in CI
- Coverage threshold enforcement

## Capabilities

### New Capabilities
- `in-app-updater`: Tauri updater plugin integration — checks latest.json on gh-pages, notifies user on new version

### Modified Capabilities
- `wu3-ci-cd-hardening`: Concurrency group re-added (tag-scoped instead of fixed name), release matrix reduced to Linux-only

## Approach

1. **Concurrency**: Restore `concurrency` in release.yml with `group: ${{ github.ref_name }}` and `cancel-in-progress: true`. Tag name is unique per release — no false cancellation, no gh-pages race.
2. **Updater**: Add `tauri-plugin-updater = "2"` to Cargo.toml, register in main.rs, add `pub_date` and `url` to latest.json generation.
3. **Bundle config**: Add `bundle` section to tauri.conf.json with explicit Linux deb/AppImage targets, icon paths, and identifier.
4. **Release matrix**: Strip macOS/Windows from matrix, keep Linux-only. Document intent to restore once cross-platform runners are verified.
5. **Dependabot**: Merge httpmock PR, run `cargo test` to validate.

## Affected Areas

| Area | Impact | Description |
|------|--------|-------------|
| `.github/workflows/release.yml` | Modified | Concurrency group, reduced matrix |
| `src-tauri/Cargo.toml` | Modified | Add tauri-plugin-updater dep |
| `src-tauri/src/main.rs` | Modified | Register updater plugin |
| `src-tauri/tauri.conf.json` | Modified | Add bundle config, updater endpoints |
| `scripts/generate_latest_json.py` | Modified | Populate url + signature in latest.json |
| `openspec/specs/wu3-ci-cd-hardening/spec.md` | Modified | Updated concurrency + matrix requirements |

## Risks

| Risk | Likelihood | Mitigation |
|------|------------|------------|
| Updater plugin breaks existing IPC | Low | Plugin is additive, no IPC override |
| httpmock 0.8 requires MSRV > CI toolchain | Low | dtolnay/rust-toolchain@stable is 1.88+ |
| gh-pages latest.json has wrong URL format | Med | Validate JSON structure before commit |

## Rollback Plan

- **CI failure on tag**: remove tag, fix, re-tag. The concurrency group only affects in-flight runs.
- **Updater broken in release**: ship next release without updater plugin registered; users on v0.1.0 stay on v0.1.0.
- **httpmock breaks tests**: revert the single Cargo.toml line and Cargo.lock.

## Dependencies

- `tauri-plugin-updater` crate — must be compatible with Tauri 2 (current dep is `"2"`)
- `dtolnay/rust-toolchain@stable` must provide Rust ≥ 1.88 (for httpmock 0.8)

## Success Criteria

- [ ] Release workflow runs on tag push and creates a Linux release
- [ ] `tauri-plugin-updater` registered, app starts without panic
- [ ] `latest.json` contains valid `url` + `signature` for linux-x86_64
- [ ] Concurrency group present in release.yml with tag-scoped group name
- [ ] Bundle section present in tauri.conf.json with explicit targets
- [ ] httpmock 0.8.3 merged, `cargo test` passes
