// SPDX-License-Identifier: GPL-3.0-or-later

use tauri::State;
use crate::services::profile_service::{ProfileService, Profile, StreakData};
use crate::AppError;
use crate::AppState;

// ── Core logic (extracted for testability without Tauri runtime) ─────────

pub async fn get_profile_cmd(
    pool: &sqlx::SqlitePool,
    user_id: String,
) -> Result<Option<Profile>, AppError> {
    let svc = ProfileService::new(pool.clone());
    svc.get_profile(&user_id).await
}

pub async fn update_profile_cmd(
    pool: &sqlx::SqlitePool,
    user_id: String,
    display_name: Option<String>,
    bio: Option<String>,
    avatar_url: Option<String>,
) -> Result<(), AppError> {
    let svc = ProfileService::new(pool.clone());
    svc.update_profile(&user_id, display_name.as_deref(), bio.as_deref(), avatar_url.as_deref()).await
}

pub async fn get_streak_cmd(
    pool: &sqlx::SqlitePool,
    user_id: String,
) -> Result<Option<StreakData>, AppError> {
    let svc = ProfileService::new(pool.clone());
    svc.get_streak(&user_id).await
}

pub async fn add_gear_to_list_cmd(
    pool: &sqlx::SqlitePool,
    user_id: String,
    gear_sku: String,
) -> Result<(), AppError> {
    let svc = ProfileService::new(pool.clone());
    svc.add_gear_to_list(&user_id, &gear_sku).await
}

// ── Tauri IPC Commands ────────────────────────────────────────────────────

#[tauri::command]
pub async fn get_profile(
    user_id: String,
    state: State<'_, AppState>,
) -> Result<Option<Profile>, AppError> {
    get_profile_cmd(&state.pool, user_id).await
}

#[tauri::command]
pub async fn update_profile(
    user_id: String,
    display_name: Option<String>,
    bio: Option<String>,
    avatar_url: Option<String>,
    state: State<'_, AppState>,
) -> Result<(), AppError> {
    update_profile_cmd(&state.pool, user_id, display_name, bio, avatar_url).await
}

#[tauri::command]
pub async fn get_streak(
    user_id: String,
    state: State<'_, AppState>,
) -> Result<Option<StreakData>, AppError> {
    get_streak_cmd(&state.pool, user_id).await
}

#[tauri::command]
pub async fn add_gear_to_list(
    user_id: String,
    gear_sku: String,
    state: State<'_, AppState>,
) -> Result<(), AppError> {
    add_gear_to_list_cmd(&state.pool, user_id, gear_sku).await
}
