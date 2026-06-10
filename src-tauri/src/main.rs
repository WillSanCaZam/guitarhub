// SPDX-License-Identifier: GPL-3.0-or-later

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
        .plugin(tauri_plugin_updater::Builder::new().build())
        .manage(state)
        .invoke_handler(tauri::generate_handler![
            guitarhub_lib::commands::image_command::get_product_image,
            guitarhub_lib::commands::price_command::get_price_history,
            guitarhub_lib::commands::price_command::get_price_insight,
            guitarhub_lib::commands::settings_command::get_setting,
            guitarhub_lib::commands::settings_command::save_setting,
            guitarhub_lib::commands::settings_command::delete_setting,
            guitarhub_lib::commands::settings_command::test_alert_channel,
            guitarhub_lib::commands::export_command::export_data,
            guitarhub_lib::commands::sync_command::sync_catalog,
            guitarhub_lib::commands::search_command::search_products,
            guitarhub_lib::commands::dashboard_command::get_total_products,
            guitarhub_lib::commands::dashboard_command::get_wishlist_count,
            guitarhub_lib::commands::dashboard_command::get_recent_searches,
            guitarhub_lib::commands::dashboard_command::get_categories,
            guitarhub_lib::commands::dashboard_command::record_search,
            guitarhub_lib::commands::collection_command::add_to_collection,
            guitarhub_lib::commands::collection_command::remove_from_collection,
            guitarhub_lib::commands::collection_command::get_collection,
            guitarhub_lib::commands::collection_command::update_collection_item,
            guitarhub_lib::commands::collection_command::get_collection_stats,
            guitarhub_lib::commands::wishlist_command::add_to_wishlist,
            guitarhub_lib::commands::wishlist_command::remove_from_wishlist,
            guitarhub_lib::commands::wishlist_command::get_wishlist,
        ])
        .run(tauri::generate_context!())
        .context("error while running tauri application")?;

    Ok(())
}
