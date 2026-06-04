# Security Hardening Specification

## Purpose

Fix SSRF, MIME smuggling, audit gaps, and secret leakage. Establish WAL integrity, toolchain pins, and disclosure policy.

## Requirements

### Requirement: SSRF protection on image URL fetch

`src-tauri/src/commands/image_command.rs` MUST parse `image_url` with `url::Url::parse()` at the IPC boundary. MUST reject non-`https` schemes, non-allowlisted domains, and IP literal hosts.

#### Scenarios

| Case | Precondition | Action | Outcome |
|------|-------------|--------|---------|
| Valid URL | `url = "https://images.reverb.com/pedal.jpg"` | Parse, validate | Passes, download proceeds |
| HTTP scheme | `url = "http://internal.dev/secret"` | Parse | REJECTED |
| IP literal | `url = "https://10.0.0.1/config"` | Parse, detect IP host | REJECTED |
| Non-allowlisted | `url = "https://evil.com/payload"` | Parse, check domain | REJECTED |
| Malformed | `url = "not-a-url"` | Parse fails | REJECTED |

### Requirement: MIME type allowlist on HTTP download

`ImageCacheService::http_get()` MUST validate `Content-Type` against `["image/jpeg", "image/png", "image/webp", "image/avif", "image/gif"]`. MUST NOT fall back to `detect_mime_from_bytes()`. (Affects: `specs/local-image-cache/spec.md` — Cache images on first display)

#### Scenarios

| Case | Precondition | Action | Outcome |
|------|-------------|--------|---------|
| Valid image | `Content-Type: image/webp` | Fetch | Blob cached, success returned |
| Unsupported | `Content-Type: image/svg+xml` | Fetch | REJECTED, blob NOT stored |
| Missing header | No `Content-Type` | Fetch | REJECTED |
| MIME mismatch | `Content-Type: text/html`, body is HTML | Fetch | REJECTED before cache write |

### Requirement: WAL journal mode for SQLite

`initialize_database()` MUST execute `PRAGMA journal_mode=WAL;` after pool creation and before any schema operations.

#### Scenarios

| Case | Precondition | Action | Outcome |
|------|-------------|--------|---------|
| Fresh DB | No existing DB file | `initialize_database()` runs | `journal_mode` returns `"wal"` |
| Existing DB | DB in DELETE mode | `initialize_database()` runs | Upgraded to WAL, no data loss |
| Startup error | Pool creation fails | PRAGMA runs | Error propagates, app fails clean |

### Requirement: Dependency security audit config

`.cargo-audit.toml` MUST set `[severity]` threshold = `"high"`. Only `high`/`critical` advisories SHALL cause failures.

#### Scenario: Audit threshold enforced

- GIVEN `.cargo-audit.toml` with `threshold = "high"`
- WHEN `cargo audit` runs
- THEN only `high`/`critical` advisories are reported

### Requirement: Rust toolchain pinning

`rust-toolchain.toml` MUST pin `channel = "stable"` with `components = ["rustfmt", "clippy"]`.

#### Scenarios

| Case | Precondition | Action | Outcome |
|------|-------------|--------|---------|
| Toolchain honored | `rust-toolchain.toml` exists | `cargo build` | Uses stable per project config |
| Override ignored | Default toolchain is nightly | Any cargo command | Project still uses stable |
| Components ready | `rustfmt`, `clippy` listed | `cargo clippy` | Tools available without manual install |

### Requirement: Vulnerability disclosure policy

`SECURITY.md` MUST contain a GitHub private vulnerability reporting link and a response SLA.

#### Scenario: Researcher finds vulnerability

- GIVEN a researcher discovers a vulnerability
- WHEN they open `SECURITY.md`
- THEN they find the reporting link and expected response time

### Requirement: Pre-commit secret detection

`.pre-commit-config.yaml` MUST include a `gitleaks` hook blocking commits containing secrets.

#### Scenario: Secret detected

- GIVEN a staged file contains an AWS access key
- WHEN `git commit` is attempted
- THEN `gitleaks` blocks the commit with a finding report

## Acceptance Criteria

| Criterion | How to verify |
|-----------|---------------|
| SSRF blocked for IP literals | Unit test: IP literal URL returns `Err`, not fetched |
| MIME blocked for SVG | Integration: SVG URL returns error, not stored in cache |
| WAL journal active | `PRAGMA journal_mode` returns `"wal"` |
| Audit only warns on high+ | `cargo audit` exits 0 with only low/medium advisories |
| Toolchain pinned | `rustc --version` matches `rust-toolchain.toml` channel |
| `gitleaks` blocks secrets | Insert test secret → `pre-commit run --all-files` fails |
