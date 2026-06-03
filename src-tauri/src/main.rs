use anyhow::Context;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let db_path = std::env::var("GUITARHUB_DB_PATH")
        .unwrap_or_else(|_| "guitarhub.db".to_string());

    let _state = guitarhub_lib::initialize_database(&db_path)
        .await
        .context("Failed to initialize database on startup")?;

    tracing::info!("GuitarHub database initialized successfully");

    // Tauri app setup will be wired here in later phases.
    // For now, the binary demonstrates that migrations run on startup.

    Ok(())
}
