# Design: WU2 — Security Hardening

## Technical Approach

Five independent security fixes for the Rust backend: SSRF protection at the IPC boundary, MIME type allowlist at the HTTP fetch layer, WAL journal mode for SQLite integrity, Rust ecosystem config files (audit threshold, toolchain pinning, vulnerability disclosure), and a pre-commit secret detection hook. Each fix is localized to a single file or new file, making them independently revertable.

## Architecture Decisions

### Decision: URL validation at the command boundary (not in the service)
- **Choice**: Validate in `image_command.rs` before calling `ImageCacheService::get()`
- **Alternatives**: Validate inside `ImageCacheService::get()` or at the HTTP fetch layer
- **Rationale**: The command is the IPC boundary — rejecting there prevents any processing. The service's `get()` takes `&str` and could be called from other paths; defense-in-depth means we validate at both layers, but the strictest check is at the outermost boundary. `image_command.rs` is the first Rust code that touches user-supplied data

### Decision: Domain allowlist, not blocklist
- **Choice**: Explicit allowlist of `reverb.com`, `mlstatic.com`, and subdomains
- **Alternatives**: Blocklist of known-bad domains; regex-based host matching
- **Rationale**: Allowlist wins for SSRF — you cannot enumerate every internal IP or bad domain. The allowlist is small (MVP has 2 CDN sources) and extensible via a `Vec` or `HashSet`

### Decision: MIME rejection in `http_get()`, not in `get()`
- **Choice**: Validate Content-Type inside `ImageCacheService::http_get()`
- **Alternatives**: Validate `Content-Type` in `get_inner()` before cache write
- **Rationale**: `http_get()` is the sole HTTP fetch method. Checking there means ANY path that fetches from the network enforces the allowlist. Also removes the `detect_mime_from_bytes()` fallback entirely — if the server doesn't send a supported Content-Type, we reject, no guesswork

### Decision: WAL PRAGMA in `initialize_database()` after pool creation, before migrations
- **Choice**: `PRAGMA journal_mode=WAL;` as a raw SQLx query right after `pool.connect()`, before `MigrationRunner::run()`
- **Alternatives**: In a migration file (v2), or in `MigrationRunner::run()` as a special step
- **Rationale**: WAL must be set BEFORE any write operations. A migration would set it too late (schema creation happens in v1). A raw PRAGMA before migrations guarantees WAL is active for all schema and data work. The `PRAGMA` returns the new journal mode, which we log for observability

## File Changes

### `src-tauri/src/commands/image_command.rs` — Modify (SSRF fix)

**Current state**: Passes `image_url` string directly to `state.get(&image_url)` — no validation at the command boundary.

```rust
#[tauri::command]
pub async fn get_product_image(
    image_url: String,
    state: State<'_, ImageCacheService>,
) -> Result<String, String> {
    let (bytes, mime) = state
        .get(&image_url)
        .await
        .map_err(|e| format!("Image load failed: {e}"))?;
    let b64 = base64::engine::general_purpose::STANDARD.encode(&bytes);
    Ok(format!("data:{mime};base64,{b64}"))
}
```

**Target state**: Parse + validate URL before passing to service.

