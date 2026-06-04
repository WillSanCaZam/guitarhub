use tauri::{AppHandle, State};
use tauri_plugin_notification::NotificationExt;

use crate::repository::settings::SettingsRepository;
use crate::repository::sqlite::settings::SqliteSettingsRepository;
use crate::services::alert_service::{AlertDispatcher, AlertTestResult, NtfyAlert, WebhookAlert};
use crate::AppError;
use crate::AppState;

/// Retrieve a setting by key.
///
/// Returns an empty string for unknown keys (not an error).
#[tauri::command]
pub async fn get_setting(
    key: String,
    state: State<'_, AppState>,
) -> Result<String, AppError> {
    let repo = SqliteSettingsRepository::new(state.pool.clone());
    Ok(repo.get(&key).await.unwrap_or_default())
}

/// Save a setting. If the key already exists, it is overwritten.
///
/// Errors with "key_required" if the key is empty.
#[tauri::command]
pub async fn save_setting(
    key: String,
    value: String,
    state: State<'_, AppState>,
) -> Result<(), AppError> {
    if key.is_empty() {
        return Err(AppError::InvalidInput("key_required".to_string()));
    }
    let repo = SqliteSettingsRepository::new(state.pool.clone());
    repo.save(&key, &value)
        .await
        .map_err(|e| AppError::Database(e.to_string()))
}

/// Delete a setting by key. If the key does not exist, this is a no-op.
#[tauri::command]
pub async fn delete_setting(
    key: String,
    state: State<'_, AppState>,
) -> Result<(), AppError> {
    let repo = SqliteSettingsRepository::new(state.pool.clone());
    repo.delete(&key)
        .await
        .map_err(|e| AppError::Database(e.to_string()))
}

/// Validate that a key is non-empty. Used internally by `save_setting`.
/// Exposed as a pure function for testing.
pub fn validate_key(key: &str) -> Result<(), String> {
    if key.is_empty() {
        Err("key_required".to_string())
    } else {
        Ok(())
    }
}

/// Core logic for `test_alert_channel`, extracted for testability.
///
/// Creates the appropriate `AlertDispatcher` based on channel type and
/// calls `test()` on it. Returns the result or an error string.
///
/// NOTE: The "app" channel is NOT handled here — it MUST be handled
/// at the Tauri command layer (where `AppHandle` is available) using
/// `tauri_plugin_notification` directly.  This function returns
/// `Err("unsupported_in_test")` for "app".
pub async fn test_alert_channel_cmd(
    channel: &str,
    config: &str,
    http_client: &reqwest::Client,
) -> Result<AlertTestResult, AppError> {
    match channel {
        "app" => Err(AppError::InvalidInput("unsupported_in_test".to_string())),
        "ntfy" => {
            let dispatcher = NtfyAlert::new(config.to_string())
                .map_err(|e| AppError::InvalidInput(format!("invalid_config: {e}")))?;
            Ok(dispatcher.test(http_client).await)
        }
        "webhook" => {
            let dispatcher = WebhookAlert::new(config.to_string())
                .map_err(|e| AppError::InvalidInput(format!("invalid_config: {e}")))?;
            Ok(dispatcher.test(http_client).await)
        }
        _ => Err(AppError::InvalidInput("invalid_channel".to_string())),
    }
}

