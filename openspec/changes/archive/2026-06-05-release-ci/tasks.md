# Tasks: Release CI Pipeline

## Review Workload Forecast

| Field | Value |
|-------|-------|
| Estimated changed lines | ~50 |
| 400-line budget risk | Low |
| Chained PRs recommended | No |
| Suggested split | Single PR (already committed to master) |
| Delivery strategy | exception-ok |
| Chain strategy | size-exception |

Decision needed before apply: No
Chained PRs recommended: No
Chain strategy: size-exception
400-line budget risk: Low

### Suggested Work Units

| Unit | Goal | Likely PR | Notes |
|------|------|-----------|-------|
| 1 | CI/CD hardening — build pipeline, artifacts, release | Single PR | Already committed as e96922f |

## Phase 1: Build Infrastructure

- [x] 1.1 Add conditional `apt-get install` for 7 Linux WebKit/GTK deps (commit e5fdd24)
- [x] 1.2 Add `npm ci` step after `setup-node` for clean frontend dep install (commit 9f0cab1)
- [x] 1.3 Set `TAURI_SKIP_SIGNING: true` at workflow env level for macOS (commit e96922f)

## Phase 2: Build Pipeline

- [x] 2.1 Rename `+page.test.ts` to `page.test.ts` to prevent SvelteKit routing collision (commit b249754)
- [x] 2.2 Replace `tauri-apps/tauri-action@v0` with direct `cargo tauri build --target` (commit a8ec218)
- [x] 2.3 Set `fail-fast: false` on build matrix for independent job completion (commit a8ec218)
- [x] 2.4 Add 30-minute timeout per build job (commit a8ec218)

## Phase 3: Artifact & Release Management

- [x] 3.1 Add `actions/upload-artifact@v4` per target with `if-no-files-found: error` guard (commit a8ec218)
- [x] 3.2 Create separate `create-release` job with `contents: write` permission (commit a8ec218)
- [x] 3.3 Add asset discovery via `find` with 6 extensions, `gh release create --generate-notes` (commit a8ec218)
- [x] 3.4 Add `publish-update-endpoint` job with gh-pages push and 3x retry (commit a8ec218)
- [x] 3.5 Add `concurrency: gh-pages-publish` with `cancel-in-progress: false` guard (commit a8ec218)

## Phase 4: Verification & Archive

- [ ] 4.1 Verify: wait for CI run #7 on master to complete (all 4 builds + release + update)
- [ ] 4.2 Archive: if CI passes, run sdd-archive to close the change
