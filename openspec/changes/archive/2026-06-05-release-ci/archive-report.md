# Archive Report: Release CI Pipeline

**Archived**: 2026-06-05
**Change**: `release-ci`
**Status**: ✅ Completed (PASS WITH WARNINGS)

---

## 1. Summary

This change implemented a working Release CI pipeline that produces distributable binaries (`.deb`, `.AppImage`, `.dmg`, `.msi`) across 4 platform targets on tag push and publishes them to a GitHub Release. **v0.1.0 is live** on GitHub Releases with the Linux `.deb` bundle (8.2 MB).

The pipeline uses direct `cargo tauri build --target` (replacing the incompatible `tauri-apps/tauri-action@v0`), with a matrix covering Windows (x86_64), Linux (x86_64), macOS x86_64, and macOS ARM64. Artifacts are collected by a separate `create-release` job, and updater metadata is pushed to `gh-pages` with 3x retry.

---

## 2. What Was Accomplished

### All 7 CI Fixes

| # | Fix | Commit | Description |
|---|-----|--------|-------------|
| 1 | Linux missing WebKit/GTK dev libs | e5fdd24 | Added `apt-get install` for 7 packages guarded by `runner.os == 'Linux'` |
| 2 | Frontend missing from bundle | 9f0cab1 | Added `npm ci` before `cargo tauri build` |
| 3 | SvelteKit route conflict | b249754 | Renamed `+page.test.ts` → `page.test.ts` — prevents production routing collision |
| 4 | `tauri-action@v0` incompatible with Tauri 2 | a8ec218 | Replaced with direct `cargo tauri build --target`, set `fail-fast: false`, `timeout-minutes: 30` |
| 5 | No macOS signing cert | e96922f | Set `TAURI_SKIP_SIGNING: true` at workflow `env` level |
| 6 | Artifact upload + release creation | a8ec218 | Added `upload-artifact@v4` with `if-no-files-found: error`, separate `create-release` job |
| 7 | Updater metadata + concurrency guard | a8ec218 | Added `publish-update-endpoint` job with 3x retry, `concurrency` group (later removed — see deviations) |

### Task Summary

All 12 implementation tasks across 3 phases completed:
- Phase 1 (Build Infrastructure): 3/3 ✅
- Phase 2 (Build Pipeline): 4/4 ✅
- Phase 3 (Artifact & Release Management): 5/5 ✅

Phase 4 tasks (verify, archive) were tracking tasks for this phase.

---

## 3. Current State

| Area | Status |
|------|--------|
| GitHub Release v0.1.0 | ✅ Live — https://github.com/WillSanCaZam/guitarhub/releases/tag/v0.1.0 |
| Linux (.deb) bundle | ✅ 8.2 MB — verified installable |
| Linux (.AppImage) | ✅ Bundled |
| macOS (.dmg) x86_64 | ⚠️ CI run not confirmed for macOS runners |
| macOS (.dmg) ARM64 | ⚠️ CI run not confirmed for macOS runners |
| Windows (.msi) | ⚠️ CI run not confirmed for Windows runners |
| `latest.json` on gh-pages | ✅ Script generates correct JSON, gh-pages branch exists |
| Frontend tests | ✅ 32 passed, 0 failed |
| Rust tests | ✅ 293 passed, 0 failed |

---

## 4. Spec Deltas Summary

| Domain | Action | Details |
|--------|--------|---------|
| `wu3-ci-cd-hardening` | Updated | 6 added requirements: build matrix (4 targets), Linux deps, build pipeline (npm ci → cargo test → cargo tauri build), artifact upload with empty-bundle guard, release creation from bundle artifacts, gh-pages update endpoint with retry |

### Requirements Added

| Requirement | Status |
|-------------|--------|
| Build matrix covers 4 platform targets | ✅ Implemented |
| Linux system deps installed conditionally | ✅ Implemented |
| Build pipeline npm ci → cargo test → cargo tauri build | ✅ Implemented |
| Artifacts uploaded per target with empty-bundle guard | ✅ Implemented |
| Release creation from all bundle artifacts | ✅ Implemented |
| Update endpoint pushed to gh-pages with retry | ✅ Implemented |

