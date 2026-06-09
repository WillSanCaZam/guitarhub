// SPDX-License-Identifier: GPL-3.0-or-later

use base64::Engine;
use tauri::State;
use url::Url;

use crate::repository::settings::SettingsRepository;
use crate::repository::sqlite::settings::SqliteSettingsRepository;
use crate::AppError;
use crate::AppState;

/// Tauri IPC command: fetch a product image via the local cache.
///
/// Security:
/// - Scheme MUST be `https`
/// - Host MUST be a known allowlisted domain or subdomain thereof
/// - IP literal hosts are rejected (prevents SSRF to internal networks)
/// - Allowed domains are read from settings at request time (no restart needed)
#[tauri::command]
pub async fn get_product_image(
    image_url: String,
    state: State<'_, AppState>,
) -> Result<String, AppError> {
    let repo = SqliteSettingsRepository::new(state.pool.clone());
    let domains = get_allowed_image_domains(&repo).await;
    let domain_refs: Vec<&str> = domains.iter().map(String::as_str).collect();
    let url = validate_image_url(&image_url, &domain_refs).map_err(AppError::InvalidInput)?;

    let (bytes, mime) = state
        .image_cache_service
        .get(url.as_str())
        .await
        .map_err(|e| AppError::Network(format!("Image load failed: {e}")))?;

    let b64 = base64::engine::general_purpose::STANDARD.encode(&bytes);
    Ok(format!("data:{mime};base64,{b64}"))
}

/// Read the allowed image domains from settings, falling back to the built-in
/// set if the setting is empty, unparseable, or missing.
pub async fn get_allowed_image_domains(repo: &dyn SettingsRepository) -> Vec<String> {
    let raw = repo.get("allowed_image_domains").await;
    match raw {
        Some(val) if !val.trim().is_empty() => {
            let parsed = parse_allowed_domains(&val);
            if parsed.is_empty() {
                vec!["reverb.com".to_string(), "mlstatic.com".to_string()]
            } else {
                parsed
            }
        }
        _ => vec!["reverb.com".to_string(), "mlstatic.com".to_string()],
    }
}

/// Parse a comma-separated list of domains, trimming whitespace and
/// filtering out empty entries.
///
/// Example: `" reverb.com , mlstatic.com "` → `["reverb.com", "mlstatic.com"]`
pub fn parse_allowed_domains(input: &str) -> Vec<String> {
    input
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

/// Validate that `raw` is a safe HTTPS URL to an allowlisted domain.
///
/// Returns `Err` describing the rejection reason on failure.
fn validate_image_url(raw: &str, allowed_domains: &[&str]) -> Result<Url, String> {
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
        return Err("Rejected IP literal host: SSRF protection".to_string());
    }

    // Host must be an allowlisted domain or subdomain thereof
    let host_str = host.to_string();
    let is_allowed = allowed_domains
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
    use crate::repository::settings::MockSettingsRepository;

    const TEST_DOMAINS: &[&str] = &["reverb.com", "mlstatic.com"];

    #[test]
    fn rejects_http_scheme() {
        let result = validate_image_url("http://reverb.com/pedal.jpg", TEST_DOMAINS);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("scheme 'http'"));
    }

    #[test]
    fn rejects_ip_literal_ipv4() {
        let result = validate_image_url("https://10.0.0.1/config", TEST_DOMAINS);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("IP literal"));
    }

    #[test]
    fn rejects_ip_literal_ipv6() {
        let result = validate_image_url("https://[::1]/config", TEST_DOMAINS);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("IP literal"));
    }

    #[test]
    fn rejects_non_allowlisted_domain() {
        let result = validate_image_url("https://evil.com/payload", TEST_DOMAINS);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not in allowlist"));
    }

    #[test]
    fn rejects_malformed_url() {
        let result = validate_image_url("not-a-url", TEST_DOMAINS);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid URL"));
    }

    #[test]
    fn accepts_valid_reverb_url() {
        let result = validate_image_url("https://images.reverb.com/pedal.jpg", TEST_DOMAINS);
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap().as_str(),
            "https://images.reverb.com/pedal.jpg"
        );
    }

    #[test]
    fn accepts_subdomain_of_allowed() {
        let result = validate_image_url("https://cdn.mlstatic.com/img.jpg", TEST_DOMAINS);
        assert!(result.is_ok());
    }

    #[test]
    fn accepts_exact_domain() {
        let result = validate_image_url("https://reverb.com/img.jpg", TEST_DOMAINS);
        assert!(result.is_ok());
    }

    // ── Error mapping tests (AppError boundary) ─────────────────────────

    #[test]
    fn validate_url_error_maps_to_invalid_input() {
        let err = validate_image_url("http://reverb.com/pedal.jpg", TEST_DOMAINS).unwrap_err();
        let app_err = crate::AppError::InvalidInput(err);
        assert!(matches!(app_err, crate::AppError::InvalidInput(_)));
    }

    #[test]
    fn validate_url_ip_literal_maps_to_invalid_input() {
        let err = validate_image_url("https://10.0.0.1/config", TEST_DOMAINS).unwrap_err();
        let app_err = crate::AppError::InvalidInput(err);
        assert!(matches!(app_err, crate::AppError::InvalidInput(s) if s.contains("IP literal")));
    }

    // ── parse_allowed_domains pure function tests ────────────────────────

    #[test]
    fn parse_domains_normal_list() {
        let result = parse_allowed_domains("reverb.com,mlstatic.com,newstore.com");
        assert_eq!(result, vec!["reverb.com", "mlstatic.com", "newstore.com"]);
    }

    #[test]
    fn parse_domains_trims_whitespace() {
        let result = parse_allowed_domains("  reverb.com ,  mlstatic.com  ");
        assert_eq!(result, vec!["reverb.com", "mlstatic.com"]);
    }

    #[test]
    fn parse_domains_empty_returns_empty() {
        let result = parse_allowed_domains("");
        assert!(result.is_empty());
    }

    #[test]
    fn parse_domains_only_whitespace_returns_empty() {
        let result = parse_allowed_domains("   ,  ,  ");
        assert!(result.is_empty());
    }

    // ── get_allowed_image_domains tests ──────────────────────────────────

    #[tokio::test]
    async fn get_domains_from_settings() {
        let mock = MockSettingsRepository::default();
        mock.save("allowed_image_domains", "reverb.com,newstore.com")
            .await
            .unwrap();
        let result = get_allowed_image_domains(&mock).await;
        assert_eq!(result, vec!["reverb.com", "newstore.com"]);
    }

    #[tokio::test]
    async fn get_domains_empty_setting_falls_back() {
        let mock = MockSettingsRepository::default();
        mock.save("allowed_image_domains", "").await.unwrap();
        let result = get_allowed_image_domains(&mock).await;
        assert_eq!(result, vec!["reverb.com", "mlstatic.com"]);
    }

    #[tokio::test]
    async fn get_domains_missing_setting_falls_back() {
        let mock = MockSettingsRepository::default();
        let result = get_allowed_image_domains(&mock).await;
        assert_eq!(result, vec!["reverb.com", "mlstatic.com"]);
    }

    #[tokio::test]
    async fn get_domains_malformed_returns_parsed_values() {
        let mock = MockSettingsRepository::default();
        mock.save("allowed_image_domains", "not,a,domain")
            .await
            .unwrap();
        // Malformed-but-parseable values are returned as-is;
        // domain validation will reject them on lookup.
        let result = get_allowed_image_domains(&mock).await;
        assert_eq!(result, vec!["not", "a", "domain"]);
    }
}
