# Proposal: Fix blocking audit errors

## Intent

The project has accumulated lint errors, type errors, and spec drift that block pre-commit hooks and create confusion between documented behavior and actual code. Fix these to unblock development workflow and restore spec accuracy — no new features, no style-only changes.

## Scope

### In Scope
- 2 Clippy errors in test code (unnecessary `.to_string()` calls, `too_many_arguments`)
- 32 mypy strict errors across 3 Python files (reverb adapter + 2 test files)
- Spec drift in `dashboard-data`: `get_recent_searches` documents localStorage but impl uses DB with LIMIT 10
- Remove suppressed mypy error codes from `pyproject.toml` that the fixes make redundant

### Out of Scope
- Svelte-check jest type def issue (infra-level, not a project bug)
- npm audit vulnerabilities (require breaking upgrades)
- Rust integration tests, Python CLI coverage, cargo-coverage setup
- Style/linter suggestions with no blocking effect

## Capabilities

### New Capabilities
None

### Modified Capabilities
- `dashboard-data`: `get_recent_searches` requirement — source changes from frontend localStorage to backend `recent_searches` table, limit changes from 5 to 10

## Approach

Three independent work streams, order by CI-blocking severity:

1. **Clippy fixes** — Remove `.to_string()` from two string literals in `migrations/mod.rs`. Extract `ProductTestParams` struct or add `#[allow(clippy::too_many_arguments)]` to the test helper in `search.rs`.
2. **Spec correction** — Rewrite the `get_recent_searches` requirement block in `dashboard-data/spec.md` to match `repository/dashboard.rs` (DB-backed, LIMIT 10, no dedup). Update scenarios accordingly.
3. **mypy strict fixes** — Annotate all untyped params/returns in `reverb.py` and both test files. Replace `type: ignore` with proper types. Fix `CatalogProduct` calls with named args. Remove now-unnecessary `pyproject.toml` suppressions.

## Affected Areas

| Area | Impact | Description |
|------|--------|-------------|
| `src-tauri/src/repository/sqlite/migrations/mod.rs` | Modified | Remove `.to_string()` on lines 1660, 1664 |
| `src-tauri/src/services/search.rs` | Modified | Reduce args or allow clippy on test helper |
| `openspec/specs/dashboard-data/spec.md` | Modified | Align get_recent_searches with DB impl |
| `scraper/adapters/reverb.py` | Modified | Fix 8 mypy strict errors |
| `scraper/tests/unit/test_reverb.py` | Modified | Fix 20 mypy strict errors |
| `scraper/tests/unit/test_domain.py` | Modified | Fix 4 mypy strict errors |
| `pyproject.toml` | Modified | Remove suppressions fixed by above |

## Risks

| Risk | Likelihood | Mitigation |
|------|------------|------------|
| Clippy fix changes semantics of `write()` path arg | Low | String literals — `to_string()` is redundant; `AsRef<Path>` accepts `&str` directly |
| mypy fix introduces new type errors elsewhere | Low | Run `mypy scraper/ --strict` after each file change |
| Spec correction misses a consumption site | Low | Search all files for `get_recent_searches` references |

## Rollback Plan

Each fix is a single-file, revertible change. If any fix breaks CI:
- Revert the specific commit
- Open an issue with the error output
- All three streams are independent — rollback one does not block others

## Dependencies

None

## Success Criteria

- [ ] `cargo clippy --all-targets -- -D warnings` passes with zero errors
- [ ] `mypy scraper/ --strict` passes with zero errors
- [ ] `get_recent_searches` spec accurately describes DB-backed impl (LIMIT 10)
- [ ] All existing tests still pass (`cargo test`, `pytest`, `npm test`)
