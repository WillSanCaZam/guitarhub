// SPDX-License-Identifier: GPL-3.0-or-later

use tauri::AppHandle;
use tauri_plugin_opener::OpenerExt;

/// Open a URL in the default system browser.
///
/// Uses the Tauri opener plugin to open the URL.
#[tauri::command]
pub async fn open_url(app: AppHandle, url: String) -> Result<(), String> {
    app.opener()
        .open_url(&url, None::<&str>)
        .map_err(|e| format!("Failed to open URL: {}", e))
}
