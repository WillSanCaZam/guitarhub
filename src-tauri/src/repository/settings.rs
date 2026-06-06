// SPDX-License-Identifier: GPL-3.0-or-later

use async_trait::async_trait;

/// Repository for app settings (key-value store).
///
/// Each setting is a string key-value pair persisted in the `settings` table.
/// Unknown keys return `None` (not an error).
#[async_trait]
pub trait SettingsRepository: Send + Sync {
    /// Retrieve a setting by key. Returns `None` if the key does not exist.
    async fn get(&self, key: &str) -> Option<String>;

    /// Save a setting. If the key already exists, it is overwritten (UPSERT).
    async fn save(&self, key: &str, value: &str) -> Result<(), sqlx::Error>;

    /// Delete a setting by key. If the key does not exist, this is a no-op.
    async fn delete(&self, key: &str) -> Result<(), sqlx::Error>;
}

// ── Mock (public in test builds) ─────────────────────────────────────────

/// Thread-safe mock implementation of `SettingsRepository` for testing.
///
/// Uses `std::sync::Mutex` for interior mutability so it can be used
/// in place of the real repository in unit tests.
#[derive(Debug, Default)]
pub struct MockSettingsRepository {
    data: std::sync::Mutex<std::collections::HashMap<String, String>>,
}

#[async_trait]
impl SettingsRepository for MockSettingsRepository {
    async fn get(&self, key: &str) -> Option<String> {
        let data = self.data.lock().expect("mock lock poisoned");
        data.get(key).cloned()
    }

    async fn save(&self, key: &str, value: &str) -> Result<(), sqlx::Error> {
        let mut data = self.data.lock().expect("mock lock poisoned");
        data.insert(key.to_string(), value.to_string());
        Ok(())
    }

    async fn delete(&self, key: &str) -> Result<(), sqlx::Error> {
        let mut data = self.data.lock().expect("mock lock poisoned");
        data.remove(key);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn mock_save_and_get_roundtrip() {
        let repo = MockSettingsRepository::default();
        repo.save("alert_channel", "ntfy").await.unwrap();
        let val = repo.get("alert_channel").await;
        assert_eq!(val, Some("ntfy".to_string()));
    }

    #[tokio::test]
    async fn mock_get_unknown_key_returns_none() {
        let repo = MockSettingsRepository::default();
        let val = repo.get("nonexistent").await;
        assert_eq!(val, None);
    }

    #[tokio::test]
    async fn mock_delete_removes_key() {
        let repo = MockSettingsRepository::default();
        repo.save("key1", "val1").await.unwrap();
        repo.delete("key1").await.unwrap();
        let val = repo.get("key1").await;
        assert_eq!(val, None);
    }

    #[tokio::test]
    async fn mock_save_overwrites_existing_key() {
        let repo = MockSettingsRepository::default();
        repo.save("key", "old").await.unwrap();
        repo.save("key", "new").await.unwrap();
        let val = repo.get("key").await;
        assert_eq!(val, Some("new".to_string()));
    }

    #[tokio::test]
    async fn mock_delete_nonexistent_key_is_noop() {
        let repo = MockSettingsRepository::default();
        // Should not error
        repo.delete("nonexistent").await.unwrap();
    }
}
