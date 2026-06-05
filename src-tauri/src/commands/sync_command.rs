// SPDX-License-Identifier: GPL-3.0-or-later
use crate::services::sync::{CatalogSyncService, SyncResult, SyncService};
use crate::AppState;
use tauri::State;

/// Fetch a remote catalog JSON and upsert all products into the database.
#[tauri::command]
pub async fn sync_catalog(
    url: String,
    state: State<'_, AppState>,
) -> Result<SyncResult, crate::AppError> {
    let service = CatalogSyncService::new(state.pool.clone(), state.http_client.clone());
    service.sync_catalog(&url).await
}