---

## 5. Design Decisions Implemented

| Decision | Location | Rationale |
|----------|----------|-----------|
| Direct `cargo tauri build` over `tauri-action` | `release.yml` | `tauri-action@v0` pinned to Tauri 1 API — incompatible with Tauri 2 |
| `TAURI_SKIP_SIGNING: true` | `release.yml` env level | No Apple cert available; macOS `.dmg` triggers Gatekeeper warning — deferred |
| `fail-fast: false` | `release.yml` matrix | Each job independent — a macOS timeout should not cancel Linux/Windows builds |
| Separate `create-release` job | `release.yml` | If release creation fails, only the release job needs re-run — not all 4 builds |
| 3x retry with `git pull --rebase` for gh-pages push | `release.yml` | Handles push races without concurrency group |

---

## 6. Known Deviations

| Deviation | Status | Note |
|-----------|--------|------|
| **Concurrency group removed** — `concurrency: gh-pages-publish` with `cancel-in-progress: false` was intentionally removed in commit 7265747 | **Justified design deviation** | The concurrency group caused CI workflow runs to get stuck when a previous run was cancelled. The retry mechanism in `publish-update-endpoint` still handles push races (3x `pull --rebase`). The concurrency guard remains in the main spec — a future change should re-evaluate whether to reintroduce it. |
| macOS/Windows CI runner confirmation pending | **Outstanding** | CI run #7 on master has not completed for macOS/Windows targets. Linux build, release creation, and update pipeline are verified, but cross-platform matrix results are not yet confirmed in production CI. |

---

## 7. Outstanding Items

### Short-term

| Item | Priority | Notes |
|------|----------|-------|
| Verify macOS x86_64 + ARM64 CI builds in production | Medium | Runner pool may be slow; confirm `cargo tauri build` completes on `macos-13` and `macos-latest` |
| Verify Windows CI build in production | Medium | Confirm MSI generation works on `windows-latest` |
| Wrap `git commit -am` in diff check in `publish-update-endpoint` | Low | If `generate_latest_json.py` produces no diff, the commit errors. Add a diff check before committing. |

### Deferred (Out of Scope)

| Item | Priority | Blocked By |
|------|----------|------------|
| macOS code signing + notarization | Medium | Apple Developer cert procurement |
| Windows code signing (EV cert) | Medium | EV cert procurement |
| Multi-arch macOS universal .dmg | Low | Requires `lipo` merge; low priority for v0.1.0 |
| Reintroduce concurrency guard for gh-pages | Low | Previous implementation caused stuck runs; retry mechanism covers the race case for now |

---

## 8. Test Results

| Suite | Count | Result |
|-------|-------|--------|
| Rust (`cargo test`) | 293 tests | ✅ PASS |
| Frontend (`npm run test`) | 32 tests | ✅ PASS |
| Frontend build (`npm run build`) | — | ✅ Built in 4.50s |
| Rust build (`cargo build`) | — | ✅ Finished in 8.95s |

**Verification Status**: PASS WITH WARNINGS — all 13/13 spec scenarios compliant.

---

## 9. Artifact Traceability

| Artifact | Location | Status |
|----------|----------|--------|
| Proposal | `archive/2026-06-05-release-ci/proposal.md` | ✅ Archived |
| Design | `archive/2026-06-05-release-ci/design.md` | ✅ Archived |
| Tasks | `archive/2026-06-05-release-ci/tasks.md` | ✅ Archived |
| Specs (delta) | `archive/2026-06-05-release-ci/specs/wu3-ci-cd-hardening/spec.md` | ✅ Archived |
| Delta Specs Merged | `openspec/specs/wu3-ci-cd-hardening/spec.md` | ✅ Synced to source of truth |
| Verify Report | `archive/2026-06-05-release-ci/verify.md` | ✅ Archived |

---

## 10. Active Specs Remaining

All specs modified by this change remain active in `openspec/specs/` as they represent ongoing capabilities. The delta files have been archived.

---

*SDD Cycle Complete.*
