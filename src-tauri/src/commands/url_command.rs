// SPDX-License-Identifier: GPL-3.0-or-later

use tauri::AppHandle;
use tauri_plugin_shell::ShellExt;

/// Open a URL in the default system browser.
///
/// Uses the Tauri shell plugin to open the URL.
#[tauri::command]
pub async fn open_url(app: AppHandle, url: String) -> Result<(), String> {
    app.shell()
        .open(&url, None)
        .map_err(|e| format!("Failed to open URL: {}", e))
}