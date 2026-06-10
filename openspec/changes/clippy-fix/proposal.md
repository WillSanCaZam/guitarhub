# Proposal: Clippy Fix

## Intent

Three Clippy errors in test code block the pre-commit hook (`cargo clippy --all-targets -- -D warnings`). Resolving them unblocks development and maintains code quality.

## Scope

### In Scope
- Remove `.to_string()` from 2 `&str` literals in `migrations/mod.rs`
- Fix `too_many_arguments` in `search.rs` test helper — prefer `ProductTestParams` struct
- Run `cargo clippy --all-targets -- -D warnings` to confirm zero errors
- Run `cargo test` to confirm all tests still pass

### Out of Scope
- mypy strict errors
- Any production code changes
- Refactoring beyond the clippy fixes

## Capabilities

### New Capabilities
None — pure lint fix, no behavior change.

### Modified Capabilities
None — no spec-level requirements change.

## Approach

**Error 1** (`to_string()`): Remove `.to_string()` from 2 string literals. `&str` implements `AsRef<Path>` so the function accepts it directly.

**Error 2** (`too_many_arguments`): Extract 9 test helper parameters into a `ProductTestParams` struct with builder pattern. Cleaner than suppressing the lint.

```
struct ProductTestParams<'a> {
    pool: &'a SqlitePool,
    sku: &'a str,
    name: &'a str,
    brand: &'a str,
    category: &'a str,
    price: f64,
    source_id: &'a str,
    condition: &'a str,
    currency: &'a str,
}
```

Verify with `cargo clippy --all-targets -- -D warnings && cargo test`.

## Affected Areas

| Area | Impact | Description |
|------|--------|-------------|
| `src-tauri/src/repository/sqlite/migrations/mod.rs` | Modified | Remove 2 `.to_string()` calls |
| `src-tauri/src/services/search.rs` | Modified | Extract `ProductTestParams`, update callers |

## Risks

| Risk | Likelihood | Mitigation |
|------|------------|------------|
| Struct change breaks test callers | Low | `cargo test` confirms |
| `AsRef<Path>` inference issue | Low | Standard impl; clippy confirms |

## Rollback Plan

Revert both files. Errors are in test-only code — no production impact.

## Dependencies

None.

## Success Criteria

- [ ] `cargo clippy --all-targets -- -D warnings` exits with code 0
- [ ] `cargo test` passes
- [ ] No production code was modified