```rust
use base64::Engine;
use tauri::State;
use url::Url;

use crate::services::image_cache::ImageCacheService;

/// The set of allowed domains for image URLs.
static ALLOWED_DOMAINS: &[&str] = &[
    "reverb.com",
    "mlstatic.com",
];

/// Tauri IPC command: fetch a product image via the local cache.
///
/// Security:
/// - Scheme MUST be `https`
/// - Host MUST be a known allowlisted domain or subdomain thereof
/// - IP literal hosts are rejected (prevents SSRF to internal networks)
#[tauri::command]
pub async fn get_product_image(
    image_url: String,
    state: State<'_, ImageCacheService>,
) -> Result<String, String> {
    let url = validate_image_url(&image_url)?;

    let (bytes, mime) = state
        .get(url.as_str())
        .await
        .map_err(|e| format!("Image load failed: {e}"))?;

    let b64 = base64::engine::general_purpose::STANDARD.encode(&bytes);
    Ok(format!("data:{mime};base64,{b64}"))
}

/// Validate that `raw` is a safe HTTPS URL to a allowlisted domain.
///
/// Returns `Err` describing the rejection reason on failure.
fn validate_image_url(raw: &str) -> Result<Url, String> {
    let url = Url::parse(raw).map_err(|_| format!("Invalid URL: {raw}"))?;

    // Scheme must be https only
    if url.scheme() != "https" {
        return Err(format!(
            "Rejected URL with scheme '{}': only https is allowed",
            url.scheme()
        ));
    }

    // Reject IP literals (IPv4 and IPv6) — prevents SSRF to internal IPs
    let host = url.host().ok_or_else(|| "URL has no host".to_string())?;
    if let url::Host::Ipv4(_) | url::Host::Ipv6(_) = host {
        return Err(format!(
            "Rejected IP literal host: SSRF protection"
        ));
    }

    // Host must be a allowlisted domain or subdomain thereof
    let host_str = host.to_string();
    let is_allowed = ALLOWED_DOMAINS
        .iter()
        .any(|domain| host_str == *domain || host_str.ends_with(&format!(".{domain}")));
    if !is_allowed {
        return Err(format!(
            "Rejected domain '{host_str}': not in allowlist"
        ));
    }

    Ok(url)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_http_scheme() {
        let result = validate_image_url("http://reverb.com/pedal.jpg");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("scheme 'http'"));
    }

    #[test]
    fn rejects_ip_literal_ipv4() {
        let result = validate_image_url("https://10.0.0.1/config");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("IP literal"));
    }

    #[test]
    fn rejects_ip_literal_ipv6() {
        let result = validate_image_url("https://[::1]/config");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("IP literal"));
    }

    #[test]
    fn rejects_non_allowlisted_domain() {
        let result = validate_image_url("https://evil.com/payload");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not in allowlist"));
    }

    #[test]
    fn rejects_malformed_url() {
        let result = validate_image_url("not-a-url");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid URL"));
    }

    #[test]
    fn accepts_valid_reverb_url() {
        let result = validate_image_url("https://images.reverb.com/pedal.jpg");
        assert!(result.is_ok());
        assert_eq!(result.unwrap().as_str(), "https://images.reverb.com/pedal.jpg");
    }

    #[test]
    fn accepts_subdomain_of_allowed() {
        let result = validate_image_url("https://cdn.mlstatic.com/img.jpg");
        assert!(result.is_ok());
    }

    #[test]
    fn accepts_exact_domain() {
        let result = validate_image_url("https://reverb.com/img.jpg");
        assert!(result.is_ok());
    }
}
```

### `src-tauri/src/services/image_cache.rs` — Modify (MIME allowlist)

**Current state**: `http_get()` accepts any Content-Type starting with `"image/"`, falls back to `detect_mime_from_bytes()` which defaults to `"image/jpeg"` for unknown content. This means arbitrary content (SVG with embedded JS, polyglot files) could be cached as "image/jpeg".

**Target state**: Replace the permissive MIME handling with an explicit allowlist. Remove `detect_mime_from_bytes()` entirely.

Changes in `http_get()`:
1. Replace the `content_type` extraction + fallback block with strict allowlist check
2. Remove `detect_mime_from_bytes()` function (lines 365-372)

New MIME validation block (replace lines 304-313):
```rust
        // Strict MIME allowlist — only known image types
        let mime_type = content_type.ok_or_else(|| {
            ImageCacheError::DownloadFailed("Missing Content-Type header".to_string())
        })?;

        match mime_type.as_str() {
            "image/jpeg" | "image/png" | "image/webp" | "image/avif" | "image/gif" => {}
            _ => {
                return Err(ImageCacheError::DownloadFailed(format!(
                    "Rejected Content-Type '{mime_type}': not in image allowlist"
                )));
            }
        }

        Ok((bytes, mime_type))
```

Also remove the `detect_mime_from_bytes()` function entirely (no callers remain).

### `src-tauri/src/lib.rs` — Modify (WAL mode)

