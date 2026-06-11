# Full Codebase Audit Report — GuitarHub (2026-06-10)

## Executive Summary

**Status**: BUILDING and TESTING PASSING but with notable lint/type issues.

| Severity | Count |
|----------|-------|
| CRITICAL | 0 |
| WARNING | 15 |
| SUGGESTION | 12 |

The project compiles cleanly, all 456 tests pass (332 Rust + 75 frontend + 49 Python), and no production code contains unsafe blocks or hardcoded secrets. However there are:

- 3 Clippy errors in test code (2 unnecessary_to_string, 1 too_many_arguments)
- 32 mypy strict errors (8 in reverb adapter + 24 in test files)
- 1 svelte-check error (jest type def missing)
- 6 npm vulnerabilities (1 high severity in serialize-javascript)
- Spec drift in dashboard-data (get_recent_searches: spec says localStorage, impl uses DB)
- 0% test coverage on scraper CLI
- No integration tests in src-tauri/tests/

---

## CRITICAL ISSUES (0)

None found. The project builds and all tests pass.

---

## WARNING ISSUES (15)

### 1. Clippy errors in test code (3 errors)

**1a. Unnecessary to_string()** — `src-tauri/src/repository/sqlite/migrations/mod.rs:1660, 1664`
```
error: unnecessary use of `to_string`
1660 |             "CREATE TABLE t1 (id INTEGER PRIMARY KEY);".to_string()
     |             ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ help: use: `"CREATE TABLE t1 (id INTEGER PRIMARY KEY);"`
1664 |             "CREATE TABLE t2 (id INTEGER PRIMARY KEY);".to_string()
     |             ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ help: use: `"CREATE TABLE t2 (id INTEGER PRIMARY KEY);"`
```

**1b. Too many arguments** — `src-tauri/src/services/search.rs:383`
```
error: this function has too many arguments (9/7)
383 |     async fn insert_product_with_condition_currency(
384 |         pool: &SqlitePool,
385 |         sku: &str,
386 |         name: &str,
387 |         brand: &str,
388 |         category: &str,
389 |         price: f64,
390 |         source_id: &str,
391 |         condition: &str,
392 |         currency: &str,
393 |     ) {
```

### 2. mypy strict errors (32 errors across 3 files)

**2a. `scraper/tests/unit/test_domain.py`** (4 errors):
- Lines 61, 70, 79, 88: Missing named argument `sku`/`name`/`price`/`url` for `CatalogProduct`

**2b. `scraper/adapters/reverb.py`** (8 errors):
- Line 156: Unused `type: ignore` comment
- Line 160: Incompatible types in assignment (requests.Session vs curl_cffi session)
- Lines 169-170: `Session[Response]` has no attribute `mount`
- Line 182: Unused `type: ignore` comment
- Line 206: Returning Any from function declared to return Response
- Line 208: Missing type arguments for generic type `dict`
- Line 216: Returning Any from function declared to return `dict[Any, Any]`
- Line 222: Missing type arguments for generic type `dict`

**2c. `scraper/tests/unit/test_reverb.py`** (20 errors):
- Lines 28, 31: Missing type arguments for `dict`, returning Any
- Lines 48, 168: Cannot assign to a method (mocking pattern not type-safe)
- Lines 68, 75, 96, 113, 177, 188, 249, 259, 269, 286: Function missing type annotation for parameters
- Lines 313, 321, 329, 340, 351: union-attr errors — mock `.return_value` / `.side_effect` not recognized

### 3. Svelte-check type error (1 error)

**File**: `node_modules/@testing-library/jest-dom/types/jest.d.ts:1`
```
Error: Cannot find type definition file for 'jest'.
/// <reference types="jest" />
```
Known infra-level issue. `@testing-library/jest-dom` expects jest types even with vitest. Does NOT affect build or tests.

### 4. npm audit vulnerabilities (6 total)

| Severity | Count | Package | Issue |
|----------|-------|---------|-------|
| HIGH | 1 | serialize-javascript (via mocha > @wdio/mocha-framework) | RCE via RegExp.flags |
| MODERATE | 2 | cookie (via @sveltejs/kit) | Out of bounds characters |
| LOW | 3 | cookie (via @sveltejs/kit) | Various |

Breakthrough fixes require major version upgrades (breaking changes).

### 5. Spec drift: dashboard-data get_recent_searches

**Spec** (`openspec/specs/dashboard-data/spec.md`, line 55):
> "The `get_recent_searches` Tauri command MUST read the last 5 unique search queries from the frontend's localStorage (key: `guitarhub_recent_searches`)"

