// SPDX-License-Identifier: GPL-3.0-or-later
use crate::repository::price_drop_notifications::PriceDropNotificationsRepo;
use crate::repository::settings::SettingsRepository;
use crate::repository::sqlite::settings::SqliteSettingsRepository;
use crate::services::alert_service::{AlertDispatcher, NtfyAlert, WebhookAlert};
use crate::services::price_drop::PriceDrop;
use crate::services::sync::{CatalogSyncService, SyncResult, SyncService};
use crate::AppState;
use tauri::{AppHandle, State};
use tauri_plugin_notification::NotificationExt;

/// Fetch a remote catalog JSON and upsert all products into the database.
#[tauri::command]
pub async fn sync_catalog(
    app: AppHandle,
    url: String,
    state: State<'_, AppState>,
) -> Result<SyncResult, crate::AppError> {
    let service = CatalogSyncService::new(state.pool.clone(), state.http_client.clone());
    let mut result = service.sync_catalog(&url).await?;

    if !result.drops.is_empty() {
        let settings_repo = SqliteSettingsRepository::new(state.pool.clone());
        let channel = settings_repo.get("alert_channel").await;
        let config = settings_repo.get("alert_config").await;

        if let Some(channel) = channel {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64;

            match channel.as_str() {
                "app" => {
                    let repo = PriceDropNotificationsRepo::new(state.pool.clone());
                    for drop in &result.drops {
                        let title = format!("Price Drop: {}", drop.sku);
                        let message = format!(
                            "{} dropped from ${:.2} to ${:.2}",
                            drop.sku, drop.previous_price, drop.new_price
                        );
                        match app
                            .notification()
                            .builder()
                            .title(&title)
                            .body(&message)
                            .show()
                        {
                            Ok(_) => {
                                if let Err(e) =
                                    repo.upsert(&drop.sku, now, drop.new_price, "app").await
                                {
                                    tracing::error!(
                                        "failed to record notification cooldown: {}",
                                        e
                                    );
                                }
                                result.drops_sent += 1;
                            }
                            Err(e) => {
                                tracing::error!("alert dispatch failed: {}", e);
                            }
                        }
                    }
                }
                _ => {
                    if let Some(dispatcher) = try_build_dispatcher(&channel, config.as_deref()) {
                        let sent = dispatch_drops(
                            &result.drops,
                            dispatcher.as_ref(),
                            &state.pool,
                            &state.http_client,
                            &channel,
                            now,
                        )
                        .await;
                        result.drops_sent += sent;
                    }
                }
            }
        }
    }

    Ok(result)
}

/// Attempt to build an `AlertDispatcher` from a channel string and optional config.
///
/// Returns `None` when:
/// - the channel is unknown,
/// - the channel requires config and it is missing, or
/// - the config is malformed.
///
/// Errors are logged by the caller; this function is silent.
pub fn try_build_dispatcher(
    channel: &str,
    config: Option<&str>,
) -> Option<Box<dyn AlertDispatcher>> {
    match channel {
        "ntfy" => {
            let config = config?;
            NtfyAlert::new(config.to_string())
                .ok()
                .map(|d| Box::new(d) as Box<dyn AlertDispatcher>)
        }
        "webhook" => {
            let config = config?;
            WebhookAlert::new(config.to_string())
                .ok()
                .map(|d| Box::new(d) as Box<dyn AlertDispatcher>)
        }
        _ => None,
    }
}

