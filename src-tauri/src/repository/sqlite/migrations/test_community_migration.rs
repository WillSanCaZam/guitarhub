// SPDX-License-Identifier: GPL-3.0-or-later

use crate::repository::sqlite::migrations::{split_statements, MigrationRunner};
use std::path::PathBuf;

async fn make_memory_pool() -> sqlx::SqlitePool {
    sqlx::SqlitePool::connect("sqlite::memory:")
        .await
        .expect("in-memory pool")
}

fn temp_dir_with_files(files: &[&str]) -> PathBuf {
    let dir = tempfile::tempdir().unwrap().into_path();
    for f in files {
        std::fs::write(dir.join(f), "-- placeholder").unwrap();
    }
    dir
}

/// Apply the full migration chain 001→011 including soft-delete columns.
async fn apply_full_chain(pool: &sqlx::SqlitePool) -> PathBuf {
    let dir = temp_dir_with_files(&[
        "001_init.sql",
        "002_add_url_validation.sql",
        "003_add_image_cache.sql",
        "004_add_price_source.sql",
        "005_add_settings.sql",
        "006_wishlist_schema.sql",
        "007_price_drop_notifications.sql",
        "008_collection_items.down.sql",
        "008_collection_items.sql",
        "009_add_recent_searches.sql",
        "010_community_schema.sql",
        "011_soft_delete.sql",
    ]);
    std::fs::write(dir.join("001_init.sql"), include_str!("../migrations/001_init.sql")).unwrap();
    std::fs::write(dir.join("002_add_url_validation.sql"), include_str!("../migrations/002_add_url_validation.sql")).unwrap();
    std::fs::write(dir.join("003_add_image_cache.sql"), include_str!("../migrations/003_add_image_cache.sql")).unwrap();
    std::fs::write(dir.join("004_add_price_source.sql"), include_str!("../migrations/004_add_price_source.sql")).unwrap();
    std::fs::write(dir.join("005_add_settings.sql"), include_str!("../migrations/005_add_settings.sql")).unwrap();
    std::fs::write(dir.join("006_wishlist_schema.sql"), include_str!("../migrations/006_wishlist_schema.sql")).unwrap();
    std::fs::write(dir.join("007_price_drop_notifications.sql"), include_str!("../migrations/007_price_drop_notifications.sql")).unwrap();
    std::fs::write(dir.join("008_collection_items.sql"), include_str!("../migrations/008_collection_items.sql")).unwrap();
    std::fs::write(dir.join("008_collection_items.down.sql"), include_str!("../migrations/008_collection_items.down.sql")).unwrap();
    std::fs::write(dir.join("009_add_recent_searches.sql"), include_str!("../migrations/009_add_recent_searches.sql")).unwrap();
    std::fs::write(dir.join("010_community_schema.sql"), include_str!("../repository/sqlite/migrations/010_community_schema.sql")).unwrap();
    std::fs::write(dir.join("011_soft_delete.sql"), include_str!("../migrations/011_soft_delete.sql")).unwrap();

    let runner = MigrationRunner::new(pool.clone(), dir.clone());
    runner.run().await.unwrap();
    dir
}

// ── Migration 010: Community Schema ─────────────────────────────────

#[tokio::test]
async fn migration_010_creates_community_tables() {
    let pool = make_memory_pool().await;
    apply_full_chain(&pool).await;

    // Verify all community tables exist
    let tables = [
        "community_users",
        "community_profiles",
        "community_lessons",
        "community_riffs",
        "community_comments",
        "community_streaks",
        "community_follows",
        "community_cache",
    ];

    for table in &tables {
        let count: i64 = sqlx::query_scalar(&format!("SELECT COUNT(*) FROM {table}"))
            .fetch_one(&pool)
            .await
            .unwrap_or(-1);
        assert!(
            count >= 0,
            "Table {table} should exist and be queryable, got count={count}"
        );
    }
}

#[tokio::test]
async fn migration_010_community_users_has_expected_columns() {
    let pool = make_memory_pool().await;
    apply_full_chain(&pool).await;

    let columns: Vec<(i64, String, String)> = sqlx::query_as("PRAGMA table_info(community_users)")
        .fetch_all(&pool)
        .await
        .unwrap();

    let col_names: Vec<&str> = columns.iter().map(|(_, name, _)| name.as_str()).collect();
    assert!(col_names.contains(&"id"), "missing id column");
    assert!(col_names.contains(&"username"), "missing username column");
    assert!(col_names.contains(&"email"), "missing email column");
    assert!(col_names.contains(&"password_hash"), "missing password_hash column");
    assert!(col_names.contains(&"created_at"), "missing created_at column");
}

