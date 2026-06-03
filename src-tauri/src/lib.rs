pub mod repository;

use repository::sqlite::migrations::MigrationRunner;
use sqlx::sqlite::SqlitePoolOptions;

/// Initialize the database connection and run pending migrations.
/// Called during app startup to ensure the local SQLite schema is up-to-date.
pub async fn initialize_database(db_path: &str) -> anyhow::Result<sqlx::SqlitePool> {
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

    Ok(pool)
}
