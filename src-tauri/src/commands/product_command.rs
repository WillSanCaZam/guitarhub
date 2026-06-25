// SPDX-License-Identifier: GPL-3.0-or-later

use crate::domain::product::RawProduct;
use crate::AppError;
use crate::AppState;
use tauri::State;

/// Return N random active products for the home page discovery feed.
///
/// Defaults to 6 products when no limit is specified.
/// Returns an empty array when no active products exist (not an error).
#[tauri::command]
pub async fn get_featured_products(
    state: State<'_, AppState>,
    limit: Option<u32>,
) -> Result<Vec<RawProduct>, AppError> {
    state.product_query.get_featured(limit.unwrap_or(6)).await
}

/// Return active products with the largest absolute price drops.
///
/// Uses a correlated-subquery join with `price_history` to compute
/// `first_recorded_price - last_recorded_price`. Defaults to 6 products.
/// Only products whose current price is strictly less than the first
/// recorded price are included.
#[tauri::command]
pub async fn get_price_drops(
    state: State<'_, AppState>,
    limit: Option<u32>,
) -> Result<Vec<RawProduct>, AppError> {
    state.product_query.get_price_drops(limit.unwrap_or(6)).await
}

/// Return the most recently synced active products.
///
/// Ordered by `synced_at DESC`. Defaults to 6 products.
#[tauri::command]
pub async fn get_new_arrivals(
    state: State<'_, AppState>,
    limit: Option<u32>,
) -> Result<Vec<RawProduct>, AppError> {
    state.product_query.get_new_arrivals(limit.unwrap_or(6)).await
}

/// Return a single product by SKU (case-insensitive).
///
/// Returns `AppError::NotFound` if no active product matches the SKU.
/// Returns `AppError::InvalidInput` if the SKU is empty.
#[tauri::command]
pub async fn get_product_detail(
    state: State<'_, AppState>,
    sku: String,
) -> Result<RawProduct, AppError> {
    state.product_query.get_by_sku(&sku).await
}
