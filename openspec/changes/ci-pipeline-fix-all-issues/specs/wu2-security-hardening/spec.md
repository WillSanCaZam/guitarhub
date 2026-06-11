# Delta for wu2-security-hardening

> **Change**: ci-pipeline-fix-all-issues — Unsilence audit advisories, pin Rust toolchain

## MODIFIED Requirements

### Requirement: Dependency security audit config

`.cargo-audit.toml` MUST set `[severity]` threshold = `"high"`. Only `high`/`critical` advisories SHALL cause failures. The `.cargo/audit.toml` file MUST NOT contain `[advisories] ignore` entries that silence specific RUSTSEC advisories. All previously silenced advisories MUST be reviewed and either fixed or explicitly justified.

(Previously: `.cargo/audit.toml` ignored 8 RUSTSEC advisories, masking real vulnerabilities.)

#### Scenario: Audit threshold enforced

- GIVEN `.cargo-audit.toml` with `threshold = "high"`
- WHEN `cargo audit` runs
- THEN only `high`/`critical` advisories are reported
- AND no advisories are silenced via `ignore` entries

#### Scenario: Previously silenced advisory now visible

- GIVEN a previously ignored RUSTSEC advisory affects the project
- WHEN `cargo audit` runs
- THEN the advisory is reported
- AND the build fails if severity is `high` or `critical`

### Requirement: Rust toolchain pinning

`rust-toolchain.toml` MUST pin a specific stable channel version (not just `"stable"`) with `components = ["rustfmt", "clippy"]`. The pinned version MUST match the toolchain used in CI builds.

(Previously: `rust-toolchain.toml` pinned `channel = "stable"` without a specific version, causing non-reproducible builds across environments.)

#### Scenarios

| Case | Precondition | Action | Outcome |
|------|-------------|--------|---------|
| Toolchain honored | `rust-toolchain.toml` with pinned version exists | `cargo build` | Uses exact pinned version |
| Version mismatch caught | Local toolchain differs from pin | `cargo build` | rustup prompts to install pinned version |
| Components ready | `rustfmt`, `clippy` listed | `cargo clippy` | Tools available without manual install |
| CI uses pinned version | CI runner has different default | Workflow runs | `rustup` installs pinned version automatically |
