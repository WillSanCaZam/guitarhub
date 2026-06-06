// SPDX-License-Identifier: GPL-3.0-or-later

pub mod commands;
pub mod domain;
pub mod repository;
pub mod services;

use repository::sqlite::migrations::MigrationRunner;
use services::image_cache::ImageCacheService;
use sqlx::sqlite::SqlitePoolOptions;

/// Shared application state injected into Tauri commands.
///
/// Clone is safe: `SqlitePool`, `ImageCacheService`, and `reqwest::Client`
/// are all internally `Arc`-based.
#[derive(Clone)]
pub struct AppState {
    pub pool: sqlx::SqlitePool,
    pub image_cache_service: ImageCacheService,
    pub http_client: reqwest::Client,
}

/// Initialize the database connection, run pending migrations, and
/// return an `AppState` ready for Tauri or direct use.
pub async fn initialize_database(db_path: &str) -> anyhow::Result<AppState> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "guitarhub=info".into()),
        )
        .json()
        .init();
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect(db_path)
        .await?;

    // Enable WAL journal mode before any schema operations.
    // Must run BEFORE the first write to the database.
    {
        let result: String = sqlx::query_scalar("PRAGMA journal_mode=WAL;")
            .fetch_one(&pool)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to set WAL journal mode: {e}"))?;
        tracing::info!("SQLite journal mode set to: {result}");
    }

    let migrations_dir = std::env::var("GUITARHUB_MIGRATIONS_DIR")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| {
            let crate_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
            crate_dir.join("src/repository/sqlite/migrations")
        });

    let runner = MigrationRunner::new(pool.clone(), migrations_dir);
    runner.run().await?;

    let image_cache_service = ImageCacheService::new_default(pool.clone());

    let http_client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .user_agent("GuitarHub/0.1")
        .build()
        .map_err(|e| anyhow::anyhow!("Failed to build HTTP client: {e}"))?;

    Ok(AppState {
        pool,
        image_cache_service,
        http_client,
    })
}

/// Unified application error for Tauri IPC commands.
///
/// Each variant carries a user-facing message via `Display`.
/// Serializes as a plain string so existing frontend error handling
/// continues to work without changes.
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("not found")]
    NotFound,
    #[error("invalid input: {0}")]
    InvalidInput(String),
    #[error("database error: {0}")]
    Database(String),
    #[error("network error: {0}")]
    Network(String),
    #[error("internal error: {0}")]
    Internal(String),
    #[error("sync already in progress")]
    SyncInProgress,
}

impl serde::Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl From<sqlx::Error> for AppError {
    fn from(e: sqlx::Error) -> Self {
        AppError::Database(e.to_string())
    }
}

impl From<anyhow::Error> for AppError {
    fn from(e: anyhow::Error) -> Self {
        AppError::Internal(e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn app_error_not_found_display() {
        let err = AppError::NotFound;
        assert_eq!(err.to_string(), "not found");
    }

    #[test]
    fn app_error_invalid_input_display() {
        let err = AppError::InvalidInput("bad sku".into());
        assert_eq!(err.to_string(), "invalid input: bad sku");
    }

    #[test]
    fn app_error_database_display() {
        let err = AppError::Database("conn failed".into());
        assert_eq!(err.to_string(), "database error: conn failed");
    }

    #[test]
    fn app_error_network_display() {
        let err = AppError::Network("timeout".into());
        assert_eq!(err.to_string(), "network error: timeout");
    }

    #[test]
    fn app_error_internal_display() {
        let err = AppError::Internal("oops".into());
        assert_eq!(err.to_string(), "internal error: oops");
    }

    #[test]
    fn app_error_serializes_to_string() {
        let err = AppError::InvalidInput("bad".into());
        let json = serde_json::to_string(&err).unwrap();
        assert_eq!(json, r#""invalid input: bad""#);
    }

    #[test]
    fn app_error_from_sqlx_error() {
        let sqlx_err = sqlx::Error::PoolClosed;
        let app_err: AppError = sqlx_err.into();
        assert!(
            app_err.to_string().contains("database error"),
            "got: {}",
            app_err
        );
    }

    #[test]
    fn app_error_from_anyhow_error() {
        let anyhow_err = anyhow::anyhow!("something went wrong");
        let app_err: AppError = anyhow_err.into();
        assert!(
            app_err.to_string().contains("internal error"),
            "got: {}",
            app_err
        );
    }
}
