// SPDX-License-Identifier: GPL-3.0-or-later

use sqlx::SqlitePool;
use crate::AppError;

/// Raw row type for profile queries — matches the SELECT column order.
type ProfileRow = (String, String, Option<String>, Option<String>, String, i32, i64);

/// Service for user profile and streak operations.
pub struct ProfileService {
    pool: SqlitePool,
}

impl ProfileService {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// Get a user profile.
    pub async fn get_profile(&self, user_id: &str) -> Result<Option<Profile>, AppError> {
        let result: Option<ProfileRow> =
            sqlx::query_as(
                "SELECT user_id, display_name, avatar_url, bio, gear_list, streak_days, joined_at
                 FROM community_profiles WHERE user_id = ?1",
            )
            .bind(user_id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(result.map(|(user_id, display_name, avatar_url, bio, gear_list, streak_days, joined_at)| {
            Profile { user_id, display_name, avatar_url, bio, gear_list, streak_days, joined_at }
        }))
    }

    /// Update a user profile.
    pub async fn update_profile(
        &self,
        user_id: &str,
        display_name: Option<&str>,
        bio: Option<&str>,
        avatar_url: Option<&str>,
    ) -> Result<(), AppError> {
        if let Some(name) = display_name {
            sqlx::query("UPDATE community_profiles SET display_name = ?1 WHERE user_id = ?2")
                .bind(name)
                .bind(user_id)
                .execute(&self.pool)
                .await
                .map_err(|e| AppError::Database(e.to_string()))?;
        }
        if let Some(b) = bio {
            sqlx::query("UPDATE community_profiles SET bio = ?1 WHERE user_id = ?2")
                .bind(b)
                .bind(user_id)
                .execute(&self.pool)
                .await
                .map_err(|e| AppError::Database(e.to_string()))?;
        }
        if let Some(url) = avatar_url {
            sqlx::query("UPDATE community_profiles SET avatar_url = ?1 WHERE user_id = ?2")
                .bind(url)
                .bind(user_id)
                .execute(&self.pool)
                .await
                .map_err(|e| AppError::Database(e.to_string()))?;
        }
        Ok(())
    }

    /// Get a user's streak data.
    pub async fn get_streak(&self, user_id: &str) -> Result<Option<StreakData>, AppError> {
        let result: Option<(String, i32, i32, Option<String>)> = sqlx::query_as(
            "SELECT user_id, current_streak, longest_streak, last_practice_date
             FROM community_streaks WHERE user_id = ?1",
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(result.map(|(user_id, current_streak, longest_streak, last_practice_date)| {
            StreakData { user_id, current_streak, longest_streak, last_practice_date }
        }))
    }

    /// Update streak after a practice session.
    pub async fn update_streak(&self, user_id: &str) -> Result<(), AppError> {
        // Use SQLite's built-in date function for today's date
        sqlx::query(
            "UPDATE community_streaks
             SET current_streak = current_streak + 1,
                 longest_streak = MAX(longest_streak, current_streak + 1),
                 last_practice_date = date('now')
             WHERE user_id = ?1",
        )
        .bind(user_id)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
        Ok(())
    }

    /// Add a gear item to user's gear list.
    pub async fn add_gear_to_list(&self, user_id: &str, gear_sku: &str) -> Result<(), AppError> {
        // Read current gear list, append, and update
        let current: Option<(String,)> = sqlx::query_as(
            "SELECT gear_list FROM community_profiles WHERE user_id = ?1",
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        if let Some((gear_list_json,)) = current {
            let mut gear_list: Vec<String> = serde_json::from_str(&gear_list_json)
                .unwrap_or_default();
            if !gear_list.contains(&gear_sku.to_string()) {
                gear_list.push(gear_sku.to_string());
            }
            let new_json = serde_json::to_string(&gear_list)
                .map_err(|e| AppError::Internal(e.to_string()))?;
            sqlx::query("UPDATE community_profiles SET gear_list = ?1 WHERE user_id = ?2")
                .bind(&new_json)
                .bind(user_id)
                .execute(&self.pool)
                .await
                .map_err(|e| AppError::Database(e.to_string()))?;
        }
        Ok(())
    }
}

/// User profile entity.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Profile {
    pub user_id: String,
    pub display_name: String,
    pub avatar_url: Option<String>,
    pub bio: Option<String>,
    pub gear_list: String,
    pub streak_days: i32,
    pub joined_at: i64,
}

/// Streak data entity.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct StreakData {
    pub user_id: String,
    pub current_streak: i32,
    pub longest_streak: i32,
    pub last_practice_date: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn memory_pool() -> SqlitePool {
        SqlitePool::connect("sqlite::memory:").await.unwrap()
    }

    async fn setup(pool: &SqlitePool) {
        sqlx::query(
            "CREATE TABLE community_users (
                id TEXT PRIMARY KEY, username TEXT NOT NULL UNIQUE,
                email TEXT NOT NULL UNIQUE, password_hash TEXT NOT NULL,
                created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
            )",
        ).execute(pool).await.unwrap();
        sqlx::query(
            "CREATE TABLE community_profiles (
                user_id TEXT PRIMARY KEY, display_name TEXT NOT NULL,
                avatar_url TEXT, bio TEXT,
                gear_list TEXT NOT NULL DEFAULT '[]',
                streak_days INTEGER NOT NULL DEFAULT 0,
                joined_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
            )",
        ).execute(pool).await.unwrap();
        sqlx::query(
            "CREATE TABLE community_streaks (
                user_id TEXT PRIMARY KEY,
                current_streak INTEGER NOT NULL DEFAULT 0,
                longest_streak INTEGER NOT NULL DEFAULT 0,
                last_practice_date TEXT
            )",
        ).execute(pool).await.unwrap();
        // Seed user + profile + streak
        sqlx::query(
            "INSERT INTO community_users (id, username, email, password_hash, created_at)
             VALUES ('u1', 'tester', 't@test.com', 'hash', 1700000000)",
        ).execute(pool).await.unwrap();
        sqlx::query(
            "INSERT INTO community_profiles (user_id, display_name, joined_at)
             VALUES ('u1', 'Tester', 1700000000)",
        ).execute(pool).await.unwrap();
        sqlx::query(
            "INSERT INTO community_streaks (user_id, current_streak, longest_streak)
             VALUES ('u1', 3, 7)",
        ).execute(pool).await.unwrap();
    }

    #[tokio::test]
    async fn get_profile_returns_data() {
        let pool = memory_pool().await;
        setup(&pool).await;
        let svc = ProfileService::new(pool);

        let profile = svc.get_profile("u1").await.unwrap();
        assert!(profile.is_some());
        let p = profile.unwrap();
        assert_eq!(p.display_name, "Tester");
        assert_eq!(p.gear_list, "[]");
    }

    #[tokio::test]
    async fn update_profile_changes_display_name() {
        let pool = memory_pool().await;
        setup(&pool).await;
        let svc = ProfileService::new(pool);

        svc.update_profile("u1", Some("New Name"), None, None).await.unwrap();
        let p = svc.get_profile("u1").await.unwrap().unwrap();
        assert_eq!(p.display_name, "New Name");
    }

    #[tokio::test]
    async fn get_streak_returns_data() {
        let pool = memory_pool().await;
        setup(&pool).await;
        let svc = ProfileService::new(pool);

        let streak = svc.get_streak("u1").await.unwrap();
        assert!(streak.is_some());
        let s = streak.unwrap();
        assert_eq!(s.current_streak, 3);
        assert_eq!(s.longest_streak, 7);
    }

    #[tokio::test]
    async fn add_gear_to_list_appends_sku() {
        let pool = memory_pool().await;
        setup(&pool).await;
        let svc = ProfileService::new(pool);

        svc.add_gear_to_list("u1", "SKU-001").await.unwrap();
        svc.add_gear_to_list("u1", "SKU-002").await.unwrap();

        let p = svc.get_profile("u1").await.unwrap().unwrap();
        let gear: Vec<String> = serde_json::from_str(&p.gear_list).unwrap();
        assert_eq!(gear, vec!["SKU-001", "SKU-002"]);
    }

    #[tokio::test]
    async fn add_gear_to_list_deduplicates() {
        let pool = memory_pool().await;
        setup(&pool).await;
        let svc = ProfileService::new(pool);

        svc.add_gear_to_list("u1", "SKU-001").await.unwrap();
        svc.add_gear_to_list("u1", "SKU-001").await.unwrap();

        let p = svc.get_profile("u1").await.unwrap().unwrap();
        let gear: Vec<String> = serde_json::from_str(&p.gear_list).unwrap();
        assert_eq!(gear.len(), 1);
    }
}
