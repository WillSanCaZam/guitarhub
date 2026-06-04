use serde::{Deserialize, Serialize};
use url::Url;

/// Alert channel configuration read from settings.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type")]
pub enum AlertChannel {
    App,
    Ntfy { topic: String },
    Webhook { url: String },
}

/// Result from a test send.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct AlertTestResult {
    pub success: bool,
    pub message: String,
}

/// Validate that a URL is suitable for webhook usage.
///
/// Rules:
/// - Must be parseable by the `url` crate
/// - Scheme must be `http` or `https`
/// - Host must not be an IP literal (SSRF prevention)
pub fn validate_webhook_url(raw: &str) -> Result<Url, String> {
    let url = Url::parse(raw).map_err(|_| "invalid_url".to_string())?;
    match url.scheme() {
        "http" | "https" => {}
        _ => return Err("invalid_url".to_string()),
    }
    // Reject IP literals (SSRF prevention)
    if matches!(url.host(), Some(url::Host::Ipv4(_)) | Some(url::Host::Ipv6(_))) {
        return Err("invalid_url".to_string());
    }
    Ok(url)
}

/// Validate a Ntfy topic: must be non-empty, alphanumeric + hyphens.
pub fn validate_ntfy_topic(topic: &str) -> Result<(), String> {
    if topic.is_empty() {
        return Err("invalid_topic".to_string());
    }
    if !topic
        .chars()
        .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
    {
        return Err("invalid_topic".to_string());
    }
    Ok(())
}

// ── AlertDispatcher trait ──────────────────────────────────────────────────

/// Trait for alert delivery mechanisms.
///
/// Each implementation delivers alerts through a different channel.
/// MUST NOT panic. MUST log errors internally.
///
/// The `client` parameter is the HTTP client used for outbound requests;
/// pass it at the call site rather than storing it in the implementation.
#[async_trait::async_trait]
pub trait AlertDispatcher: Send + Sync {
    /// Send an alert with the given title and message body.
    async fn send(
        &self,
        title: &str,
        message: &str,
        client: &reqwest::Client,
    ) -> Result<(), String>;

    /// Send a test notification to verify channel configuration.
    async fn test(&self, client: &reqwest::Client) -> AlertTestResult;
}

// ── AppNotificationAlert ───────────────────────────────────────────────────

/// In-app notification dispatcher (service-layer stub).
///
/// This logs the notification via `tracing::info!` and does NOT use the Tauri
/// notification API because the service layer has no access to `AppHandle`.
///
/// Production callers (commands) MUST use `tauri_plugin_notification` directly
/// with the `AppHandle` instead of this service-layer implementation.  This
/// struct exists to satisfy the `AlertDispatcher` trait contract in unit tests
/// and as a fallback when no Tauri runtime is available.
#[derive(Debug, Clone)]
pub struct AppNotificationAlert;

#[async_trait::async_trait]
impl AlertDispatcher for AppNotificationAlert {
    async fn send(
        &self,
        title: &str,
        message: &str,
        _client: &reqwest::Client,
    ) -> Result<(), String> {
        tracing::info!("App notification (stub): {title} — {message}");
        Ok(())
    }

    async fn test(&self, _client: &reqwest::Client) -> AlertTestResult {
        AlertTestResult {
            success: true,
            message: "Notification would be sent (app channel)".to_string(),
        }
    }
}

// ── NtfyAlert ──────────────────────────────────────────────────────────────

/// Ntfy.sh push notification dispatcher.
///
/// POSTs a JSON body to `{base_url}/{topic}` with title, message, and tags.
/// The `base_url` defaults to `https://ntfy.sh` but can be overridden
/// for testing with a local mock server.
///
/// Does NOT own an HTTP client — one must be passed to `send` / `test`.
#[derive(Debug, Clone)]
pub struct NtfyAlert {
    topic: String,
    /// Base URL of the ntfy server (default: `https://ntfy.sh`).
    /// Made `pub(crate)` to allow test injection of a mock server URL.
    pub(crate) base_url: String,
}

impl NtfyAlert {
    pub fn new(topic: String) -> Result<Self, String> {
        validate_ntfy_topic(&topic)?;
        Ok(Self {
            topic,
            base_url: "https://ntfy.sh".to_string(),
        })
    }