/// Send every `PriceDrop` through the provided dispatcher and record cooldown rows.
///
/// Individual send failures are logged and do **not** abort the loop.
/// Returns the count of successfully dispatched drops.
pub async fn dispatch_drops(
    drops: &[PriceDrop],
    dispatcher: &dyn AlertDispatcher,
    pool: &sqlx::SqlitePool,
    http_client: &reqwest::Client,
    channel: &str,
    now: i64,
) -> u32 {
    let repo = PriceDropNotificationsRepo::new(pool.clone());

    let mut sent = 0u32;
    for drop in drops {
        let title = format!("Price Drop: {}", drop.sku);
        let message = format!(
            "{} dropped from ${:.2} to ${:.2}",
            drop.sku, drop.previous_price, drop.new_price
        );
        match dispatcher.send(&title, &message, http_client).await {
            Ok(()) => {
                if let Err(e) = repo.upsert(&drop.sku, now, drop.new_price, channel).await {
                    tracing::error!("failed to record notification cooldown: {}", e);
                }
                sent += 1;
            }
            Err(e) => {
                tracing::error!("alert dispatch failed: {}", e);
            }
        }
    }
    sent
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::alert_service::AppNotificationAlert;
    use crate::services::price_drop::{DropReason, PriceDrop};
    use httpmock::prelude::*;
    use sqlx::SqlitePool;

    async fn setup_pool() -> SqlitePool {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        sqlx::query(
            "CREATE TABLE price_drop_notifications (
                sku           TEXT    PRIMARY KEY,
                last_notified INTEGER NOT NULL,
                last_price    REAL    NOT NULL,
                channel       TEXT    NOT NULL
            )",
        )
        .execute(&pool)
        .await
        .unwrap();
        pool
    }

    // ── dispatch_drops: mock dispatcher (always succeeds) ──────────────

    #[tokio::test]
    async fn dispatch_drops_records_cooldown_for_each_drop() {
        let pool = setup_pool().await;
        let client = reqwest::Client::new();
        let drops = vec![
            PriceDrop {
                sku: "SKU-A".to_string(),
                previous_price: 1000.0,
                new_price: 850.0,
                channel: "app".to_string(),
                reason: DropReason::Relative,
            },
            PriceDrop {
                sku: "SKU-B".to_string(),
                previous_price: 500.0,
                new_price: 400.0,
                channel: "app".to_string(),
                reason: DropReason::Absolute,
            },
        ];

        let dispatcher = AppNotificationAlert;
        let sent = dispatch_drops(&drops, &dispatcher, &pool, &client, "app", 1_700_000_000).await;

        assert_eq!(sent, 2, "both drops should be dispatched");

        let repo = PriceDropNotificationsRepo::new(pool);
        let last_a = repo.get_last_notified("SKU-A").await.unwrap();
        let last_b = repo.get_last_notified("SKU-B").await.unwrap();
        assert_eq!(last_a, Some(1_700_000_000));
        assert_eq!(last_b, Some(1_700_000_000));
    }

    // ── dispatch_drops: HTTP integration via NtfyAlert + httpmock ────────

    #[tokio::test]
    async fn dispatch_drops_ntfy_sends_request_and_records_cooldown() {
        let server = MockServer::start();
        let mock = server.mock(|when, then| {
            when.method(POST).path("/guitar-deals");
            then.status(200)
                .json_body(serde_json::json!({"ok": true}));
        });

        let pool = setup_pool().await;
        let client = reqwest::Client::new();
        let drops = vec![PriceDrop {
            sku: "SKU-DROP-2".to_string(),
            previous_price: 500.0,
            new_price: 400.0,
            channel: "ntfy".to_string(),
            reason: DropReason::Absolute,
        }];

        let mut alert = NtfyAlert::new("guitar-deals".to_string()).unwrap();
        alert.base_url = server.base_url();

        let sent = dispatch_drops(&drops, &alert, &pool, &client, "ntfy", 1_700_000_000).await;

        assert_eq!(sent, 1, "expected 1 drop sent via ntfy");
        mock.assert_hits(1);

        let repo = PriceDropNotificationsRepo::new(pool);
        let last = repo.get_last_notified("SKU-DROP-2").await.unwrap();
        assert_eq!(last, Some(1_700_000_000));
    }

    // ── dispatch_drops: failure path (HTTP 500) does not abort loop ────

    #[tokio::test]
    async fn dispatch_drops_handles_single_failure_and_continues() {
        let server = MockServer::start();
        let mock = server.mock(|when, then| {
            when.method(POST).path("/guitar-deals");
            then.status(500);
        });

        let pool = setup_pool().await;
        let client = reqwest::Client::new();
        let drops = vec![PriceDrop {
            sku: "SKU-FAIL".to_string(),
            previous_price: 500.0,
            new_price: 400.0,
            channel: "ntfy".to_string(),
            reason: DropReason::Absolute,
        }];

        let mut alert = NtfyAlert::new("guitar-deals".to_string()).unwrap();
        alert.base_url = server.base_url();

        let sent = dispatch_drops(&drops, &alert, &pool, &client, "ntfy", 1_700_000_000).await;

        assert_eq!(sent, 0, "HTTP 500 means 0 sent");
        // Retry-once sends two requests on failure.
        mock.assert_hits(2);

        let repo = PriceDropNotificationsRepo::new(pool);
        let last = repo.get_last_notified("SKU-FAIL").await.unwrap();
        assert!(last.is_none(), "cooldown must NOT be recorded on failure");
    }

    // ── try_build_dispatcher edge cases ─────────────────────────────────

    #[test]
    fn try_build_dispatcher_returns_none_for_unknown_channel() {
        assert!(try_build_dispatcher("slack", Some("x")).is_none());
    }

    #[test]
    fn try_build_dispatcher_returns_none_for_missing_config() {
        assert!(try_build_dispatcher("webhook", None).is_none());
        assert!(try_build_dispatcher("ntfy", None).is_none());
    }

    #[test]
    fn try_build_dispatcher_returns_none_for_invalid_config() {
        assert!(try_build_dispatcher("webhook", Some("not-a-url")).is_none());
        assert!(try_build_dispatcher("ntfy", Some("")).is_none());
    }

    #[test]
    fn try_build_dispatcher_ntfy_with_valid_topic() {
        let d = try_build_dispatcher("ntfy", Some("guitar-deals"));
        assert!(d.is_some());
    }
}