#[tokio::test]
async fn migration_010_community_lessons_has_expected_columns() {
    let pool = make_memory_pool().await;
    apply_full_chain(&pool).await;

    let columns: Vec<(i64, String, String)> = sqlx::query_as("PRAGMA table_info(community_lessons)")
        .fetch_all(&pool)
        .await
        .unwrap();

    let col_names: Vec<&str> = columns.iter().map(|(_, name, _)| name.as_str()).collect();
    assert!(col_names.contains(&"id"), "missing id column");
    assert!(col_names.contains(&"author_id"), "missing author_id column");
    assert!(col_names.contains(&"title"), "missing title column");
    assert!(col_names.contains(&"description"), "missing description column");
    assert!(col_names.contains(&"content_url"), "missing content_url column");
    assert!(col_names.contains(&"difficulty"), "missing difficulty column");
    assert!(col_names.contains(&"tags"), "missing tags column");
    assert!(col_names.contains(&"likes"), "missing likes column");
}

#[tokio::test]
async fn migration_010_community_riffs_has_expected_columns() {
    let pool = make_memory_pool().await;
    apply_full_chain(&pool).await;

    let columns: Vec<(i64, String, String)> = sqlx::query_as("PRAGMA table_info(community_riffs)")
        .fetch_all(&pool)
        .await
        .unwrap();

    let col_names: Vec<&str> = columns.iter().map(|(_, name, _)| name.as_str()).collect();
    assert!(col_names.contains(&"id"), "missing id column");
    assert!(col_names.contains(&"author_id"), "missing author_id column");
    assert!(col_names.contains(&"title"), "missing title column");
    assert!(col_names.contains(&"tablature"), "missing tablature column");
    assert!(col_names.contains(&"bpm"), "missing bpm column");
    assert!(col_names.contains(&"tuning"), "missing tuning column");
}

#[tokio::test]
async fn migration_010_community_users_accepts_insert() {
    let pool = make_memory_pool().await;
    apply_full_chain(&pool).await;

    sqlx::query(
        "INSERT INTO community_users (id, username, email, password_hash, created_at)
         VALUES ('u1', 'testuser', 'test@example.com', 'hash123', 1700000000)",
    )
    .execute(&pool)
    .await
    .unwrap();

    let username: String = sqlx::query_scalar("SELECT username FROM community_users WHERE id = 'u1'")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(username, "testuser");
}

#[tokio::test]
async fn migration_010_community_lessons_accepts_insert() {
    let pool = make_memory_pool().await;
    apply_full_chain(&pool).await;

    // First create a user (foreign key)
    sqlx::query(
        "INSERT INTO community_users (id, username, email, password_hash, created_at)
         VALUES ('u1', 'teacher', 't@example.com', 'hash', 1700000000)",
    )
    .execute(&pool)
    .await
    .unwrap();

    sqlx::query(
        "INSERT INTO community_lessons (id, author_id, title, description, content_url, difficulty, tags, likes, created_at)
         VALUES ('l1', 'u1', 'Basic Chords', 'Learn G, C, D', 'https://youtube.com/abc', 'beginner', '[]', 0, 1700000000)",
    )
    .execute(&pool)
    .await
    .unwrap();

    let title: String = sqlx::query_scalar("SELECT title FROM community_lessons WHERE id = 'l1'")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(title, "Basic Chords");
}

#[tokio::test]
async fn migration_010_community_follows_accepts_insert() {
    let pool = make_memory_pool().await;
    apply_full_chain(&pool).await;

    // Create two users
    sqlx::query(
        "INSERT INTO community_users (id, username, email, password_hash, created_at)
         VALUES ('u1', 'user1', 'u1@example.com', 'hash', 1700000000)",
    )
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO community_users (id, username, email, password_hash, created_at)
         VALUES ('u2', 'user2', 'u2@example.com', 'hash', 1700000000)",
    )
    .execute(&pool)
    .await
    .unwrap();

    sqlx::query(
        "INSERT INTO community_follows (follower_id, following_id, created_at)
         VALUES ('u1', 'u2', 1700000000)",
    )
    .execute(&pool)
    .await
    .unwrap();

    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM community_follows")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(count, 1);
}

#[tokio::test]
async fn migration_010_community_streaks_accepts_insert() {
    let pool = make_memory_pool().await;
    apply_full_chain(&pool).await;

    sqlx::query(
        "INSERT INTO community_users (id, username, email, password_hash, created_at)
         VALUES ('u1', 'streaker', 's@example.com', 'hash', 1700000000)",
    )
    .execute(&pool)
    .await
    .unwrap();

    sqlx::query(
        "INSERT INTO community_streaks (user_id, current_streak, longest_streak, last_practice_date)
         VALUES ('u1', 5, 12, '2024-01-15')",
    )
    .execute(&pool)
    .await
    .unwrap();

    let streak: i64 = sqlx::query_scalar("SELECT current_streak FROM community_streaks WHERE user_id = 'u1'")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(streak, 5);
}

