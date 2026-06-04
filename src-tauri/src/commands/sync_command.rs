use crate::services::sync::{JsonFixtureLoader, SyncResult, SyncService};
use crate::AppState;
use tauri::State;

/// Read a JSON catalog fixture file and upsert all products into the database.
#[tauri::command]
pub async fn sync_catalog(
    path: String,
    state: State<'_, AppState>,
) -> Result<SyncResult, crate::AppError> {
    let loader = JsonFixtureLoader::new(state.pool.clone());
    loader.sync_from_json(&path).await
}
