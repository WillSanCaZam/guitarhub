# Delta for Repo Hygiene

## ADDED Requirements

### Requirement: Test code MUST pass clippy with `-D warnings`

Test code MUST NOT trigger clippy errors when checked with `cargo clippy --all-targets -- -D warnings`. The following lint rules SHALL be enforced in test code:

- **`clippy::unnecessary_to_owned`** (`.to_string()` on `&str`): String literals passed to functions accepting `AsRef<Path>` or `&str` MUST NOT call `.to_string()`.
- **`clippy::too_many_arguments`**: Test helper functions with 9+ parameters MUST use a struct parameter instead.

#### Scenario: to_string on string literal suppressed

- GIVEN test code in `src-tauri/src/repository/sqlite/migrations/mod.rs` calls `run_migration` with `&str` literals
- WHEN the code contains `"CREATE TABLE t1 ...;".to_string()`
- THEN clippy MUST emit error `unnecessary_to_owned`
- AND the fix MUST remove `.to_string()` leaving the bare `&str` literal

#### Scenario: too_many_arguments suppressed

- GIVEN test helper `insert_product` in `src-tauri/src/services/search.rs` accepts 9 positional parameters
- WHEN clippy checks the file
- THEN clippy MUST emit error `too_many_arguments`
- AND the fix MUST extract parameters into a `ProductTestParams` struct

#### Scenario: Full clippy pass

- GIVEN all clippy fixes have been applied
- WHEN `cargo clippy --all-targets -- -D warnings` runs
- THEN it MUST exit with code 0

### Requirement: Refactored test helpers MUST preserve semantics

Structural changes to test helpers (extracting a struct, removing `.to_string()`) MUST NOT alter test behavior.

#### Scenario: All existing tests pass

- GIVEN the refactored `search.rs` and `migrations/mod.rs`
- WHEN `cargo test` runs
- THEN ALL tests MUST pass (exit code 0)

#### Scenario: ProductTestParams produces same SQL

- GIVEN a test creates `ProductTestParams { pool, sku: "X", name: "Y", brand: "Z", category: "C", price: 1.0, source_id: "S", condition: "New", currency: "USD" }`
- WHEN `insert_product` is called with the struct
- THEN the resulting SQL MUST be identical to the original 9-argument call with the same values