/// Test an alert channel configuration.
///
/// Accepts "app", "ntfy", or "webhook" as the channel type.
/// For ntfy: config is the topic name.
/// For webhook: config is the URL.
/// For app: config is ignored (sends a real Tauri notification).
#[tauri::command]
pub async fn test_alert_channel(
    app: AppHandle,
    channel: String,
    config: String,
    state: State<'_, AppState>,
) -> Result<AlertTestResult, AppError> {
    match channel.as_str() {
        "app" => {
            // Send a real Tauri notification via the plugin.
            app.notification()
                .builder()
                .title("GuitarHub")
                .body("Test notification — alert channel is working!")
                .show()
                .map_err(|e| AppError::Internal(format!("notification_error: {e}")))?;
            Ok(AlertTestResult {
                success: true,
                message: "Test notification sent (app channel)".to_string(),
            })
        }
        _ => test_alert_channel_cmd(&channel, &config, &state.http_client).await,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repository::settings::MockSettingsRepository;
    use crate::AppError;

    /// Helper: create an in-memory pool with the settings table.
    async fn memory_pool() -> sqlx::SqlitePool {
        let pool = sqlx::SqlitePool::connect("sqlite::memory:")
            .await
            .unwrap();
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS settings (
                key   TEXT PRIMARY KEY,
                value TEXT NOT NULL
            )",
        )
        .execute(&pool)
        .await
        .unwrap();
        pool
    }

    // ── Pure function tests ─────────────────────────────────────────────

    #[test]
    fn validate_key_rejects_empty() {
        assert_eq!(validate_key(""), Err("key_required".to_string()));
    }

    #[test]
    fn validate_key_accepts_non_empty() {
        assert_eq!(validate_key("alert_channel"), Ok(()));
    }

    // ── Command-level tests ─────────────────────────────────────────────

    #[tokio::test]
    async fn save_then_get_roundtrip() {
        let pool = memory_pool().await;
        let repo = SqliteSettingsRepository::new(pool);

        repo.save("alert_channel", "webhook").await.unwrap();
        let val = repo.get("alert_channel").await;
        assert_eq!(val, Some("webhook".to_string()));
    }

    #[tokio::test]
    async fn get_unknown_key_returns_none() {
        let pool = memory_pool().await;
        let repo = SqliteSettingsRepository::new(pool);

        let val = repo.get("nonexistent").await;
        assert_eq!(val, None);
    }

    #[tokio::test]
    async fn empty_key_skipped_by_validation() {
        // The command layer rejects empty keys before reaching the repo
        assert!(validate_key("").is_err());
    }

    #[tokio::test]
    async fn mock_settings_repository_works_as_drop_in() {
        let mock = MockSettingsRepository::default();
        mock.save("test_key", "test_val").await.unwrap();
        assert_eq!(mock.get("test_key").await, Some("test_val".to_string()));
        assert_eq!(mock.get("missing").await, None);
    }

    #[tokio::test]
    async fn concurrent_save_and_read() {
        let pool = memory_pool().await;
        let repo = std::sync::Arc::new(SqliteSettingsRepository::new(pool));

        let mut handles = Vec::new();
        for i in 0..5 {
            let r = repo.clone();
            handles.push(tokio::spawn(async move {
                r.save(&format!("key_{i}"), &format!("val_{i}"))
                    .await
                    .unwrap();
                let v = r.get(&format!("key_{i}")).await;
                assert_eq!(v, Some(format!("val_{i}")));
            }));
        }
        for h in handles {
            h.await.unwrap();
        }
    }

    #[tokio::test]
    async fn delete_then_get_returns_none() {
        let pool = memory_pool().await;
        let repo = SqliteSettingsRepository::new(pool);

        repo.save("todelete", "value").await.unwrap();
        assert_eq!(repo.get("todelete").await, Some("value".to_string()));

        repo.delete("todelete").await.unwrap();
        assert_eq!(repo.get("todelete").await, None);
    }

    // ── test_alert_channel_cmd tests ────────────────────────────────────

    // ── Error mapping tests (AppError boundary) ─────────────────────────

    #[test]
    fn validate_key_maps_to_app_error() {
        let err = validate_key("").unwrap_err();
        let app_err = AppError::InvalidInput(err);
        assert!(matches!(app_err, AppError::InvalidInput(_)));
    }

    #[tokio::test]
    async fn test_alert_channel_cmd_invalid_channel_returns_error() {
        let http = reqwest::Client::new();
        let result = test_alert_channel_cmd("slack", "", &http).await;
        assert!(matches!(result, Err(AppError::InvalidInput(_))));
    }

    #[tokio::test]
    async fn test_alert_channel_cmd_app_returns_unsupported() {
        let http = reqwest::Client::new();
        let result = test_alert_channel_cmd("app", "", &http).await;
        assert!(matches!(result, Err(AppError::InvalidInput(_))));
    }

    #[tokio::test]
    async fn test_alert_channel_cmd_ntfy_with_bad_topic_returns_error() {
        let http = reqwest::Client::new();
        let result = test_alert_channel_cmd("ntfy", "", &http).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_alert_channel_cmd_webhook_with_bad_url_returns_error() {
        let http = reqwest::Client::new();
        let result = test_alert_channel_cmd("webhook", "not-a-url", &http).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_alert_channel_cmd_webhook_with_ip_url_returns_error() {
        let http = reqwest::Client::new();
        let result = test_alert_channel_cmd("webhook", "http://10.0.0.1/hook", &http).await;
        assert!(result.is_err());
    }
}
