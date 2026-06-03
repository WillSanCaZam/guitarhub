pub mod commands;
pub mod repository;
pub mod services;

use repository::sqlite::migrations::MigrationRunner;
use services::image_cache::ImageCacheService;
use sqlx::sqlite::SqlitePoolOptions;

/// Shared application state injected into Tauri commands.
///
/// Clone is safe: `SqlitePool` and `ImageCacheService` are both
/// internally `Arc`-based.
#[derive(Clone)]
pub struct AppState {
    pub pool: sqlx::SqlitePool,
    pub image_cache_service: ImageCacheService,
}

/// Initialize the database connection, run pending migrations, and
/// return an `AppState` ready for Tauri or direct use.
pub async fn initialize_database(db_path: &str) -> anyhow::Result<AppState> {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect(db_path)
        .await?;

    let migrations_dir = std::env::var("GUITARHUB_MIGRATIONS_DIR")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| {
            let crate_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
            crate_dir.join("src/repository/sqlite/migrations")
        });

    let runner = MigrationRunner::new(pool.clone(), migrations_dir);
    runner.run().await?;

    let image_cache_service = ImageCacheService::new_default(pool.clone());

    Ok(AppState {
        pool,
        image_cache_service,
    })
}