**Current state**: `initialize_database()` creates pool, runs migrations, builds service. No WAL pragma.

Add after `pool.connect()` and before `runner.run()`:
```rust
    // Enable WAL mode before any schema operations
    // Must run BEFORE the first connection writes to the database.
    {
        let result: String = sqlx::query_scalar("PRAGMA journal_mode=WAL;")
            .fetch_one(&pool)
            .await
            .context("Failed to set WAL journal mode")?;
        tracing::info!("SQLite journal mode set to: {result}");
    }
```

**Exact insertion point** (lib.rs line 33, after `let pool = ... .await?;`):
```rust
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect(db_path)
        .await?;

    // ▶ INSERT WAL PRAGMA HERE ◀

    let migrations_dir = std::env::var("GUITARHUB_MIGRATIONS_DIR")
        // ...
```

### `.cargo-audit.toml` — Create

```toml
[severity]
threshold = "high"
```

### `rust-toolchain.toml` — Create

```toml
[toolchain]
channel = "stable"
components = ["rustfmt", "clippy"]
```

### `SECURITY.md` — Create

```markdown
# Security Policy

## Reporting a Vulnerability

Please report security vulnerabilities to the GitHub private vulnerability reporting tool:

https://github.com/{owner}/GuitarHub/security/advisories

## Response SLA

We acknowledge reports within 72 hours and provide an initial assessment within 5 business days.
```

### `.pre-commit-config.yaml` — Modify

Add gitleaks hook after the existing local hooks (after line 47). This places it at the end of the hook sequence — secrets are checked after formatting and linting, which is the conventional order.

```yaml
  # ── Secret detection ─────────────────────────────────────────────────────
  - repo: https://github.com/gitleaks/gitleaks
    rev: v8.18.2
    hooks:
      - id: gitleaks
```

## Sequence

1. `lib.rs` — Add WAL PRAGMA (no-op change for existing DBs, upgrades new ones immediately)
2. `image_command.rs` — Add SSRF validation (new function, no side effects until actually invoked)
3. `image_cache.rs` — Add MIME allowlist, remove `detect_mime_from_bytes` (changes HTTP fetch behavior)
4. Create `.cargo-audit.toml` — changes `cargo audit` behavior
5. Create `rust-toolchain.toml` — changes toolchain resolution
6. Create `SECURITY.md` — docs only
7. `.pre-commit-config.yaml` — Add gitleaks hook

Items 1-3 are Rust code changes (need `cargo build`). Items 4-7 are config/docs changes.

## Risks

| Risk | Likelihood | Mitigation |
|------|-----------|------------|
| Domain allowlist blocks valid CDN | Low | MVP has 2 known CDNs; allowlist is a `static` array, trivial to extend |
| MIME allowlist too strict | Low | Covers all common web image formats (jpeg, png, webp, avif, gif); SVG is intentionally excluded (XSS vector) |
| WAL mode breaks on network FS | Low | MVP targets local SQLite only; WAL is the standard recommendation for sqlx+SQLite |
| gitleaks false positives slow commits | Low | Pre-commit hooks can be skipped with `git commit --no-verify`; tune via `.gitleaks.toml` if needed |

## Testing Approach

| Layer | What | How |
|-------|------|-----|
| Unit | SSRF validation (6 tests) | `cargo test` — tests in `image_command.rs` exercise all rejection cases + valid cases |
| Unit | MIME allowlist | Existing tests in `image_cache.rs` already use `Content-Type: image/png` and `image/jpeg` — they continue to pass. Add a test with `Content-Type: text/html` to confirm rejection |
| Unit | WAL mode | `cargo test` — verify `PRAGMA journal_mode` in test pool returns `"wal"` |
| Audit | `cargo audit` | Must run with `threshold = "high"` — low/medium advisories exit 0 |
| Toolchain | `rustc --version` | Must match stable channel per `rust-toolchain.toml` |
| Pre-commit | `pre-commit run --all-files` | Must detect test secrets (add `gitleaks` to CI step) |
| Integration | SSRF blocked | `curl` a local test that sends IP literal → confirm `Err` returned, no HTTP call made |