#[tokio::test]
async fn migration_010_community_cache_accepts_insert() {
    let pool = make_memory_pool().await;
    apply_full_chain(&pool).await;

    sqlx::query(
        "INSERT INTO community_cache (key, value, expires_at)
         VALUES ('feed:page:1', '{\"items\":[]}', 1700003600)",
    )
    .execute(&pool)
    .await
    .unwrap();

    let value: String = sqlx::query_scalar("SELECT value FROM community_cache WHERE key = 'feed:page:1'")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert!(value.contains("items"));
}

#[tokio::test]
async fn migration_011_db_version_reaches_11() {
    let pool = make_memory_pool().await;
    apply_full_chain(&pool).await;

    let version: String = sqlx::query_scalar(
        "SELECT value FROM schema_meta WHERE key = 'db_version'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(version, "11", "db_version should be 11 after full chain including 011");
}

#[tokio::test]
async fn migration_011_soft_delete_adds_columns() {
    let pool = make_memory_pool().await;
    apply_full_chain(&pool).await;

    // Verify columns exist
    let columns: Vec<(i64, String, String)> = sqlx::query_as("PRAGMA table_info(products_meta)")
        .fetch_all(&pool)
        .await
        .unwrap();

    let col_names: Vec<&str> = columns.iter().map(|(_, name, _)| name.as_str()).collect();
    assert!(col_names.contains(&"is_active"), "missing is_active column");
    assert!(col_names.contains(&"delisted_at"), "missing delisted_at column");

    // Verify defaults
    let col_row: Vec<(i64, String, String, String, Option<i64>, Option<i64>)> = sqlx::query_as(
        "PRAGMA table_info(products_meta)",
    )
    .fetch_all(&pool)
    .await
    .unwrap();

    for (_, name, _, dflt, _, _) in &col_row {
        if name == "is_active" {
            assert_eq!(dflt.as_deref(), Some("1"), "is_active should default to 1");
        }
    }
}

#[tokio::test]
async fn migration_011_is_idempotent() {
    let pool = make_memory_pool().await;
    apply_full_chain(&pool).await;

    let version: String = sqlx::query_scalar(
        "SELECT value FROM schema_meta WHERE key = 'db_version'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(version, "11", "db_version should be 11 after full chain");

    // Re-run should not fail
    let dir = temp_dir_with_files(&[
        "001_init.sql", "002_add_url_validation.sql", "003_add_image_cache.sql",
        "004_add_price_source.sql", "005_add_settings.sql", "006_wishlist_schema.sql",
        "007_price_drop_notifications.sql", "008_collection_items.down.sql",
        "008_collection_items.sql", "009_add_recent_searches.sql", "010_community_schema.sql",
        "011_soft_delete.sql",
    ]);
    std::fs::write(dir.join("001_init.sql"), include_str!("../migrations/001_init.sql")).unwrap();
    std::fs::write(dir.join("002_add_url_validation.sql"), include_str!("../migrations/002_add_url_validation.sql")).unwrap();
    std::fs::write(dir.join("003_add_image_cache.sql"), include_str!("../migrations/003_add_image_cache.sql")).unwrap();
    std::fs::write(dir.join("004_add_price_source.sql"), include_str!("../migrations/004_add_price_source.sql")).unwrap();
    std::fs::write(dir.join("005_add_settings.sql"), include_str!("../migrations/005_add_settings.sql")).unwrap();
    std::fs::write(dir.join("006_wishlist_schema.sql"), include_str!("../migrations/006_wishlist_schema.sql")).unwrap();
    std::fs::write(dir.join("007_price_drop_notifications.sql"), include_str!("../migrations/007_price_drop_notifications.sql")).unwrap();
    std::fs::write(dir.join("008_collection_items.sql"), include_str!("../migrations/008_collection_items.sql")).unwrap();
    std::fs::write(dir.join("008_collection_items.down.sql"), include_str!("../migrations/008_collection_items.down.sql")).unwrap();
    std::fs::write(dir.join("009_add_recent_searches.sql"), include_str!("../migrations/009_add_recent_searches.sql")).unwrap();
    std::fs::write(dir.join("010_community_schema.sql"), include_str!("../repository/sqlite/migrations/010_community_schema.sql")).unwrap();
    std::fs::write(dir.join("011_soft_delete.sql"), include_str!("../migrations/011_soft_delete.sql")).unwrap();

    let runner = MigrationRunner::new(pool.clone(), dir);
    runner.run().await.unwrap();

    let version: String = sqlx::query_scalar(
        "SELECT value FROM schema_meta WHERE key = 'db_version'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(version, "11");
}
