use anyhow::Context;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let db_path = std::env::var("GUITARHUB_DB_PATH")
        .unwrap_or_else(|_| "guitarhub.db".to_string());

    let state = guitarhub_lib::initialize_database(&db_path)
        .await
        .context("Failed to initialize database on startup")?;

    tracing::info!("GuitarHub database initialized successfully");

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_notification::init())
        .manage(state)
        .invoke_handler(tauri::generate_handler![
            guitarhub_lib::commands::image_command::get_product_image,
            guitarhub_lib::commands::price_command::get_price_history,
            guitarhub_lib::commands::price_command::get_price_insight,
            guitarhub_lib::commands::settings_command::get_setting,
            guitarhub_lib::commands::settings_command::save_setting,
            guitarhub_lib::commands::settings_command::test_alert_channel,
            guitarhub_lib::commands::export_command::export_data,
            guitarhub_lib::commands::sync_command::sync_catalog,
        ])
        .run(tauri::generate_context!())
        .context("error while running tauri application")?;

    Ok(())
}
