# Design: CI Pipeline Fix — All Issues

## Technical Approach

Three-phase implementation fixing 25 issues. Phase 1 unblocks CI (5 items), Phase 2 prevents silent data loss (4 items), Phase 3 hardens quality (11+ items). Each phase is independently revertible via `git revert` on its merge commit.

## Architecture Decisions

### Decision: Lockfile Cleanup

| Option | Tradeoff | Decision |
|--------|----------|----------|
| Delete `pnpm-lock.yaml` only | Fast, minimal. Assumes `package-lock.json` is correct. | ✅ |
| Regenerate both lockfiles | Guarantees consistency. Slower, risks changing dependency versions. | Rejected |
| Migrate to pnpm entirely | Clean single-lockfile story. High effort, out of scope. | Rejected |

**Rationale**: The file is stale pnpm artifact. `npm ci` already uses `package-lock.json` successfully. Delete and verify `npm ci` passes.

### Decision: Mypy Override Reduction

| Option | Tradeoff | Decision |
|--------|----------|----------|
| Remove all 3 overrides at once | Fast. If reverb adapter has real type issues, CI breaks. | Rejected |
| Remove overrides incrementally per module | Safe. Each removal runs `mypy --strict` to verify. | ✅ |
| Fix all type errors first, then remove | Thorough. But reverb adapter type work is out of scope. | Rejected |

**Rationale**: Remove test overrides (safe), domain override (safe — `call-arg` is a known false positive with dataclasses), reverb override last after verifying. If reverb fails, keep that one override and document it.

### Decision: Cargo Audit Unsilencing

| Option | Tradeoff | Decision |
|--------|----------|----------|
| Remove all 8 ignores at once | Clean. If any advisory is actionable, CI blocks. | ✅ (with review) |
| Remove one at a time | Safe but slow (8 CI runs). | Rejected |
| Review each advisory before removing | Thorough. Requires external research per CVE. | ✅ (pre-work) |

**Rationale**: Run `cargo audit` locally without ignores first. If all 8 advisories are informational/unmaintained (common for transitive deps), remove them all. If any are actual vulnerabilities, fix the dep or re-add a targeted ignore with a TODO.

### Decision: Testing Strategy

| Layer | What to Test | Approach |
|-------|-------------|----------|
| Unit | `product.rs` batch upsert with `specs_json` | Existing tests already cover this — verify INSERT column list includes `specs_json` |
| Unit | Component Svelte tests (4 new) | Mock `invoke`, test render + click handlers |
| Unit | Store tests (3 new) | Mock `invoke`, test store functions |
| Integration | Migration 009 chain | `cargo test` runs `apply_full_migration_chain` |
| E2E | CI workflow validation | Push to branch, verify all 3 workflow YAMLs pass |
| CI | Pipeline smoke test | PR triggers `ci.yml`, `e2e.yml` |

## Data Flow — CI Pipeline Fixes

    PR opened
       │
       ├─→ ci.yml (python, frontend, rust jobs)
       │     ├─ python: pip install → ruff → mypy → pip-audit → pytest
       │     ├─ frontend: npm ci → svelte-kit sync → test → build → check
       │     └─ rust: cargo clippy → cargo test → cargo audit
       │
       ├─→ e2e.yml (system deps → npm ci → tauri build → xvfb-run test:e2e)
       │
       └─→ scrape.yml (venv → pip install → pip-audit → scraper → validate → deploy)

## File Changes

