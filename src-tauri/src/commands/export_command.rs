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

    async fn test_pool() -> sqlx::SqlitePool {
        let pool = sqlx::SqlitePool::connect("sqlite::memory:").await.unwrap();

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS wishlist (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                sku TEXT, name TEXT, brand TEXT, price REAL,
                currency TEXT, image_url TEXT, product_url TEXT,
                notes TEXT, added_at INTEGER
            )",
        )
        .execute(&pool)
        .await
        .unwrap();

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS price_history (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                sku TEXT NOT NULL, price REAL NOT NULL,
                recorded_at INTEGER NOT NULL, source_id TEXT NOT NULL DEFAULT ''
            )",
        )
        .execute(&pool)
        .await
        .unwrap();

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS settings (
                key TEXT PRIMARY KEY, value TEXT NOT NULL
            )",
        )
        .execute(&pool)
        .await
        .unwrap();

        pool
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
        assert_eq!(result.file_count, 3);
    }

    #[tokio::test]
    async fn export_data_cmd_valid_path_with_no_data_succeeds() {
        let pool = test_pool().await;
        let tmp = tempfile::NamedTempFile::new().unwrap();
        let path = tmp.path().to_str().unwrap().to_string();

        let result = export_data_cmd(&pool, &path).await.unwrap();
        assert!(result.success);
        assert_eq!(result.file_count, 3);
    }
}
