# Archive Report: Post-Release Hardening

**Archived**: 2026-06-06
**Commit**: 60efe9e
**SDD Cycle**: Complete (proposal → spec → design → tasks → apply → verify → archive)

---

## Summary

v0.1.0 shipped but exposed critical gaps in the release pipeline and update infrastructure. This change hardened the release path across three independent subsystems: CI/CD pipeline guardrails (concurrency, matrix), in-app updater integration (tauri-plugin-updater), and a Dependabot merge (httpmock 0.7 → 0.8.3). All 10 tasks were applied, verified, and committed.

## What Was Accomplished

### Phase 1: CI/CD Pipeline (release.yml)
- ✅ 1.1 — Tag-scoped concurrency block: `group: ${{ github.ref_name }}`, `cancel-in-progress: false`
- ✅ 1.2 — Build matrix reduced to Linux-only: `x86_64-unknown-linux-gnu` on `ubuntu-latest`
- ✅ 1.3 — YAML validity verified manually (no syntax errors)

### Phase 2: Bundle Config (tauri.conf.json)
- ✅ 2.1 — Added `bundle` block: `active: true`, `targets: ["deb"]`, `icon: ["icons/icon.png"]`

### Phase 3: In-App Updater
- ✅ 3.1 — Added `tauri-plugin-updater = "2"` to `[dependencies]` in Cargo.toml
- ✅ 3.2 — Registered `.plugin(tauri_plugin_updater::Builder::new().build())` in `main.rs`
- ✅ 3.3 — Added `plugins.updater` block in tauri.conf.json with endpoints and empty pubkey
- ✅ 3.4 — Created `capabilities/updater.json` with `"updater:default"` permission
- ✅ 3.5 — `cargo check` compiles, `cargo test` passes (293 tests)

### Phase 4: Dependabot (httpmock)
- ✅ 4.1 — Upgraded `httpmock` from `"0.7"` to `"0.8.3"` in dev-dependencies
- ✅ 4.2 — All 293 tests pass (19 pre-existing deprecation warnings)

## Current State

| Metric | Value |
|--------|-------|
| Tasks total | 10 |
| Tasks complete | 10 |
| Tasks incomplete | 0 |
| Build | ✅ `cargo check` compiles cleanly |
| Tests | ✅ 293 Rust + 32 frontend tests passing |
| Verdict | **PASS WITH WARNINGS** |

### Files Changed

| File | Action |
|------|--------|
| `.github/workflows/release.yml` | Modified — concurrency group, reduced matrix |
| `src-tauri/Cargo.toml` | Modified — added `tauri-plugin-updater` dep, `httpmock` upgrade |
| `src-tauri/src/main.rs` | Modified — registered updater plugin |
| `src-tauri/tauri.conf.json` | Modified — added `bundle` config, `plugins.updater` block |
| `src-tauri/capabilities/updater.json` | Created — updater permission |

## Outstanding Items

| Item | Severity | Details |
|------|----------|---------|
| Empty `pubkey` | ⚠️ WARNING | `pubkey: ""` in tauri.conf.json — updater compiles but will fail signature verification at runtime. Signing key generation deferred. |
| CI not yet executed | ⚠️ WARNING | The updated release.yml workflow has not run against a tag push. Actual CI behavior (concurrency, build, artifact upload, latest.json generation) is unexercised. |
| macOS/Windows builds | ℹ️ DEFERRED | Scoped out per proposal. Linux-only for now; cross-platform runners need verification. |
| Notification test coverage | ℹ️ UNTESTED | Two scenarios (notification shown/not shown per version comparison) require Tauri runtime integration tests — out of scope. |

## Artifact References

| Artifact | Path |
|----------|------|
| Proposal | `openspec/changes/archive/2026-06-06-post-release-hardening/proposal.md` |
| Spec — In-App Updater | `openspec/changes/archive/2026-06-06-post-release-hardening/specs/in-app-updater/spec.md` |
| Spec — CI/CD Hardening (delta) | `openspec/changes/archive/2026-06-06-post-release-hardening/specs/wu3-ci-cd-hardening/spec.md` |
| Design | `openspec/changes/archive/2026-06-06-post-release-hardening/design.md` |
| Tasks | `openspec/changes/archive/2026-06-06-post-release-hardening/tasks.md` |
| Verify | `openspec/changes/archive/2026-06-06-post-release-hardening/verify.md` |
| Archive | `openspec/changes/archive/2026-06-06-post-release-hardening/archive.md` |

## Source of Truth Updated

| Domain | Action | Details |
|--------|--------|---------|
| `wu3-ci-cd-hardening` | Updated | Concurrency group changed to tag-scoped; build matrix reduced to Linux-only; added bundle config and httpmock upgrade requirements |
| `in-app-updater` | Created | New spec for Tauri updater plugin integration with 7 requirements |

## SDD Cycle Complete

The post-release-hardening change has been fully planned, specified, designed, implemented, verified, and archived. Ready for the next change.