| File | Action | Description |
|------|--------|-------------|
| `pnpm-lock.yaml` | Delete | Remove stale dual-lockfile conflict |
| `.github/workflows/ci.yml` | Modify | Use virtualenv for pip install + pip-audit |
| `.github/workflows/e2e.yml` | Modify | Add libgtk-3-dev, libayatana-appindicator3-dev, librsvg2-dev |
| `.github/workflows/scrape.yml` | Modify | Activate venv before pip-audit step |
| `.github/workflows/release.yml` | Modify | Add Swatinem/rust-cache, fix retry exit code |
| `src-tauri/src/repository/product.rs` | Modify | Add `specs_json` to INSERT column list |
| `.cargo/audit.toml` | Modify | Remove 8 silenced advisories |
| `scraper/pyproject.toml` | Modify | Remove/reduce mypy overrides |
| `src-tauri/src/lib.rs` | Modify | Update user-agent `0.2.0` → `0.3.0` |
| `src-tauri/src/commands/export_command.rs` | Modify | Add migration 009 to `migrated_pool()` helper |
| `src-tauri/src/repository/sqlite/migrations/mod.rs` | Modify | Add 008 + 009 to `apply_full_migration_chain` |
| `src/lib/components/Settings.svelte` | Modify | Remove 5 `console.error` calls |
| `src/lib/components/ProductCard.svelte` | Modify | Remove 2 `console.error` calls |
| `src/lib/components/__tests__/ProductDetail.test.ts` | Create | Component test |
| `src/lib/components/__tests__/SearchPanel.test.ts` | Create | Component test |
| `src/lib/components/__tests__/CollectionStatsCell.test.ts` | Create | Component test |
| `src/lib/components/__tests__/SyncStatusCell.test.ts` | Create | Component test |
| `src/lib/stores/__tests__/wishlist.test.ts` | Create | Store test |
| `src/lib/stores/__tests__/sync.test.ts` | Create | Store test |
| `src/lib/stores/__tests__/dashboard.test.ts` | Create | Store test |
| `scraper/tests/conftest.py` | Create | Shared fixtures for scraper tests |
| `.gitignore` | Modify | Add `.venv/`, `*.egg-info/`, `.mypy_cache/` |
| `rust-toolchain.toml` | Modify | Pin `channel = "1.85.0"` |
| `.github/dependabot.yml` | Modify | Add npm + pip ecosystems |
| `vitest.config.ts` | Modify | Add coverage thresholds (lines 80%) |
| `package.json` | Modify | Remove `@types/jest`, verify vitest types |
| `tsconfig.json` | Modify | Fix type conflicts with vitest |
| `.github/PULL_REQUEST_TEMPLATE.md` | Create | PR template |
| `.github/ISSUE_TEMPLATE/bug_report.md` | Create | Bug report template |
| `.github/ISSUE_TEMPLATE/feature_request.md` | Create | Feature request template |
| `vitest.e2e.config.ts` | Delete | Unused config |
| `src/lib/components/Reverb.svelte` | Modify | Remove inline import (if exists) |

## Interfaces / Contracts

No new interfaces. The `specs_json` fix changes the INSERT statement — column already exists with `DEFAULT '{}'`, so this is additive, not breaking:

```sql
-- Before: specs_json missing from INSERT (defaults to '{}')
-- After: specs_json included in INSERT
INSERT INTO products_meta (..., specs_json, ...)
VALUES (?, ?, ..., ?16, ...)
```

## Migration / Rollout

- **Migration 009**: Already exists as `009_add_recent_searches.sql`. Only needs to be added to test helpers (`apply_full_migration_chain` and `export_command::migrated_pool`). No production migration needed — the file is already in the migrations directory.
- **Feature flags**: None. All changes are file-level.
- **Rollback**: Each phase is a separate PR/merge commit. `git revert` restores prior state.

## Risk Assessment

### Phase 1 — CI-Breaking (5 items)

| Risk | Likelihood | Mitigation |
|------|------------|------------|
| `pnpm-lock.yaml` deletion breaks something | Low | Only `npm ci` is used in CI; pnpm is not configured |
| e2e.yml missing deps cause build failure | Low | Copy dep list from release.yml which already works |
| scrape.yml venv activation fails in CI | Low | Pattern already used in same workflow for other steps |

### Phase 2 — Data & Security (4 items)

| Risk | Likelihood | Mitigation |
|------|------------|------------|
| Unsilencing audit reveals blocking vuln | Medium | Run `cargo audit` locally without ignores first |
| `specs_json` INSERT changes upsert behavior | Low | Column already has DEFAULT; INSERT just stops ignoring it |
| User-agent version mismatch | Low | Change one string literal; verify `Cargo.toml` version matches |

### Phase 3 — Quality (11+ items)

| Risk | Likelihood | Mitigation |
|------|------------|------------|
| Mypy strict reveals many type errors | Medium | Remove overrides incrementally; fix errors per module |
| New component tests are flaky | Low | Follow existing test patterns (mock `invoke`) |
| Removing console.error hides real errors | Low | These are debug leftovers; errors are handled by try/catch |

## Testing Strategy

| Layer | What to Test | Approach |
|-------|-------------|----------|
| Unit | product.rs `specs_json` in INSERT | `cargo test` — existing tests validate upsert |
| Unit | Component tests (4 new) | `npm run test` — vitest + jsdom |
| Unit | Store tests (3 new) | `npm run test` — mock `invoke` calls |
| Unit | Migration chain completeness | `cargo test` — `apply_full_migration_chain` includes 008+009 |
| Integration | CI workflow validation | Push to PR branch; all 3 workflow YAMLs must pass |
| Integration | mypy strict mode | `mypy --config-file scraper/pyproject.toml scraper/` |
| Security | cargo audit | `cargo audit` in `src-tauri/` with no ignores |
| E2E | Full pipeline | PR triggers ci.yml, e2e.yml; merge triggers release.yml |

## Open Questions

- [ ] Which of the 8 RUSTSEC advisories are actionable vs informational? Need to run `cargo audit` locally without ignores to determine.
- [ ] Does removing the reverb adapter mypy override cause failures? Need to run `mypy --strict` on that module.
- [ ] Are the 4 component tests and 3 store tests the right set? The proposal mentions "4 component tests + 3 store tests" but doesn't name them — I've identified the most likely candidates based on existing test coverage gaps.
