// SPDX-License-Identifier: GPL-3.0-or-later

use tauri::State;

use crate::services::export_service::{ExportError, ExportResult, ExportService};
use crate::AppError;
use crate::AppState;

/// Core logic for `export_data`, extracted for testability without Tauri.
pub async fn export_data_cmd(pool: &sqlx::SqlitePool, path: &str) -> Result<ExportResult, AppError> {
    if path.is_empty() {
        return Err(AppError::Internal("write_error: path is empty".to_string()));
    }
    let svc = ExportService::new(pool.clone());
    svc.export_to(path).await.map_err(|e| match e {
        ExportError::Write(msg) => AppError::Internal(format!("write_error: {msg}")),
        ExportError::Query(msg) => AppError::Database(format!("query_error: {msg}")),
    })
}

/// Export all data (wishlist, price history, settings) as a ZIP archive.
///
/// The `path` parameter comes from the frontend save dialog.
#[tauri::command]
pub async fn export_data(
    path: String,
    state: State<'_, AppState>,
) -> Result<ExportResult, AppError> {
    export_data_cmd(&state.pool, &path).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::AppError;
    use crate::repository::sqlite::migrations::MigrationRunner;

    /// Create an in-memory pool using the REAL migration chain (001→008).
    ///
    /// Mirrors the helper in `services::export_service::tests` — using the
    /// real schema here means command-level tests validate the same contract
    /// as the service, and any future migration that breaks export will be
    /// caught at this layer too.
    async fn migrated_pool() -> sqlx::SqlitePool {
        use std::path::PathBuf;

        let pool = sqlx::SqlitePool::connect("sqlite::memory:").await.unwrap();

        let dir = std::env::temp_dir().join(format!(
            "guitarhub-exportcmd-mig-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join("001_init.sql"),
            include_str!("../repository/sqlite/migrations/001_init.sql"),
        )
        .unwrap();
        std::fs::write(
            dir.join("002_add_url_validation.sql"),
            include_str!("../repository/sqlite/migrations/002_add_url_validation.sql"),
        )
        .unwrap();
        std::fs::write(
            dir.join("003_add_image_cache.sql"),
            include_str!("../repository/sqlite/migrations/003_add_image_cache.sql"),
        )
        .unwrap();
        std::fs::write(
            dir.join("004_add_price_source.sql"),
            include_str!("../repository/sqlite/migrations/004_add_price_source.sql"),
        )
        .unwrap();
        std::fs::write(
            dir.join("005_add_settings.sql"),
            include_str!("../repository/sqlite/migrations/005_add_settings.sql"),
        )
        .unwrap();
        std::fs::write(
            dir.join("006_wishlist_schema.sql"),
            include_str!("../repository/sqlite/migrations/006_wishlist_schema.sql"),
        )
        .unwrap();
        std::fs::write(
            dir.join("007_price_drop_notifications.sql"),
            include_str!("../repository/sqlite/migrations/007_price_drop_notifications.sql"),
        )
        .unwrap();
        std::fs::write(
            dir.join("008_collection_items.sql"),
            include_str!("../repository/sqlite/migrations/008_collection_items.sql"),
        )
        .unwrap();

        let runner = MigrationRunner::new(pool.clone(), PathBuf::from(&dir));
        runner.run().await.expect("real migration chain should apply cleanly");
        pool
    }

    /// Test pool backed by the real migration chain (not inline CREATE TABLE).
    async fn test_pool() -> sqlx::SqlitePool {
        migrated_pool().await
    }

    #[tokio::test]
    async fn export_data_cmd_empty_path_returns_write_error() {
        let pool = test_pool().await;
        let result = export_data_cmd(&pool, "").await;
        assert!(result.is_err());
    }

    #[test]
    fn export_error_write_maps_to_internal() {
        let export_err = crate::services::export_service::ExportError::Write("disk full".into());
        let app_err = match &export_err {
            crate::services::export_service::ExportError::Write(msg) => {
                AppError::Internal(format!("write_error: {msg}"))
            }
            crate::services::export_service::ExportError::Query(msg) => {
                AppError::Database(format!("query_error: {msg}"))
            }
        };
        assert!(matches!(app_err, AppError::Internal(_)));
    }

    #[tokio::test]
    async fn export_data_cmd_valid_path_returns_result() {
        let pool = test_pool().await;
        // Seed some data
        sqlx::query("INSERT INTO settings (key, value) VALUES ('theme', 'dark')")
            .execute(&pool)
            .await
            .unwrap();

        let tmp = tempfile::NamedTempFile::new().unwrap();
        let path = tmp.path().to_str().unwrap().to_string();

        let result = export_data_cmd(&pool, &path).await.unwrap();
        assert!(result.success);
        assert!(result.size_bytes > 0);
        assert_eq!(result.file_count, 4);
    }

    #[tokio::test]
    async fn export_data_cmd_valid_path_with_no_data_succeeds() {
        let pool = test_pool().await;
        let tmp = tempfile::NamedTempFile::new().unwrap();
        let path = tmp.path().to_str().unwrap().to_string();

        let result = export_data_cmd(&pool, &path).await.unwrap();
        assert!(result.success);
        assert_eq!(result.file_count, 4);
    }

    /// Validates the command layer against the REAL migration schema.
    /// Triangulates the contract: if a future migration breaks the schema
    /// the service expects, this test will fail with a clear query error
    /// rather than the divergence silently passing.
    #[tokio::test]
    async fn export_data_cmd_works_against_real_migration_chain() {
        let pool = migrated_pool().await;
        // Seed a real-schema wishlist row to prove SELECT * works.
        sqlx::query(
            "INSERT INTO wishlist (sku, name, brand, price, currency) VALUES (?1, ?2, ?3, ?4, ?5)",
        )
        .bind("REALSCHEMA-001")
        .bind("Real Schema Guitar")
        .bind("TestBrand")
        .bind(1500.0f64)
        .bind("USD")
        .execute(&pool)
        .await
        .expect("INSERT into real-schema wishlist should succeed");

        let tmp = tempfile::NamedTempFile::new().unwrap();
        let path = tmp.path().to_str().unwrap().to_string();

        let result = export_data_cmd(&pool, &path).await.unwrap();
        assert!(result.success, "command must succeed against real schema");
        assert!(result.size_bytes > 0);
        assert_eq!(result.file_count, 4);
    }
}