**Implementation** (`src-tauri/src/repository/dashboard.rs`, lines 39-46):
> `SELECT query FROM recent_searches ORDER BY searched_at DESC LIMIT 10`

Drift:
- Uses a database `recent_searches` table, not localStorage
- Limit is 10 (not 5)
- Storage is backend-side (not frontend)
- The spec needs updating to reflect the actual architecture

---

## SUGGESTIONS (12)

### 6. No Rust integration tests

`src-tauri/tests/` contains only `fixtures/`. No integration test files exist despite the directory being set up for them.

### 7. Python CLI has 0% test coverage

`scraper/cli.py` (87 lines) and `scraper/__main__.py` (4 lines) have zero test coverage. Overall Python coverage is 79% (581 stmts, 121 missed).

### 8. Frontend coverage limited

Vitest coverage config only covers `src/lib/`, not `src/routes/`. Route-level components (pages) are not measured.

### 9. No Rust code coverage tool installed

No `cargo-tarpaulin` or `cargo-llvm-cov` installed. Config coverage threshold in `openspec/config.yaml` is 70% but cannot be checked.

### 10. Pre-commit cargo clippy will fail

`.pre-commit-config.yaml` runs `cargo clippy --all-targets -- -D warnings` which fails on the 3 test-code issues. All commits will be blocked until fixed.

### 11. Suppressed mypy issues in pyproject.toml

Lines 22-27 disable various mypy error codes rather than fixing the underlying issues:
- `disable_error_code = ["assignment", "attr-defined", "unused-ignore", "var-annotated", "no-any-return", "type-arg"]` for reverb adapter
- `disallow_untyped_defs = false` for all tests

### 12. No type-level tests

`src/lib/types/__tests__/` does not exist. Type interfaces like `WishlistItem`, `SearchFilters`, etc. have no compile-time scenario tests.

### 13. No `.env` template completeness

`lib.rs:51` references `GUITARHUB_MIGRATIONS_DIR` env var, but `.env.example` doesn't document it.

### 14. `cargo audit` threshold at "high"

`src-tauri/.cargo-audit.toml` sets `threshold = "high"`, meaning medium/low advisories are silently ignored.

---

## Test Status

| Test Suite | Result |
|------------|--------|
| Rust `cargo test` | ✅ **332/332 PASSED** (0 failed, 0 ignored) |
| Frontend `npm test` (vitest) | ✅ **75/75 PASSED** (11 test files) |
| Python `pytest scraper/tests/ -v` | ✅ **49/49 PASSED** |
| Svelte `npm run check` | ❌ **1 error** (jest type def infra issue) |

## Build Status

| Component | Result |
|-----------|--------|
| Rust `cargo check` | ✅ Compiles clean |
| Rust `cargo clippy --lib` | ✅ Clean |
| Rust `cargo clippy --all-targets -- -D warnings` | ❌ **3 errors** (test code) |
| Python `ruff check scraper/` | ✅ All checks passed |
| Python `mypy scraper/ --strict` | ❌ **32 errors** (3 files) |

## Spec Drift Summary

| Spec | Issue | Evaluation |
|------|-------|------------|
| `dashboard-data` — get_recent_searches | localStorage vs DB, 5 vs 10 limit | Spec stale — update needed |
| `search-filters-ui` | All features matching | ✅ Clean |
| `ui-components-typing` | All 5 components typed | ✅ Clean |
| `wishlist-crud` | Repo + commands + store + page all exist | ✅ Clean |
| `local-image-cache` | HTTPS validation, MIME, domain allowlist | ✅ Clean |
| `structured-errors` | AppError enum with thiserror, Serialize | ✅ Clean |
| `sync-service` | State machine, price history, drop dispatch | ✅ Clean |
| `scraper` | Ports & Adapters, CLI, CI cron | ✅ Clean (CLI untested) |

## Security

- **No `unsafe` blocks** in Rust backend
- **No hardcoded secrets** (API keys, passwords, tokens)
- **No HTTP URLs hardcoded** — all `https://`
- **npm**: 6 known vulns (1 high via WDIO E2E deps)
- **Cargo audit**: Threshold at "high" only
- **Gitleaks**: Configured in pre-commit (not run manually for this audit)

## Git State

- Branch: `master` (up to date with `origin/master`)
- Working tree: clean
- Untracked: `.agents/`, `.claude/`, `openspec/explorations/`, `skills-lock.json` (all new/expected)
- Recent commits: last 20 are a mix of fixes, features, and chore commits through v0.3.0
