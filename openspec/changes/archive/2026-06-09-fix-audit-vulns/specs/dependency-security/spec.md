# Dependency Security Specification

## Purpose

Eliminate actionable `cargo audit` vulnerabilities by upgrading sqlx to a patched version with minimal feature surface, and suppress known-safe GTK3 advisories in audit output.

## Requirements

### Requirement: sqlx dependency minimum version

`src-tauri/Cargo.toml` MUST declare `sqlx` at version `"0.8"` with `default-features = false` and explicit features `["runtime-tokio", "sqlite", "derive"]`. No transitive dependency SHALL bring in the `any` or `rsa` crates.

#### Scenario: sqlx declared at 0.8 without default features

- GIVEN `src-tauri/Cargo.toml`
- WHEN checked for the sqlx dependency
- THEN `sqlx.version` SHALL be `"0.8"` or a 0.8.x patch
- AND `sqlx.default-features` SHALL be `false`
- AND `sqlx.features` SHALL include `runtime-tokio`, `sqlite`, and `derive`

#### Scenario: rsa crate absent from dependency tree

- GIVEN the resolved lockfile after `cargo update -p sqlx`
- WHEN `cargo tree -i rsa` is run
- THEN rsa MUST NOT appear in the output

### Requirement: Audit advisory ignore list

The project MUST provide an audit config at `.cargo/audit.toml` that ignores advisories RUSTSEC-2024-0412 through RUSTSEC-2024-0418 (GTK3-related, informational, no fix available).

#### Scenario: Advisory ignore file present

- GIVEN the project root
- WHEN `.cargo/audit.toml` is read
- THEN it SHALL contain an `[advisories.ignore]` section listing all 7 GTK3 advisory IDs

#### Scenario: Ignored advisories excluded from audit output

- GIVEN `.cargo/audit.toml` with the ignore list
- WHEN `cargo audit` runs
- THEN GTK3 advisories SHALL appear as "ignored" rather than "warnings" or "vulnerabilities"

### Requirement: Compilation and test integrity

The project MUST compile and pass all tests after the sqlx upgrade. Any sqlx 0.8 API breaks (e.g., `FromRow` trait changes, `query_as` return types) SHALL be fixed.

#### Scenario: Full build succeeds

- GIVEN the updated `Cargo.toml` and any code fixes
- WHEN `cargo build` runs from `src-tauri/`
- THEN it SHALL exit 0 with no errors

#### Scenario: Test suite passes

- GIVEN the updated dependency tree
- WHEN `cargo test` runs from `src-tauri/`
- THEN all tests SHALL pass

### Requirement: Audit gate passes clean

The two actionable vulnerabilities (RUSTSEC-2023-0071 via `rsa`, RUSTSEC-2024-0363 in sqlx 0.7) MUST NOT appear in audit output after the change.

#### Scenario: Actionable vulns eliminated

- GIVEN the updated project with `.cargo/audit.toml`
- WHEN `cargo audit` runs
- THEN its exit code SHALL be 0
- AND RUSTSEC-2023-0071 SHALL NOT appear
- AND RUSTSEC-2024-0363 SHALL NOT appear
