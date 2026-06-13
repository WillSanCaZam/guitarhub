// SPDX-License-Identifier: GPL-3.0-or-later

use tauri::State;
use crate::services::auth_service::{AuthService, CommunityUser};
use crate::AppError;
use crate::AppState;

// ── Core logic (extracted for testability without Tauri runtime) ─────────

pub async fn register_cmd(
    pool: &sqlx::SqlitePool,
    username: String,
    email: String,
    password_hash: String,
) -> Result<String, AppError> {
    let svc = AuthService::new(pool.clone());
    svc.register(&username, &email, &password_hash).await
}

pub async fn login_cmd(
    pool: &sqlx::SqlitePool,
    email: String,
    password_hash: String,
) -> Result<Option<String>, AppError> {
    let svc = AuthService::new(pool.clone());
    svc.login(&email, &password_hash).await
}

pub async fn get_current_user_cmd(
    pool: &sqlx::SqlitePool,
    user_id: String,
) -> Result<Option<CommunityUser>, AppError> {
    let svc = AuthService::new(pool.clone());
    svc.get_user(&user_id).await
}

pub async fn logout_cmd() -> Result<(), AppError> {
    // Token invalidation is handled client-side via secure store deletion.
    // Server-side session management (if any) would go here.
    Ok(())
}

// ── Tauri IPC Commands ────────────────────────────────────────────────────

#[tauri::command]
pub async fn register(
    username: String,
    email: String,
    password_hash: String,
    state: State<'_, AppState>,
) -> Result<String, AppError> {
    register_cmd(&state.pool, username, email, password_hash).await
}

#[tauri::command]
pub async fn login(
    email: String,
    password_hash: String,
    state: State<'_, AppState>,
) -> Result<Option<String>, AppError> {
    login_cmd(&state.pool, email, password_hash).await
}

#[tauri::command]
pub async fn get_current_user(
    user_id: String,
    state: State<'_, AppState>,
) -> Result<Option<CommunityUser>, AppError> {
    get_current_user_cmd(&state.pool, user_id).await
}

#[tauri::command]
pub async fn logout() -> Result<(), AppError> {
    logout_cmd().await
}

#[tauri::command]
pub async fn refresh_token() -> Result<(), AppError> {
    // Token refresh logic — implementation depends on OAuth provider.
    // Placeholder for now.
    Ok(())
}
