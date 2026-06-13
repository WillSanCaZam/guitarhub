// SPDX-License-Identifier: GPL-3.0-or-later

use tauri::State;
use crate::services::community_service::{CommunityService, Lesson, LessonInput, Comment, CommentInput};
use crate::AppError;
use crate::AppState;

// ── Core logic (extracted for testability without Tauri runtime) ─────────

pub async fn get_feed_cmd(
    pool: &sqlx::SqlitePool,
    limit: u32,
    offset: u32,
) -> Result<Vec<Lesson>, AppError> {
    let svc = CommunityService::new(pool.clone());
    svc.get_feed(limit, offset).await
}

pub async fn create_lesson_cmd(
    pool: &sqlx::SqlitePool,
    input: LessonInput,
) -> Result<String, AppError> {
    let svc = CommunityService::new(pool.clone());
    svc.create_lesson(input).await
}

pub async fn get_lesson_cmd(
    pool: &sqlx::SqlitePool,
    lesson_id: String,
) -> Result<Option<Lesson>, AppError> {
    let svc = CommunityService::new(pool.clone());
    svc.get_lesson(&lesson_id).await
}

pub async fn like_content_cmd(
    pool: &sqlx::SqlitePool,
    lesson_id: String,
) -> Result<(), AppError> {
    let svc = CommunityService::new(pool.clone());
    svc.like_lesson(&lesson_id).await
}

pub async fn add_comment_cmd(
    pool: &sqlx::SqlitePool,
    input: CommentInput,
) -> Result<String, AppError> {
    let svc = CommunityService::new(pool.clone());
    svc.add_comment(input).await
}

pub async fn get_comments_cmd(
    pool: &sqlx::SqlitePool,
    content_type: String,
    content_id: String,
) -> Result<Vec<Comment>, AppError> {
    let svc = CommunityService::new(pool.clone());
    svc.get_comments(&content_type, &content_id).await
}

// ── Health Check ──────────────────────────────────────────────────────────

/// Check if the community server is reachable.
///
/// Sends a GET request to `{server_url}/health` with a 5-second timeout.
/// Returns `Ok(true)` if the server responds with HTTP 200, `Ok(false)` otherwise.
pub async fn health_check_cmd(server_url: &str) -> Result<bool, AppError> {
    let url = format!("{}/health", server_url.trim_end_matches('/'));
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .map_err(|e| AppError::Internal(e.to_string()))?;

    match client.get(&url).send().await {
        Ok(resp) => Ok(resp.status().is_success()),
        Err(_) => Ok(false),
    }
}

#[tauri::command]
pub async fn health_check(
    server_url: String,
    _state: State<'_, AppState>,
) -> Result<bool, AppError> {
    health_check_cmd(&server_url).await
}

// ── Tauri IPC Commands ────────────────────────────────────────────────────

#[tauri::command]
pub async fn get_feed(
    limit: Option<u32>,
    offset: Option<u32>,
    state: State<'_, AppState>,
) -> Result<Vec<Lesson>, AppError> {
    get_feed_cmd(&state.pool, limit.unwrap_or(20), offset.unwrap_or(0)).await
}

#[tauri::command]
pub async fn create_lesson(
    input: LessonInput,
    state: State<'_, AppState>,
) -> Result<String, AppError> {
    create_lesson_cmd(&state.pool, input).await
}

#[tauri::command]
pub async fn get_lesson(
    lesson_id: String,
    state: State<'_, AppState>,
) -> Result<Option<Lesson>, AppError> {
    get_lesson_cmd(&state.pool, lesson_id).await
}

#[tauri::command]
pub async fn like_content(
    lesson_id: String,
    state: State<'_, AppState>,
) -> Result<(), AppError> {
    like_content_cmd(&state.pool, lesson_id).await
}

#[tauri::command]
pub async fn add_comment(
    input: CommentInput,
    state: State<'_, AppState>,
) -> Result<String, AppError> {
    add_comment_cmd(&state.pool, input).await
}

#[tauri::command]
pub async fn get_comments(
    content_type: String,
    content_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<Comment>, AppError> {
    get_comments_cmd(&state.pool, content_type, content_id).await
}

// ── Tests ─────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn health_check_invalid_url_returns_false() {
        // Port 19999 is almost certainly not listening
        let result = health_check_cmd("http://127.0.0.1:19999").await;
        assert!(result.is_ok(), "should not error — unreachable returns Ok(false)");
        assert!(!result.unwrap(), "unreachable server should return false");
    }

    #[tokio::test]
    async fn health_check_trims_trailing_slash() {
        // Should produce URL like "http://127.0.0.1:19999/health" not "http://127.0.0.1:19999//health"
        let result = health_check_cmd("http://127.0.0.1:19999/").await;
        assert!(result.is_ok());
        assert!(!result.unwrap());
    }

    #[tokio::test]
    async fn health_check_with_path_prefix() {
        // URL with path prefix — should append /health correctly
        let result = health_check_cmd("http://127.0.0.1:19999/api/v1").await;
        assert!(result.is_ok());
        assert!(!result.unwrap());
    }
}
