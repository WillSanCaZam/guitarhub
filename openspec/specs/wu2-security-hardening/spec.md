# Security Hardening Specification

## Purpose

Fix SSRF, MIME smuggling, audit gaps, and secret leakage. Establish WAL integrity, toolchain pins, and disclosure policy.

## Requirements

### Requirement: SSRF protection on image URL fetch

`src-tauri/src/commands/image_command.rs` MUST parse `image_url` with `url::Url::parse()` at the IPC boundary. MUST reject non-`https` schemes, IP literal hosts, and domains not in the configured allowlist. The allowed domains MUST be read from `get_setting("allowed_image_domains")` at validation time. If the setting is empty, unparseable, or missing, the system MUST fall back to `["reverb.com", "mlstatic.com"]`.
(Previously: domains were checked against a hardcoded `static ALLOWED_DOMAINS`; now read from settings with fallback)

#### Updated Scenarios

| Case | Precondition | Action | Outcome |
|------|-------------|--------|---------|
| Domain in settings allowlist | Setting = "reverb.com,mlstatic.com,newstore.com" | `url = "https://newstore.com/pedal.jpg"` | Passes, download proceeds |
| Domain in fallback only | Setting empty | `url = "https://reverb.com/pedal.jpg"` | Passes (fallback allows) |
| Domain rejected | Setting = "reverb.com" | `url = "https://evil.com/payload"` | REJECTED |
| IP literal | Setting = "reverb.com" | `url = "https://10.0.0.1/config"` | REJECTED |
| HTTP scheme | Any setting | `url = "http://internal.dev"` | REJECTED |
| Malformed setting | Setting = "not,a,domain" | Parse, validate | Fallback to static, proceed per fallback rules |

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

### Requirement: Settings UI MUST expose domain allowlist

`Settings.svelte` MUST provide a text input field for the `allowed_image_domains` setting, accepting a comma-separated list of domain names. On save, the value MUST be persisted via the IPC `save_setting` command. The field SHOULD display the current value with a placeholder hint.

#### Scenario: User updates allowlist
- GIVEN the Settings view is open
- WHEN the user enters "reverb.com,mlstatic.com,newstore.com" in the domain field
- AND clicks Save
- THEN `get_setting("allowed_image_domains")` returns the saved value
- AND images from `newstore.com` are now allowed

#### Scenario: Empty allowlist accepted
- GIVEN the Settings view is open
- WHEN the user clears the domain field
- AND clicks Save
- THEN the system falls back to `["reverb.com", "mlstatic.com"]` on next validation

## Acceptance Criteria

| Criterion | How to verify |
|-----------|---------------|
| SSRF blocked for IP literals | Unit test: IP literal URL returns `Err`, not fetched |
| MIME blocked for SVG | Integration: SVG URL returns error, not stored in cache |
| WAL journal active | `PRAGMA journal_mode` returns `"wal"` |
| Audit only warns on high+ | `cargo audit` exits 0 with only low/medium advisories |
| Toolchain pinned | `rustc --version` matches `rust-toolchain.toml` channel |
| `gitleaks` blocks secrets | Insert test secret → `pre-commit run --all-files` fails |
