# Tasks: Clippy Fix

## Review Workload Forecast

| Field | Value |
|-------|-------|
| Estimated changed lines | ~40–60 |
| 400-line budget risk | Low |
| Chained PRs recommended | No |
| Suggested split | Single PR |
| Delivery strategy | ask-on-risk |
| Chain strategy | pending |

Decision needed before apply: No
Chained PRs recommended: No
Chain strategy: pending
400-line budget risk: Low

## Phase 1: Fix Clippy Errors

- [x] 1.1 — `src-tauri/src/repository/sqlite/migrations/mod.rs`: Remove `.to_string()` from the two `&str` literals at lines ~1660 and 1664
- [x] 1.2 — `src-tauri/src/services/search.rs`: Add `ProductTestParams<'a>` struct before the test helper function (no Default — `&SqlitePool` can't implement Default)
- [x] 1.3 — `src-tauri/src/services/search.rs`: Collapse `insert_product_with_condition_currency` signature from 9 positional params to single `ProductTestParams<'_>` param
- [x] 1.4 — `src-tauri/src/services/search.rs`: Update all 9 call sites to use struct init syntax

## Phase 2: Verification

- [x] 2.1 — Run `cargo clippy --all-targets -- -D warnings` — must exit 0
- [x] 2.2 — Run `cargo test` — all tests must pass

## Phase 3: Commit

- [ ] 3.1 — Commit both files together with conventional commit message