    /// URL used for the HTTP POST request.
    fn post_url(&self) -> String {
        format!("{}/{}", self.base_url.trim_end_matches('/'), self.topic)
    }

    /// Core HTTP POST logic, extracted for retry support.
    pub(crate) async fn send_inner(
        &self,
        title: &str,
        message: &str,
        client: &reqwest::Client,
    ) -> Result<(), String> {
        let url = self.post_url();
        let body = serde_json::json!({
            "topic": self.topic,
            "title": title,
            "message": message,
            "tags": ["moneybag"],
        });
        let response = client
            .post(&url)
            .json(&body)
            .timeout(std::time::Duration::from_secs(5))
            .send()
            .await
            .map_err(|e| format!("ntfy_error: {e}"))?;
        if !response.status().is_success() {
            return Err(format!("ntfy_error: HTTP {}", response.status().as_u16()));
        }
        Ok(())
    }
}

#[async_trait::async_trait]
impl AlertDispatcher for NtfyAlert {
    async fn send(
        &self,
        title: &str,
        message: &str,
        client: &reqwest::Client,
    ) -> Result<(), String> {
        let first = self.send_inner(title, message, client).await;
        if first.is_ok() {
            return first;
        }
        tracing::warn!("Ntfy send failed, retrying once after 3s: {:?}", first);
        tokio::time::sleep(std::time::Duration::from_secs(3)).await;
        self.send_inner(title, message, client).await
    }

    async fn test(&self, client: &reqwest::Client) -> AlertTestResult {
        match self
            .send(
                "Test Notification",
                "This is a test alert from GuitarHub",
                client,
            )
            .await
        {
            Ok(()) => AlertTestResult {
                success: true,
                message: "Test notification sent to Ntfy.sh".to_string(),
            },
            Err(e) => AlertTestResult {
                success: false,
                message: e,
            },
        }
    }
}

// ── WebhookAlert ───────────────────────────────────────────────────────────

/// Generic webhook POST dispatcher.
///
/// Sends a JSON payload to a user-configured URL with alert details.
///
/// Does NOT own an HTTP client — one must be passed to `send` / `test`.
#[derive(Debug, Clone)]
pub struct WebhookAlert {
    pub(crate) url: Url,
}

impl WebhookAlert {
    pub fn new(raw_url: String) -> Result<Self, String> {
        let url = validate_webhook_url(&raw_url)?;
        Ok(Self { url })
    }

    /// Core HTTP POST logic, extracted for retry support.
    pub(crate) async fn send_inner(
        &self,
        title: &str,
        message: &str,
        client: &reqwest::Client,
    ) -> Result<(), String> {
        let body = serde_json::json!({
            "title": title,
            "message": message,
        });
        let response = client
            .post(self.url.as_str())
            .json(&body)
            .timeout(std::time::Duration::from_secs(5))
            .send()
            .await
            .map_err(|e| format!("webhook_error: {e}"))?;
        if !response.status().is_success() {
            return Err(format!("webhook_error: HTTP {}", response.status().as_u16()));
        }
        Ok(())
    }
}

#[async_trait::async_trait]
impl AlertDispatcher for WebhookAlert {
    async fn send(
        &self,
        title: &str,
        message: &str,
        client: &reqwest::Client,
    ) -> Result<(), String> {
        let first = self.send_inner(title, message, client).await;
        if first.is_ok() {
            return first;
        }
        tracing::warn!("Webhook send failed, retrying once after 3s: {:?}", first);
        tokio::time::sleep(std::time::Duration::from_secs(3)).await;
        self.send_inner(title, message, client).await
    }

    async fn test(&self, client: &reqwest::Client) -> AlertTestResult {
        match self
            .send(
                "Test Notification",
                "This is a test alert from GuitarHub",
                client,
            )
            .await
        {
            Ok(()) => AlertTestResult {
                success: true,
                message: "Test notification sent to webhook".to_string(),
            },
            Err(e) => AlertTestResult {
                success: false,
                message: e,
            },
        }
    }
}

// ── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use httpmock::prelude::*;

    // ── validate_webhook_url ─────────────────────────────────────────────

    #[test]
    fn validate_webhook_url_rejects_empty() {
        let result = validate_webhook_url("");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "invalid_url");
    }

    #[test]
    fn validate_webhook_url_rejects_non_http() {
        let result = validate_webhook_url("ftp://example.com/hook");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "invalid_url");
    }

    #[test]
    fn validate_webhook_url_rejects_ip_literal() {
        let result = validate_webhook_url("http://10.0.0.1/hook");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "invalid_url");
    }

    #[test]
    fn validate_webhook_url_accepts_valid_https() {
        let result = validate_webhook_url("https://hooks.example.com/alert");
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap().as_str(),
            "https://hooks.example.com/alert"
        );
    }

    #[test]
    fn validate_webhook_url_accepts_http_localhost() {
        let result = validate_webhook_url("http://localhost:8080/hook");
        assert!(result.is_ok());
    }

    // ── validate_ntfy_topic ─────────────────────────────────────────────

    #[test]
    fn validate_ntfy_topic_rejects_empty() {
        assert!(validate_ntfy_topic("").is_err());
    }

    #[test]
    fn validate_ntfy_topic_accepts_valid() {
        assert!(validate_ntfy_topic("guitar-deals").is_ok());
        assert!(validate_ntfy_topic("guitar_deals_123").is_ok());
    }

    #[test]
    fn validate_ntfy_topic_rejects_special_chars() {
        assert!(validate_ntfy_topic("guitar deals!").is_err());
        assert!(validate_ntfy_topic("topic/path").is_err());
    }

    // ── AppNotificationAlert ────────────────────────────────────────────

    #[tokio::test]
    async fn app_notification_alert_send_returns_ok() {
        let alert = AppNotificationAlert;
        let client = reqwest::Client::new();
        let result = alert.send("Title", "Message", &client).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn app_notification_alert_test_returns_success() {
        let alert = AppNotificationAlert;
        let client = reqwest::Client::new();
        let result = alert.test(&client).await;
        assert!(result.success);
        assert!(!result.message.is_empty());
    }

    // ── NtfyAlert ───────────────────────────────────────────────────────

    #[tokio::test]
    async fn ntfy_alert_sends_post_to_correct_url() {
        let server = MockServer::start();
        let mock = server.mock(|when, then| {
            when.method(POST).path("/guitar-deals");
            then.status(200).json_body(serde_json::json!({"ok": true}));
        });

        let client = reqwest::Client::new();
        let alert = NtfyAlert {
            topic: "guitar-deals".to_string(),
            base_url: server.base_url(),
        };

        let result = alert.send("Price Drop", "Item is now cheaper!", &client).await;
        assert!(result.is_ok());
        mock.assert_hits(1);
    }

    #[tokio::test]
    async fn ntfy_alert_test_returns_success_on_ok() {
        let server = MockServer::start();
        let mock = server.mock(|when, then| {
            when.method(POST).path("/test-topic");
            then.status(200).json_body(serde_json::json!({"ok": true}));
        });

        let client = reqwest::Client::new();
        let alert = NtfyAlert {
            topic: "test-topic".to_string(),
            base_url: server.base_url(),
        };

        let result = alert.test(&client).await;
        assert!(result.success);
        assert!(result.message.contains("sent"));
        mock.assert_hits(1);
    }

    #[tokio::test]
    async fn ntfy_alert_handles_http_500() {
        let server = MockServer::start();
        let mock = server.mock(|when, then| {
            when.method(POST).path("/fail-topic");
            then.status(500);
        });

        let client = reqwest::Client::new();
        let alert = NtfyAlert {
            topic: "fail-topic".to_string(),
            base_url: server.base_url(),
        };

        let result = alert.test(&client).await;
        assert!(!result.success);
        assert!(result.message.contains("HTTP 500"));
        // Retry-once sends two requests on failure.
        mock.assert_hits(2);
    }

    #[tokio::test]
    async fn ntfy_alert_handles_network_error() {
        // Point to a server that doesn't exist on the wire.
        // Use a localhost address that nothing is listening on.
        let client = reqwest::Client::new();
        let alert = NtfyAlert {
            topic: "test".to_string(),
            base_url: "http://127.0.0.1:1".to_string(), // port 1 is unlikely to be listening
        };

        let result = alert.send("Title", "Message", &client).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("ntfy_error"));
    }

    // ── WebhookAlert ────────────────────────────────────────────────────

    #[tokio::test]
    async fn webhook_alert_sends_valid_json() {
        let server = MockServer::start();
        let mock = server.mock(|when, then| {
            when.method(POST)
                .path("/webhook")
                .header("Content-Type", "application/json");
            then.status(200).json_body(serde_json::json!({"ok": true}));
        });

        let client = reqwest::Client::new();
        let url = format!("{}/webhook", server.base_url());
        let alert = WebhookAlert {
            url: Url::parse(&url).unwrap(),
        };

        let result = alert.send("Alert!", "Something happened", &client).await;
        assert!(result.is_ok());
        mock.assert_hits(1);
    }

    #[tokio::test]
    async fn webhook_alert_handles_http_400() {
        let server = MockServer::start();
        let mock = server.mock(|when, then| {
            when.method(POST).path("/bad-request");
            then.status(400);
        });

        let client = reqwest::Client::new();
        let url = format!("{}/bad-request", server.base_url());
        let alert = WebhookAlert {
            url: Url::parse(&url).unwrap(),
        };

        let result = alert.test(&client).await;
        assert!(!result.success);
        assert!(result.message.contains("HTTP 400"));
        // Retry-once sends two requests on failure.
        mock.assert_hits(2);
    }

    #[tokio::test]
    async fn webhook_alert_handles_network_failure() {
        let client = reqwest::Client::new();
        let alert = WebhookAlert {
            url: Url::parse("http://127.0.0.1:1/webhook").unwrap(),
        };

        let result = alert.send("Title", "Message", &client).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("webhook_error"));
    }

    #[tokio::test]
    async fn webhook_alert_construction_validates_url() {
        let result = WebhookAlert::new(
            "https://hooks.example.com/alert".to_string(),
        );
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn webhook_alert_rejects_bad_url() {
        let result = WebhookAlert::new(
            "not-a-url".to_string(),
        );
        assert!(result.is_err());
    }

    // ── AlertChannel serialization ──────────────────────────────────────

    #[test]
    fn alert_channel_app_serializes_correctly() {
        let json = serde_json::to_string(&AlertChannel::App).unwrap();
        assert_eq!(json, r#"{"type":"App"}"#);
    }

    #[test]
    fn alert_channel_ntfy_serializes_correctly() {
        let json = serde_json::to_string(&AlertChannel::Ntfy {
            topic: "guitar".to_string(),
        })
        .unwrap();
        assert_eq!(json, r#"{"type":"Ntfy","topic":"guitar"}"#);
    }

    #[test]
    fn alert_channel_webhook_serializes_correctly() {
        let json =
            serde_json::to_string(&AlertChannel::Webhook {
                url: "https://hooks.example.com".to_string(),
            })
            .unwrap();
        assert_eq!(
            json,
            r#"{"type":"Webhook","url":"https://hooks.example.com"}"#
        );
    }

    #[test]
    fn alert_channel_deserializes_app() {
        let result: AlertChannel =
            serde_json::from_str(r#"{"type":"App"}"#).unwrap();
        assert_eq!(result, AlertChannel::App);
    }

    #[test]
    fn alert_channel_deserializes_ntfy() {
        let result: AlertChannel = serde_json::from_str(
            r#"{"type":"Ntfy","topic":"guitar-deals"}"#,
        )
        .unwrap();
        assert_eq!(
            result,
            AlertChannel::Ntfy {
                topic: "guitar-deals".to_string()
            }
        );
    }

    #[test]
    fn alert_channel_deserializes_webhook() {
        let result: AlertChannel = serde_json::from_str(
            r#"{"type":"Webhook","url":"https://hooks.example.com"}"#,
        )
        .unwrap();
        assert_eq!(
            result,
            AlertChannel::Webhook {
                url: "https://hooks.example.com".to_string()
            }
        );
    }
}
