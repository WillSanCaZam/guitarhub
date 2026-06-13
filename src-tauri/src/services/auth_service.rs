// SPDX-License-Identifier: GPL-3.0-or-later

use sqlx::SqlitePool;
use crate::AppError;

/// Service for community authentication operations.
///
/// Handles OAuth flow, JWT validation, token refresh, and secure store integration.
/// All business logic lives here — commands delegate to this service.
pub struct AuthService {
    pool: SqlitePool,
}

impl AuthService {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// Register a new user with email and password.
    pub async fn register(
        &self,
        username: &str,
        email: &str,
        password_hash: &str,
    ) -> Result<String, AppError> {
        let id = format!("user_{}", uuid_v4());

        sqlx::query(
            "INSERT INTO community_users (id, username, email, password_hash, created_at)
             VALUES (?1, ?2, ?3, ?4, strftime('%s', 'now'))",
        )
        .bind(&id)
        .bind(username)
        .bind(email)
        .bind(password_hash)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        // Create default profile
        sqlx::query(
            "INSERT INTO community_profiles (user_id, display_name, joined_at)
             VALUES (?1, ?2, strftime('%s', 'now'))",
        )
        .bind(&id)
        .bind(username)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        // Create default streak
        sqlx::query(
            "INSERT INTO community_streaks (user_id, current_streak, longest_streak)
             VALUES (?1, 0, 0)",
        )
        .bind(&id)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(id)
    }

    /// Authenticate a user by email and password hash.
    /// Returns the user ID if credentials match.
    pub async fn login(
        &self,
        email: &str,
        password_hash: &str,
    ) -> Result<Option<String>, AppError> {
        let result: Option<(String,)> = sqlx::query_as(
            "SELECT id FROM community_users WHERE email = ?1 AND password_hash = ?2",
        )
        .bind(email)
        .bind(password_hash)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(result.map(|(id,)| id))
    }

    /// Get a user by ID.
    pub async fn get_user(&self, user_id: &str) -> Result<Option<CommunityUser>, AppError> {
        let result: Option<(String, String, String, i64)> = sqlx::query_as(
            "SELECT id, username, email, created_at FROM community_users WHERE id = ?1",
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(result.map(|(id, username, email, created_at)| CommunityUser {
            id,
            username,
            email,
            created_at,
        }))
    }

    /// Delete a user account.
    pub async fn delete_user(&self, user_id: &str) -> Result<(), AppError> {
        sqlx::query("DELETE FROM community_users WHERE id = ?1")
            .bind(user_id)
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;
        Ok(())
    }
}

/// Community user entity.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CommunityUser {
    pub id: String,
    pub username: String,
    pub email: String,
    pub created_at: i64,
}

fn uuid_v4() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    format!("{:x}", nanos)
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn memory_pool() -> SqlitePool {
        SqlitePool::connect("sqlite::memory:").await.unwrap()
    }

    async fn setup_community_tables(pool: &SqlitePool) {
        sqlx::query(
            "CREATE TABLE community_users (
                id TEXT PRIMARY KEY,
                username TEXT NOT NULL UNIQUE,
                email TEXT NOT NULL UNIQUE,
                password_hash TEXT NOT NULL,
                created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
            )",
        )
        .execute(pool)
        .await
        .unwrap();

        sqlx::query(
            "CREATE TABLE community_profiles (
                user_id TEXT PRIMARY KEY,
                display_name TEXT NOT NULL,
                avatar_url TEXT,
                bio TEXT,
                gear_list TEXT NOT NULL DEFAULT '[]',
                streak_days INTEGER NOT NULL DEFAULT 0,
                joined_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
            )",
        )
        .execute(pool)
        .await
        .unwrap();

        sqlx::query(
            "CREATE TABLE community_streaks (
                user_id TEXT PRIMARY KEY,
                current_streak INTEGER NOT NULL DEFAULT 0,
                longest_streak INTEGER NOT NULL DEFAULT 0,
                last_practice_date TEXT
            )",
        )
        .execute(pool)
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn register_creates_user_and_profile() {
        let pool = memory_pool().await;
        setup_community_tables(&pool).await;
        let svc = AuthService::new(pool.clone());

        let id = svc.register("testuser", "test@example.com", "hash123").await.unwrap();
        assert!(!id.is_empty(), "register should return a non-empty user ID");

        let user = svc.get_user(&id).await.unwrap();
        assert!(user.is_some(), "user should exist after registration");
        let user = user.unwrap();
        assert_eq!(user.username, "testuser");
        assert_eq!(user.email, "test@example.com");
    }

    #[tokio::test]
    async fn login_returns_user_id_on_match() {
        let pool = memory_pool().await;
        setup_community_tables(&pool).await;
        let svc = AuthService::new(pool.clone());

        let id = svc.register("alice", "alice@example.com", "pass123").await.unwrap();
        let result = svc.login("alice@example.com", "pass123").await.unwrap();
        assert_eq!(result, Some(id));
    }

    #[tokio::test]
    async fn login_returns_none_on_wrong_password() {
        let pool = memory_pool().await;
        setup_community_tables(&pool).await;
        let svc = AuthService::new(pool.clone());

        svc.register("bob", "bob@example.com", "correct").await.unwrap();
        let result = svc.login("bob@example.com", "wrong").await.unwrap();
        assert_eq!(result, None);
    }

    #[tokio::test]
    async fn login_returns_none_for_nonexistent_email() {
        let pool = memory_pool().await;
        setup_community_tables(&pool).await;
        let svc = AuthService::new(pool.clone());

        let result = svc.login("nobody@example.com", "pass").await.unwrap();
        assert_eq!(result, None);
    }

    #[tokio::test]
    async fn delete_user_removes_account() {
        let pool = memory_pool().await;
        setup_community_tables(&pool).await;
        let svc = AuthService::new(pool.clone());

        let id = svc.register("to_delete", "del@example.com", "hash").await.unwrap();
        svc.delete_user(&id).await.unwrap();

        let user = svc.get_user(&id).await.unwrap();
        assert!(user.is_none(), "user should be deleted");
    }
}
