// SPDX-License-Identifier: GPL-3.0-or-later

use sqlx::SqlitePool;
use crate::AppError;

/// Raw row type for lesson queries — matches the SELECT column order.
type LessonRow = (String, String, String, String, String, String, String, u32, i64);

/// Service for community content operations (lessons, riffs, feed, comments).
pub struct CommunityService {
    pool: SqlitePool,
}

impl CommunityService {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// Create a new lesson.
    pub async fn create_lesson(&self, input: LessonInput) -> Result<String, AppError> {
        let id = format!("lesson_{}", uuid_v4());
        sqlx::query(
            "INSERT INTO community_lessons (id, author_id, title, description, content_url, difficulty, tags, likes, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 0, strftime('%s', 'now'))",
        )
        .bind(&id)
        .bind(&input.author_id)
        .bind(&input.title)
        .bind(&input.description)
        .bind(&input.content_url)
        .bind(&input.difficulty)
        .bind(&input.tags)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
        Ok(id)
    }

    /// Get a lesson by ID.
    pub async fn get_lesson(&self, lesson_id: &str) -> Result<Option<Lesson>, AppError> {
        let result: Option<LessonRow> =
            sqlx::query_as(
                "SELECT id, author_id, title, description, content_url, difficulty, tags, likes, created_at
                 FROM community_lessons WHERE id = ?1",
            )
            .bind(lesson_id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(result.map(|(id, author_id, title, description, content_url, difficulty, tags, likes, created_at)| {
            Lesson { id, author_id, title, description, content_url, difficulty, tags, likes, created_at }
        }))
    }

    /// Get lessons list with optional difficulty filter.
    pub async fn get_lessons(
        &self,
        difficulty: Option<&str>,
        limit: u32,
        offset: u32,
    ) -> Result<Vec<Lesson>, AppError> {
        let rows: Vec<LessonRow> =
            if let Some(diff) = difficulty {
                sqlx::query_as(
                    "SELECT id, author_id, title, description, content_url, difficulty, tags, likes, created_at
                     FROM community_lessons WHERE difficulty = ?1 ORDER BY created_at DESC LIMIT ?2 OFFSET ?3",
                )
                .bind(diff)
                .bind(limit)
                .bind(offset)
                .fetch_all(&self.pool)
                .await
                .map_err(|e| AppError::Database(e.to_string()))?
            } else {
                sqlx::query_as(
                    "SELECT id, author_id, title, description, content_url, difficulty, tags, likes, created_at
                     FROM community_lessons ORDER BY created_at DESC LIMIT ?1 OFFSET ?2",
                )
                .bind(limit)
                .bind(offset)
                .fetch_all(&self.pool)
                .await
                .map_err(|e| AppError::Database(e.to_string()))?
            };

        Ok(rows.into_iter().map(|(id, author_id, title, description, content_url, difficulty, tags, likes, created_at)| {
            Lesson { id, author_id, title, description, content_url, difficulty, tags, likes, created_at }
        }).collect())
    }

    /// Like a lesson (increment likes count).
    pub async fn like_lesson(&self, lesson_id: &str) -> Result<(), AppError> {
        sqlx::query("UPDATE community_lessons SET likes = likes + 1 WHERE id = ?1")
            .bind(lesson_id)
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;
        Ok(())
    }

    /// Add a comment to a lesson or riff.
    pub async fn add_comment(&self, input: CommentInput) -> Result<String, AppError> {
        let id = format!("comment_{}", uuid_v4());
        sqlx::query(
            "INSERT INTO community_comments (id, author_id, content_type, content_id, body, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, strftime('%s', 'now'))",
        )
        .bind(&id)
        .bind(&input.author_id)
        .bind(&input.content_type)
        .bind(&input.content_id)
        .bind(&input.body)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
        Ok(id)
    }

    /// Get comments for a lesson or riff.
    pub async fn get_comments(
        &self,
        content_type: &str,
        content_id: &str,
    ) -> Result<Vec<Comment>, AppError> {
        let rows: Vec<(String, String, String, String, i64)> = sqlx::query_as(
            "SELECT id, author_id, body, content_id, created_at
             FROM community_comments WHERE content_type = ?1 AND content_id = ?2
             ORDER BY created_at ASC",
        )
        .bind(content_type)
        .bind(content_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(rows.into_iter().map(|(id, author_id, body, _, created_at)| {
            Comment { id, author_id, content_type: content_type.to_string(), content_id: content_id.to_string(), body, created_at }
        }).collect())
    }

    /// Get feed items (all lessons, ordered by creation date).
    pub async fn get_feed(&self, limit: u32, offset: u32) -> Result<Vec<Lesson>, AppError> {
        self.get_lessons(None, limit, offset).await
    }
}

/// Input for creating a lesson.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LessonInput {
    pub author_id: String,
    pub title: String,
    pub description: String,
    pub content_url: String,
    pub difficulty: String,
    pub tags: String,
}

/// Lesson entity.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Lesson {
    pub id: String,
    pub author_id: String,
    pub title: String,
    pub description: String,
    pub content_url: String,
    pub difficulty: String,
    pub tags: String,
    pub likes: u32,
    pub created_at: i64,
}

