// SPDX-License-Identifier: GPL-3.0-or-later

use base64::Engine;
use tauri::State;
use url::Url;

use crate::AppError;
use crate::AppState;

/// The set of allowed domains for image URLs.
static ALLOWED_DOMAINS: &[&str] = &["reverb.com", "mlstatic.com"];

/// Tauri IPC command: fetch a product image via the local cache.
///
/// Security:
/// - Scheme MUST be `https`
/// - Host MUST be a known allowlisted domain or subdomain thereof
/// - IP literal hosts are rejected (prevents SSRF to internal networks)
#[tauri::command]
pub async fn get_product_image(
    image_url: String,
    state: State<'_, AppState>,
) -> Result<String, AppError> {
    let url = validate_image_url(&image_url).map_err(AppError::InvalidInput)?;

    let (bytes, mime) = state
        .image_cache_service
        .get(url.as_str())
        .await
        .map_err(|e| AppError::Network(format!("Image load failed: {e}")))?;

    let b64 = base64::engine::general_purpose::STANDARD.encode(&bytes);
    Ok(format!("data:{mime};base64,{b64}"))
}

/// Validate that `raw` is a safe HTTPS URL to an allowlisted domain.
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
        return Err("Rejected IP literal host: SSRF protection".to_string());
    }

    // Host must be an allowlisted domain or subdomain thereof
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
        assert_eq!(
            result.unwrap().as_str(),
            "https://images.reverb.com/pedal.jpg"
        );
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

    // ── Error mapping tests (AppError boundary) ─────────────────────────

    #[test]
    fn validate_url_error_maps_to_invalid_input() {
        let err = validate_image_url("http://reverb.com/pedal.jpg").unwrap_err();
        let app_err = crate::AppError::InvalidInput(err);
        assert!(matches!(app_err, crate::AppError::InvalidInput(_)));
    }

    #[test]
    fn validate_url_ip_literal_maps_to_invalid_input() {
        let err = validate_image_url("https://10.0.0.1/config").unwrap_err();
        let app_err = crate::AppError::InvalidInput(err);
        assert!(matches!(app_err, crate::AppError::InvalidInput(s) if s.contains("IP literal")));
    }
}
