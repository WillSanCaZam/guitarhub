# Proposal: CI Pipeline Fix — All Issues

## Intent

25 issues found across CI, data layer, security, and test coverage. 5 are CI-breaking or cause silent data loss on every sync. This change fixes all blocking issues in three priority phases so PRs pass CI and the release pipeline is reliable.

## Scope

### In Scope

- Remove stale `pnpm-lock.yaml` (dual lockfile conflict)
- Fix `specs_json` missing from batch upsert (data loss on every sync)
- Remove 8 silenced Rust security advisories from `.cargo/audit.toml`
- Fix `pip-audit` running outside virtualenv in `scrape.yml`
- Add missing system deps to `e2e.yml`
- Add `Swatinem/rust-cache` to `release.yml`
- Remove 7 `console.error` calls from production components
- Fix mypy strict mode overrides in `pyproject.toml`
- Align user-agent version (`0.2.0` → `0.3.0`)
- Complete migration chain (add 009 to `apply_full_migration_chain`)
- Add missing Svelte component and store tests
- Fix `@types/jest` / vitest type conflicts
- Fix CI `pip install` without virtualenv
- Add retry exit code to release push-with-retry
- Add missing `conftest.py` for scraper tests
- Add Python entries to `.gitignore`
- Pin Rust toolchain version
- Expand Dependabot to npm + pip ecosystems
- Add vitest coverage threshold
- Fix redundant E2E rebuild
- Clean up `reverb.py` inline import
- Remove unused `vitest.e2e.config.ts`
- Add PR and issue templates

### Out of Scope

- New features, architectural refactors, or UI changes
- Full E2E test expansion (existing spec covers this)
- Scraper feature work

## Capabilities

### New Capabilities
- `ci-pipeline-fixes`: CI workflow corrections across `ci.yml`, `scrape.yml`, `e2e.yml`, `release.yml`
- `data-layer-fixes`: Fix `specs_json` data loss in batch upsert and migration chain completeness

### Modified Capabilities
- `sync-service`: Fix `specs_json` column in batch upsert (spec-level requirement change)
- `wu3-ci-cd-hardening`: Fix `pip-audit` virtualenv, release caching, retry exit code
- `wu2-security-hardening`: Unsilence 8 audit advisories, pin Rust toolchain
- `e2e-testing`: Add missing system deps to E2E workflow
- `frontend-test-coverage`: Add 4 missing component tests + 3 missing store tests
- `wu4-repo-hygiene`: Add Python entries to `.gitignore`
- `user-agent`: Align version string to `0.3.0`

## Approach

**Phase 1 — CI-Breaking (unblock PRs):**
Remove `pnpm-lock.yaml`, fix `e2e.yml` system deps, fix `scrape.yml` pip-audit virtualenv, fix `ci.yml` pip virtualenv.

**Phase 2 — Data & Security (prevent silent failures):**
Fix `specs_json` in batch upsert, remove audit silences, pin Rust toolchain, complete migration chain, align user-agent version.

**Phase 3 — Quality & Hygiene (hardening):**
Add Rust cache to release, remove `console.error`, fix mypy overrides, add missing tests, fix type conflicts, add `.gitignore` entries, expand Dependabot, add coverage threshold, add templates.

## Affected Areas

| Area | Impact | Description |
|------|--------|-------------|
| `pnpm-lock.yaml` | Removed | Stale lockfile deleted |
| `src-tauri/src/repository/product.rs:76-100` | Modified | Add `specs_json` to INSERT |
| `.cargo/audit.toml` | Modified | Remove 8 silenced advisories |
| `.github/workflows/scrape.yml:46-49` | Modified | Activate virtualenv before pip-audit |
| `.github/workflows/e2e.yml:25-27` | Modified | Add 5 missing system deps |
| `.github/workflows/release.yml` | Modified | Add rust-cache, fix retry exit |
| `.github/workflows/ci.yml:19-20` | Modified | Use virtualenv for pip |
| `src/lib/components/ProductCard.svelte` | Modified | Remove console.error |
| `src/lib/components/Settings.svelte` | Modified | Remove console.error |
| `scraper/pyproject.toml:13-31` | Modified | Remove mypy overrides |
| `src-tauri/src/lib.rs:67` | Modified | Version 0.2.0 → 0.3.0 |
| `src-tauri/src/commands/export_command.rs:44-101` | Modified | Add migration 009 |
| `.gitignore` | Modified | Add Python entries |
| `rust-toolchain.toml` | Modified | Pin version |
| `.github/dependabot.yml` | Modified | Add npm + pip |
| `vitest.config.ts` | Modified | Add coverage threshold |
| `package.json`, `tsconfig.json` | Modified | Fix type conflicts |
| `src/lib/components/__tests__/` | Added | 4 new test files |
| `src/lib/stores/__tests__/` | Added | 3 new test files |
| `scraper/tests/conftest.py` | Added | Missing test config |
| `.github/PULL_REQUEST_TEMPLATE.md` | Added | PR template |
| `.github/ISSUE_TEMPLATE/` | Added | Issue templates |

## Risks

| Risk | Likelihood | Mitigation |
|------|------------|------------|
| Unsilencing audit reveals new blocking vulns | Medium | Review each advisory before removing; fix if actionable |
| `specs_json` fix changes upsert behavior | Low | Column already exists with DEFAULT `'{}'`; INSERT just stops ignoring it |
| Removing `console.error` hides real errors | Low | These are debug leftovers, not error handling |
| Mypy strict reveals many type errors | Medium | Remove overrides incrementally; fix errors before removing each |

## Rollback Plan

Each phase is independently revertible:
- **Phase 1**: Re-add `pnpm-lock.yaml`, revert workflow YAML changes
- **Phase 2**: Revert `product.rs`, re-add audit ignores, revert migration code
- **Phase 3**: Revert individual files; no cross-cutting dependencies

All changes are file-level; `git revert` on the merge commit restores prior state.

## Dependencies

- None (all changes are internal fixes)

## Success Criteria

- [ ] `npm ci` succeeds with single lockfile
- [ ] `cargo audit` fails on real vulnerabilities (no silences)
- [ ] `pip-audit` in `scrape.yml` audits project deps, not system
- [ ] `e2e.yml` build succeeds with all system deps
- [ ] `cargo test` passes with `specs_json` in upsert
- [ ] `npm run test` passes with all new component/store tests
- [ ] `release.yml` builds with Rust cache
- [ ] No `console.error` in production components
- [ ] mypy strict mode has no override blocks
- [ ] User-agent version matches `Cargo.toml` version
- [ ] All 25 issues resolved
