// SPDX-License-Identifier: GPL-3.0-or-later

use async_trait::async_trait;
use sqlx::SqlitePool;

use crate::repository::settings::SettingsRepository;

/// SQLite-backed implementation of `SettingsRepository`.
///
/// Uses direct UPSERT/SELECT queries against the `settings` table.
/// The table must exist (created by migration 005).
#[derive(Debug, Clone)]
pub struct SqliteSettingsRepository {
    pub(crate) pool: SqlitePool,
}

impl SqliteSettingsRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl SettingsRepository for SqliteSettingsRepository {
    async fn get(&self, key: &str) -> Option<String> {
        sqlx::query_scalar::<_, String>(
            "SELECT value FROM settings WHERE key = ?1",
        )
        .bind(key)
        .fetch_optional(&self.pool)
        .await
        .ok()
        .flatten()
    }

    async fn save(&self, key: &str, value: &str) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT OR REPLACE INTO settings (key, value) VALUES (?1, ?2)",
        )
        .bind(key)
        .bind(value)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn delete(&self, key: &str) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM settings WHERE key = ?1")
            .bind(key)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Create an in-memory SQLite pool with the settings table.
    async fn test_pool() -> SqlitePool {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
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

    #[tokio::test]
    async fn save_and_get_roundtrip() {
        let pool = test_pool().await;
        let repo = SqliteSettingsRepository::new(pool);

        repo.save("alert_channel", "ntfy").await.unwrap();
        let val = repo.get("alert_channel").await;
        assert_eq!(val, Some("ntfy".to_string()));
    }

    #[tokio::test]
    async fn get_unknown_key_returns_none() {
        let pool = test_pool().await;
        let repo = SqliteSettingsRepository::new(pool);

        let val = repo.get("nonexistent").await;
        assert_eq!(val, None);
    }

    #[tokio::test]
    async fn save_overwrites_existing_key() {
        let pool = test_pool().await;
        let repo = SqliteSettingsRepository::new(pool);

        repo.save("key", "old").await.unwrap();
        repo.save("key", "new").await.unwrap();

        let val = repo.get("key").await;
        assert_eq!(val, Some("new".to_string()));
    }

    #[tokio::test]
    async fn delete_removes_key() {
        let pool = test_pool().await;
        let repo = SqliteSettingsRepository::new(pool);

        repo.save("key", "val").await.unwrap();
        repo.delete("key").await.unwrap();

        let val = repo.get("key").await;
        assert_eq!(val, None);
    }

    #[tokio::test]
    async fn delete_nonexistent_key_is_noop() {
        let pool = test_pool().await;
        let repo = SqliteSettingsRepository::new(pool);

        // Should not error
        repo.delete("nonexistent").await.unwrap();
    }

    #[tokio::test]
    async fn structured_json_survives_roundtrip() {
        let pool = test_pool().await;
        let repo = SqliteSettingsRepository::new(pool);

        let json_value = r#"{"channel":"ntfy","topic":"guitar-deals"}"#;
        repo.save("alert_config", json_value).await.unwrap();
        let val = repo.get("alert_config").await;
        assert_eq!(val, Some(json_value.to_string()));
    }
}
