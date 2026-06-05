// SPDX-License-Identifier: GPL-3.0-or-later

use crate::domain::product::{SearchFilters, SearchResult, SortOrder};
use crate::services::search::FtsSearchService;
use crate::AppState;
use tauri::State;

/// Execute a full-text search across the product catalog.
///
/// Sanitizes the query for FTS5, applies optional filters (category, price
/// range, source), sorts by the given `SortOrder`, and paginates results.
#[tauri::command]
pub async fn search_products(
    query: String,
    filters: SearchFilters,
    sort: SortOrder,
    page: u32,
    page_size: u32,
    state: State<'_, AppState>,
) -> Result<SearchResult, crate::AppError> {
    let service = FtsSearchService::new(state.pool.clone());
    service.search(&query, &filters, sort, page, page_size).await
}
