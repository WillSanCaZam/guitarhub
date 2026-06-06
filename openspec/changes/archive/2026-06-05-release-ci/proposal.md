# Proposal: Release CI Pipeline

## Intent

Make `release.yml` produce distributable binaries (AppImage, .deb, .dmg, .msi) on tag push across all 4 platform targets and publish them to a GitHub Release, unblocking v0.1.0.

## Scope

### In Scope

- Linux system deps (apt) for Tauri WebKit/GTK build
- `npm ci` before `cargo tauri build` for frontend assets
- SvelteKit routing conflict fix (`+page.test.ts` rename)
- Replace `tauri-apps/tauri-action@v0` with `cargo tauri build --target`
- `TAURI_SKIP_SIGNING=true` for macOS (no Apple cert)
- Artifact upload → `gh release create` pipeline with asset discovery

### Out of Scope

- Code signing or notarization (deferred until certs obtained)
- Multi-arch bundle merging for macOS universal .dmg
- Windows code signing (deferred until EV cert)

## Capabilities

### New Capabilities

None — all changes are operational fixes to the CI pipeline, not new spec-level behaviors.

### Modified Capabilities

- `wu3-ci-cd-hardening` (concurrency guard): requirement already captured in spec. This change implements artifact build/publish mechanics as a delta below the concurrency requirement.

## Approach

Incremental debug-and-fix across 5 commits, each addressing a distinct CI failure:

1. **e5fdd24** — Linux missing WebKit/GTK dev libs → `apt-get install` 7 packages guarded by `runner.os == 'Linux'`
2. **9f0cab1** — frontend missing from bundle → `npm ci` before `cargo tauri build`
3. **b249754** — `+page.test.ts` treated as SvelteKit route → renamed to `page.test.ts`
4. **a8ec218** — `tauri-action@v0` incompatible with Tauri 2 → direct `cargo tauri build`, `fail-fast: false`, `upload-artifact@v4` + `gh release create` in separate job
5. **e96922f** — macOS runners lack signing cert → `TAURI_SKIP_SIGNING: true` at env level

## Affected Areas

| Area | Impact | Description |
|------|--------|-------------|
| `.github/workflows/release.yml` | Modified | Full rewrite of build, release, and publish jobs |
| `src/routes/__tests__/+page.test.ts` | Renamed | Renamed to `page.test.ts` — no SvelteKit routing conflict |

## Risks

| Risk | Likelihood | Mitigation |
|------|------------|------------|
| macOS runner pool contention (macos-13 for x86_64) | Med | `fail-fast: false` — other platforms complete independently |
| `gh release create` asset glob misses bundles | Low | Explicit `find` with 6 extensions, `head -20`, debug output on fail |
| gh-pages publish push race | Low | `concurrency` block with `cancel-in-progress: false` |

## Rollback

Revert the last commit. Changes are additive — no prior working state to recover, safe to iterate forward.

## Dependencies

None. Zero cost (GitHub Actions free tier + Pages).

## Success Criteria

- [ ] Tag push `v0.1.0` produces a GitHub Release with artifacts for all 4 targets
- [ ] All 4 build jobs pass in CI (Windows, Linux, macOS x86_64, macOS arm64)
- [ ] Generated bundles are loadable/installable on the target OS (verified manually post-release)