/// Input for adding a comment.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CommentInput {
    pub author_id: String,
    pub content_type: String,
    pub content_id: String,
    pub body: String,
}

/// Comment entity.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Comment {
    pub id: String,
    pub author_id: String,
    pub content_type: String,
    pub content_id: String,
    pub body: String,
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
                id TEXT PRIMARY KEY, username TEXT NOT NULL UNIQUE,
                email TEXT NOT NULL UNIQUE, password_hash TEXT NOT NULL,
                created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
            )",
        ).execute(pool).await.unwrap();
        sqlx::query(
            "CREATE TABLE community_lessons (
                id TEXT PRIMARY KEY, author_id TEXT NOT NULL,
                title TEXT NOT NULL, description TEXT NOT NULL DEFAULT '',
                content_url TEXT NOT NULL DEFAULT '', difficulty TEXT NOT NULL,
                tags TEXT NOT NULL DEFAULT '[]', likes INTEGER NOT NULL DEFAULT 0,
                created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
            )",
        ).execute(pool).await.unwrap();
        sqlx::query(
            "CREATE TABLE community_riffs (
                id TEXT PRIMARY KEY, author_id TEXT NOT NULL,
                title TEXT NOT NULL, tablature TEXT NOT NULL DEFAULT '',
                bpm INTEGER NOT NULL DEFAULT 120, tuning TEXT NOT NULL DEFAULT 'E A D G B E',
                tags TEXT NOT NULL DEFAULT '[]', likes INTEGER NOT NULL DEFAULT 0,
                created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
            )",
        ).execute(pool).await.unwrap();
        sqlx::query(
            "CREATE TABLE community_comments (
                id TEXT PRIMARY KEY, author_id TEXT NOT NULL,
                content_type TEXT NOT NULL, content_id TEXT NOT NULL,
                body TEXT NOT NULL, created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
            )",
        ).execute(pool).await.unwrap();
        // Seed a user
        sqlx::query(
            "INSERT INTO community_users (id, username, email, password_hash, created_at)
             VALUES ('u1', 'teacher', 't@example.com', 'hash', 1700000000)",
        ).execute(pool).await.unwrap();
    }

    #[tokio::test]
    async fn create_lesson_returns_id() {
        let pool = memory_pool().await;
        setup_community_tables(&pool).await;
        let svc = CommunityService::new(pool);

        let id = svc.create_lesson(LessonInput {
            author_id: "u1".to_string(),
            title: "Basic Chords".to_string(),
            description: "Learn G, C, D".to_string(),
            content_url: "https://youtube.com/abc".to_string(),
            difficulty: "beginner".to_string(),
            tags: "[]".to_string(),
        }).await.unwrap();

        assert!(!id.is_empty());
        assert!(id.starts_with("lesson_"));
    }

    #[tokio::test]
    async fn get_lesson_returns_created_lesson() {
        let pool = memory_pool().await;
        setup_community_tables(&pool).await;
        let svc = CommunityService::new(pool);

        let id = svc.create_lesson(LessonInput {
            author_id: "u1".to_string(),
            title: "Barre Chords".to_string(),
            description: "F and Bm".to_string(),
            content_url: "https://youtube.com/def".to_string(),
            difficulty: "intermediate".to_string(),
            tags: "[\"barre\"]".to_string(),
        }).await.unwrap();

        let lesson = svc.get_lesson(&id).await.unwrap();
        assert!(lesson.is_some());
        let lesson = lesson.unwrap();
        assert_eq!(lesson.title, "Barre Chords");
        assert_eq!(lesson.difficulty, "intermediate");
        assert_eq!(lesson.likes, 0);
    }

    #[tokio::test]
    async fn like_lesson_increments_count() {
        let pool = memory_pool().await;
        setup_community_tables(&pool).await;
        let svc = CommunityService::new(pool);

        let id = svc.create_lesson(LessonInput {
            author_id: "u1".to_string(),
            title: "Test".to_string(),
            description: "".to_string(),
            content_url: "".to_string(),
            difficulty: "beginner".to_string(),
            tags: "[]".to_string(),
        }).await.unwrap();

        svc.like_lesson(&id).await.unwrap();
        svc.like_lesson(&id).await.unwrap();

        let lesson = svc.get_lesson(&id).await.unwrap().unwrap();
        assert_eq!(lesson.likes, 2);
    }

    #[tokio::test]
    async fn get_lessons_filters_by_difficulty() {
        let pool = memory_pool().await;
        setup_community_tables(&pool).await;
        let svc = CommunityService::new(pool);

        svc.create_lesson(LessonInput {
            author_id: "u1".to_string(),
            title: "Easy".to_string(), description: "".to_string(),
            content_url: "".to_string(), difficulty: "beginner".to_string(), tags: "[]".to_string(),
        }).await.unwrap();
        svc.create_lesson(LessonInput {
            author_id: "u1".to_string(),
            title: "Hard".to_string(), description: "".to_string(),
            content_url: "".to_string(), difficulty: "advanced".to_string(), tags: "[]".to_string(),
        }).await.unwrap();

        let beginners = svc.get_lessons(Some("beginner"), 10, 0).await.unwrap();
        assert_eq!(beginners.len(), 1);
        assert_eq!(beginners[0].title, "Easy");
    }

    #[tokio::test]
    async fn add_comment_and_get_comments() {
        let pool = memory_pool().await;
        setup_community_tables(&pool).await;
        let svc = CommunityService::new(pool);

        let lesson_id = svc.create_lesson(LessonInput {
            author_id: "u1".to_string(),
            title: "Test".to_string(), description: "".to_string(),
            content_url: "".to_string(), difficulty: "beginner".to_string(), tags: "[]".to_string(),
        }).await.unwrap();

        let comment_id = svc.add_comment(CommentInput {
            author_id: "u1".to_string(),
            content_type: "lesson".to_string(),
            content_id: lesson_id.clone(),
            body: "Great lesson!".to_string(),
        }).await.unwrap();

        assert!(!comment_id.is_empty());

        let comments = svc.get_comments("lesson", &lesson_id).await.unwrap();
        assert_eq!(comments.len(), 1);
        assert_eq!(comments[0].body, "Great lesson!");
    }

    #[tokio::test]
    async fn get_feed_returns_lessons() {
        let pool = memory_pool().await;
        setup_community_tables(&pool).await;
        let svc = CommunityService::new(pool);

        svc.create_lesson(LessonInput {
            author_id: "u1".to_string(),
            title: "L1".to_string(), description: "".to_string(),
            content_url: "".to_string(), difficulty: "beginner".to_string(), tags: "[]".to_string(),
        }).await.unwrap();
        svc.create_lesson(LessonInput {
            author_id: "u1".to_string(),
            title: "L2".to_string(), description: "".to_string(),
            content_url: "".to_string(), difficulty: "beginner".to_string(), tags: "[]".to_string(),
        }).await.unwrap();

        let feed = svc.get_feed(10, 0).await.unwrap();
        assert_eq!(feed.len(), 2);
    }
}
