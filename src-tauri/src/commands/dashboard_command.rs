// SPDX-License-Identifier: GPL-3.0-or-later

use crate::repository::dashboard::DashboardRepo;
use crate::AppError;
use crate::AppState;
use tauri::State;

/// Return the total number of products in the catalog.
#[tauri::command]
pub async fn get_total_products(state: State<'_, AppState>) -> Result<u32, AppError> {
    let repo = DashboardRepo::new(state.pool.clone());
    repo.get_total_products().await
}

/// Return the total number of items in the wishlist.
#[tauri::command]
pub async fn get_wishlist_count(state: State<'_, AppState>) -> Result<u32, AppError> {
    let repo = DashboardRepo::new(state.pool.clone());
    repo.get_wishlist_count().await
}

/// Return recent search queries.
#[tauri::command]
pub async fn get_recent_searches(state: State<'_, AppState>) -> Result<Vec<String>, AppError> {
    let repo = DashboardRepo::new(state.pool.clone());
    repo.get_recent_searches().await
}

/// Return distinct product categories sorted alphabetically.
#[tauri::command]
pub async fn get_categories(state: State<'_, AppState>) -> Result<Vec<String>, AppError> {
    let repo = DashboardRepo::new(state.pool.clone());
    repo.get_categories().await
}

/// Record a search query for recent-searches tracking.
#[tauri::command]
pub async fn record_search(query: String, state: State<'_, AppState>) -> Result<(), AppError> {
    let repo = DashboardRepo::new(state.pool.clone());
    repo.record_search(&query).await
}
