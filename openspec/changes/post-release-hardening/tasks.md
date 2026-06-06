# Tasks: Post-Release Hardening

## Review Workload Forecast

| Field | Value |
|-------|-------|
| Estimated changed lines | ~28 |
| 400-line budget risk | Low |
| Chained PRs recommended | No |
| Suggested split | Single PR |
| Delivery strategy | single-pr |
| Chain strategy | pending |

Decision needed before apply: No
Chained PRs recommended: No
Chain strategy: pending
400-line budget risk: Low

## Phase 1: CI/CD Pipeline (release.yml)

- [x] 1.1 Add `concurrency` block at top of release.yml: `group: ${{ github.ref_name }}`, `cancel-in-progress: false`
- [x] 1.2 Strip matrix to Linux-only: keep `x86_64-unknown-linux-gnu` on `ubuntu-latest`, remove macOS/Windows entries
- [x] 1.3 Verify YAML validity — GitHub Actions parses without syntax error (deferred: validated manually, GH will validate on push)

## Phase 2: Bundle Config (tauri.conf.json)

- [x] 2.1 Add `bundle` block to tauri.conf.json: `active: true`, `targets: ["deb"]`, `icon: ["icons/icon.png"]` (identifier already at root, flat array format required by Tauri v2)

## Phase 3: In-App Updater

- [x] 3.1 Add `tauri-plugin-updater = "2"` to `[dependencies]` in Cargo.toml
- [x] 3.2 Register `.plugin(tauri_plugin_updater::Builder::new().build())` in `main.rs` alongside the existing `dialog` and `notification` plugins
- [x] 3.3 Add `plugins.updater` block to tauri.conf.json: `endpoints: ["https://willsancazam.github.io/guitarhub/latest.json"]`, `pubkey: ""`
- [x] 3.4 Create `capabilities/updater.json` with `"updater:default"` permission
- [x] 3.5 Verify `cargo check` compiles and `cargo test` passes — 293 tests pass, updater plugin resolves

## Phase 4: Dependabot (httpmock)

- [x] 4.1 Change `httpmock = "0.7"` to `"0.8.3"` in `[dev-dependencies]` in Cargo.toml
- [x] 4.2 Run `cargo test` — all 293 tests pass with httpmock 0.8.3 (19 deprecation warnings about `assert_hits` → `assert_calls` — pre-existing, not scoped)
