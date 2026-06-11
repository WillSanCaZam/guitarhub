# Tasks: CI Pipeline Fix — All Issues

## Review Workload Forecast

| Field | Value |
|-------|-------|
| Estimated changed lines | 350–450 |
| 400-line budget risk | Medium |
| Chained PRs recommended | Yes |
| Suggested split | PR 1 (Phase 1) → PR 2 (Phase 2) → PR 3 (Phase 3) |
| Delivery strategy | ask-on-risk |
| Chain strategy | stacked-to-main |

Decision needed before apply: Yes
Chained PRs recommended: Yes
Chain strategy: stacked-to-main
400-line budget risk: Medium

### Suggested Work Units

| Unit | Goal | Likely PR | Notes |
|------|------|-----------|-------|
| 1 | CI-Breaking Fixes | PR 1 | Unblocks all PRs; independent merge |
| 2 | Data & Security Fixes | PR 2 | Depends on PR 1 green; prevents data loss |
| 3 | Quality & Hygiene | PR 3 | Largest slice; hardening only |

## Phase 1: CI-Breaking Fixes

- [x] 1.1 Delete `pnpm-lock.yaml`. Add `.venv/`, `*.egg-info/`, `.mypy_cache/` to `.gitignore`. Verify `npm ci` passes.
- [x] 1.2 Fix `.github/workflows/e2e.yml`: add `libgtk-3-dev`, `libayatana-appindicator3-dev`, `librsvg2-dev` to apt-get install.
- [x] 1.3 Fix `.github/workflows/scrape.yml`: activate virtualenv before pip-audit step (add `source .venv/bin/activate`).
- [x] 1.4 Fix `.github/workflows/ci.yml`: use virtualenv for pip install + pip-audit in Python job.
- [x] 1.5 Add `Swatinem/rust-cache@v2` to `.github/workflows/release.yml`.

## Phase 2: Data & Security Fixes

- [ ] 2.1 Add `specs_json` to INSERT column list in `src-tauri/src/repository/product.rs` (batch upsert).
- [ ] 2.2 Review `.cargo/audit.toml`: remove silenced advisories that are informational/unmaintained. Run `cargo audit` locally to confirm.
- [ ] 2.3 Audit `scraper/pyproject.toml` mypy overrides: remove test overrides, verify reverb override still needed.
- [ ] 2.4 Update user-agent version `0.2.0` → `0.3.0` in `src-tauri/src/lib.rs`. Verify matches `Cargo.toml`.

## Phase 3: Quality & Hygiene

- [x] 3.1 Add `exit 1` to release.yml retry loop so failures propagate.
- [x] 3.2 Remove `console.error` calls from `src/lib/components/ProductCard.svelte` and `src/lib/components/Settings.svelte`.
- [x] 3.3 Fix export test migration chain: add migration 009 to `src-tauri/src/commands/export_command.rs` and `src-tauri/src/repository/sqlite/migrations/mod.rs` `apply_full_migration_chain`.
- [x] 3.4 Add component tests: `src/lib/components/__tests__/CollectionStatsCell.test.ts`, `ProductDetail.test.ts`, `SyncStatusCell.test.ts`, `SearchPanel.test.ts`.
- [x] 3.5 Add store tests: `src/lib/stores/__tests__/dashboard.test.ts`, `sync.test.ts`, `wishlist.test.ts`.
- [x] 3.6 Pin Rust toolchain in `rust-toolchain.toml` to `channel = "1.85.0"`.
- [x] 3.7 Add `npm` and `pip` ecosystems to `.github/dependabot.yml`.
- [x] 3.8 Add vitest coverage thresholds (lines 80%) to `vitest.config.ts`.
- [x] 3.9 Create `.github/PULL_REQUEST_TEMPLATE.md` and `.github/ISSUE_TEMPLATE/` (bug_report.md, feature_request.md).
- [x] 3.10 Remove unused `vitest.e2e.config.ts`.
- [ ] 3.11 Fix `@types/jest` / vitest type conflicts in `package.json` and `tsconfig.json`.
