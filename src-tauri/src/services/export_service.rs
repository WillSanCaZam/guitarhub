use serde::Serialize;
use sqlx::SqlitePool;
use std::io::{Cursor, Write};
use zip::write::SimpleFileOptions;
use zip::ZipWriter;


/// Service for exporting database contents as a ZIP archive.
#[derive(Debug, Clone)]
pub struct ExportService {
    pool: SqlitePool,
}

/// Result returned after a successful export.
#[derive(Debug, Clone, Serialize)]
pub struct ExportResult {
    pub success: bool,
    pub size_bytes: u64,
    pub file_count: u32,
}

/// Errors that can occur during export.
#[derive(Debug, thiserror::Error)]
pub enum ExportError {
    #[error("write_error: {0}")]
    Write(String),
    #[error("query_error: {0}")]
    Query(String),
}

impl ExportService {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// Collect data from all relevant tables, build a ZIP in memory,
    /// and write it to the given file path.
    pub async fn export_to(&self, path: &str) -> Result<ExportResult, ExportError> {
        let mut buf = Cursor::new(Vec::new());
        let mut zip = ZipWriter::new(&mut buf);

        // ── wishlist.json ───────────────────────────────────────────────
        let wishlist: Vec<WishlistRow> =
            sqlx::query_as::<_, WishlistRow>("SELECT * FROM wishlist")
                .fetch_all(&self.pool)
                .await
                .map_err(|e| ExportError::Query(e.to_string()))?;

        let wishlist_json =
            serde_json::to_string_pretty(&wishlist).map_err(|e| ExportError::Write(e.to_string()))?;

        zip.start_file("wishlist.json", SimpleFileOptions::default())
            .map_err(|e| ExportError::Write(e.to_string()))?;
        zip.write_all(wishlist_json.as_bytes())
            .map_err(|e| ExportError::Write(e.to_string()))?;

        // ── price_history.json ──────────────────────────────────────────
        let history: Vec<PriceHistoryRow> =
            sqlx::query_as::<_, PriceHistoryRow>("SELECT * FROM price_history")
                .fetch_all(&self.pool)
                .await
                .map_err(|e| ExportError::Query(e.to_string()))?;

        let history_json =
            serde_json::to_string_pretty(&history).map_err(|e| ExportError::Write(e.to_string()))?;

        zip.start_file("price_history.json", SimpleFileOptions::default())
            .map_err(|e| ExportError::Write(e.to_string()))?;
        zip.write_all(history_json.as_bytes())
            .map_err(|e| ExportError::Write(e.to_string()))?;

        // ── settings.json ───────────────────────────────────────────────
        let settings: Vec<SettingRow> =
            sqlx::query_as::<_, SettingRow>("SELECT * FROM settings")
                .fetch_all(&self.pool)
                .await
                .map_err(|e| ExportError::Query(e.to_string()))?;

        // Convert to a flat JSON object { key: value, ... }
        let settings_map: std::collections::BTreeMap<String, String> = settings
            .into_iter()
            .map(|s| (s.key, s.value))
            .collect();
        let settings_json =
            serde_json::to_string_pretty(&settings_map).map_err(|e| ExportError::Write(e.to_string()))?;

        zip.start_file("settings.json", SimpleFileOptions::default())
            .map_err(|e| ExportError::Write(e.to_string()))?;
        zip.write_all(settings_json.as_bytes())
            .map_err(|e| ExportError::Write(e.to_string()))?;

        // ── Finalize ────────────────────────────────────────────────────
        let file_count: u32 = 3;
        zip.finish()
            .map_err(|e| ExportError::Write(e.to_string()))?;

        let size_bytes = buf.position();

        // Write to disk
        std::fs::write(path, buf.into_inner())
            .map_err(|e| ExportError::Write(format!("{e}")))?;

        Ok(ExportResult {
            success: true,
            size_bytes,
            file_count,
        })
    }
}

// ── Row types for sqlx deserialization ─────────────────────────────────────

#[derive(Debug, sqlx::FromRow, serde::Serialize)]
struct WishlistRow {
    #[allow(dead_code)]
    id: Option<i64>,
    #[allow(dead_code)]
    sku: Option<String>,
    #[allow(dead_code)]
    name: Option<String>,
    #[allow(dead_code)]
    brand: Option<String>,
    #[allow(dead_code)]
    price: Option<f64>,
    #[allow(dead_code)]
    currency: Option<String>,
    #[allow(dead_code)]
    image_url: Option<String>,
    #[allow(dead_code)]
    product_url: Option<String>,
    #[allow(dead_code)]
    notes: Option<String>,
    #[allow(dead_code)]
    added_at: Option<i64>,
}

#[derive(Debug, sqlx::FromRow, serde::Serialize)]
struct PriceHistoryRow {
    #[allow(dead_code)]
    id: Option<i64>,
    #[allow(dead_code)]
    sku: Option<String>,
    #[allow(dead_code)]
    price: Option<f64>,
    #[allow(dead_code)]
    recorded_at: Option<i64>,
    #[allow(dead_code)]
    source_id: Option<String>,
}

#[derive(Debug, sqlx::FromRow, serde::Serialize)]
struct SettingRow {
    #[allow(dead_code)]
    key: String,
    #[allow(dead_code)]
    value: String,
}

// ── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repository::sqlite::migrations::MigrationRunner;
    use std::io::Read as _;

    /// Create an in-memory pool using the REAL migration chain (001→006).
    ///
    /// This is the source of truth for the export service: it ensures tests
    /// validate against the actual schema the app uses at runtime, so a
    /// future migration that drops or renames a column will be caught here.
    async fn migrated_pool() -> SqlitePool {
        use std::path::PathBuf;

        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();

        // Materialise the real migration files into a temp dir for the runner.
        let dir = std::env::temp_dir().join(format!(
            "guitarhub-export-mig-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join("001_init.sql"),
            include_str!("../repository/sqlite/migrations/001_init.sql"),
        )
        .unwrap();
        std::fs::write(
            dir.join("002_add_url_validation.sql"),
            include_str!("../repository/sqlite/migrations/002_add_url_validation.sql"),
        )
        .unwrap();
        std::fs::write(
            dir.join("003_add_image_cache.sql"),
            include_str!("../repository/sqlite/migrations/003_add_image_cache.sql"),
        )
        .unwrap();
        std::fs::write(
            dir.join("004_add_price_source.sql"),
            include_str!("../repository/sqlite/migrations/004_add_price_source.sql"),
        )
        .unwrap();
        std::fs::write(
            dir.join("005_add_settings.sql"),
            include_str!("../repository/sqlite/migrations/005_add_settings.sql"),
        )
        .unwrap();
        std::fs::write(
            dir.join("006_wishlist_schema.sql"),
            include_str!("../repository/sqlite/migrations/006_wishlist_schema.sql"),
        )
        .unwrap();

        let runner = MigrationRunner::new(pool.clone(), PathBuf::from(&dir));
        runner.run().await.expect("real migration chain should apply cleanly");
        pool
    }

    /// Create an in-memory pool with all tables needed for export.
    ///
    /// Uses the REAL migration chain (001→006) so tests validate against the
    /// same schema the app uses at runtime. This catches future migrations
    /// that drop or rename columns the export service depends on.
    async fn test_pool() -> SqlitePool {
        migrated_pool().await
    }

    /// Seed the pool with sample data for testing.
    async fn seed_data(pool: &SqlitePool) {
        // Wishlist items
        sqlx::query(
            "INSERT INTO wishlist (sku, name, brand, price, currency) VALUES (?1, ?2, ?3, ?4, ?5)",
        )
        .bind("STRAT_001")
        .bind("Stratocaster")
        .bind("Fender")
        .bind(1299.99f64)
        .bind("USD")
        .execute(pool)
        .await
        .unwrap();

        sqlx::query(
            "INSERT INTO wishlist (sku, name, brand, price, currency) VALUES (?1, ?2, ?3, ?4, ?5)",
        )
        .bind("LESP_001")
        .bind("Les Paul")
        .bind("Gibson")
        .bind(2499.99f64)
        .bind("USD")
        .execute(pool)
        .await
        .unwrap();

        // Price history
        sqlx::query(
            "INSERT INTO price_history (sku, price, recorded_at, source_id) VALUES (?1, ?2, ?3, ?4)",
        )
        .bind("STRAT_001")
        .bind(1299.99f64)
        .bind(1_717_200_000i64)
        .bind("reverb")
        .execute(pool)
        .await
        .unwrap();

        sqlx::query(
            "INSERT INTO price_history (sku, price, recorded_at, source_id) VALUES (?1, ?2, ?3, ?4)",
        )
        .bind("STRAT_001")
        .bind(1199.99f64)
        .bind(1_717_286_400i64)
        .bind("reverb")
        .execute(pool)
        .await
        .unwrap();

        // Settings
        sqlx::query("INSERT INTO settings (key, value) VALUES (?1, ?2)")
            .bind("alert_channel")
            .bind("ntfy")
            .execute(pool)
            .await
            .unwrap();

        sqlx::query("INSERT INTO settings (key, value) VALUES (?1, ?2)")
            .bind("theme")
            .bind("dark")
            .execute(pool)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn export_produces_valid_zip_with_data() {
        let pool = test_pool().await;
        seed_data(&pool).await;
        let svc = ExportService::new(pool);

        let tmp = tempfile::NamedTempFile::new().unwrap();
        let path = tmp.path().to_str().unwrap().to_string();

        let result = svc.export_to(&path).await.unwrap();
        assert!(result.success);
        assert!(result.size_bytes > 0, "ZIP should have content");
        assert_eq!(result.file_count, 3);

        // Read back and verify ZIP structure
        let zip_bytes = std::fs::read(&path).unwrap();
        let reader = std::io::Cursor::new(zip_bytes);
        let mut archive = zip::ZipArchive::new(reader).unwrap();

        assert_eq!(archive.len(), 3, "ZIP should have 3 files");

        // Check each file exists
        let mut found_files: Vec<String> = (0..archive.len())
            .map(|i| archive.by_index(i).unwrap().name().to_string())
            .collect();
        found_files.sort();

        assert_eq!(found_files[0], "price_history.json");
        assert_eq!(found_files[1], "settings.json");
        assert_eq!(found_files[2], "wishlist.json");

        // Verify wishlist.json is valid JSON with 2 items
        {
            let mut file = archive.by_name("wishlist.json").unwrap();
            let mut content = String::new();
            file.read_to_string(&mut content).unwrap();
            let items: Vec<serde_json::Value> =
                serde_json::from_str(&content).expect("valid JSON");
            assert_eq!(items.len(), 2, "expected 2 wishlist items");
        }

        // Verify price_history.json has 2 records
        {
            let mut file = archive.by_name("price_history.json").unwrap();
            let mut content = String::new();
            file.read_to_string(&mut content).unwrap();
            let items: Vec<serde_json::Value> =
                serde_json::from_str(&content).expect("valid JSON");
            assert_eq!(items.len(), 2, "expected 2 price history records");
        }

        // Verify settings.json has 2 entries
        {
            let mut file = archive.by_name("settings.json").unwrap();
            let mut content = String::new();
            file.read_to_string(&mut content).unwrap();
            let map: std::collections::BTreeMap<String, String> =
                serde_json::from_str(&content).expect("valid JSON");
            assert_eq!(map.len(), 2, "expected 2 settings");
            assert_eq!(map.get("alert_channel").unwrap(), "ntfy");
            assert_eq!(map.get("theme").unwrap(), "dark");
        }
    }

    #[tokio::test]
    async fn export_with_empty_tables_produces_valid_zip() {
        let pool = test_pool().await;
        let svc = ExportService::new(pool);

        let tmp = tempfile::NamedTempFile::new().unwrap();
        let path = tmp.path().to_str().unwrap().to_string();

        let result = svc.export_to(&path).await.unwrap();
        assert!(result.success);
        assert!(result.size_bytes > 0);
        assert_eq!(result.file_count, 3);

        // Verify all files contain empty arrays/objects
        let zip_bytes = std::fs::read(&path).unwrap();
        let reader = std::io::Cursor::new(zip_bytes);
        let mut archive = zip::ZipArchive::new(reader).unwrap();

        {
            let mut file = archive.by_name("wishlist.json").unwrap();
            let mut content = String::new();
            file.read_to_string(&mut content).unwrap();
            let items: Vec<serde_json::Value> =
                serde_json::from_str(&content).expect("valid JSON");
            assert!(items.is_empty(), "wishlist should be empty");
        }

        {
            let mut file = archive.by_name("price_history.json").unwrap();
            let mut content = String::new();
            file.read_to_string(&mut content).unwrap();
            let items: Vec<serde_json::Value> =
                serde_json::from_str(&content).expect("valid JSON");
            assert!(items.is_empty(), "price_history should be empty");
        }

        {
            let mut file = archive.by_name("settings.json").unwrap();
            let mut content = String::new();
            file.read_to_string(&mut content).unwrap();
            let map: std::collections::BTreeMap<String, String> =
                serde_json::from_str(&content).expect("valid JSON");
            assert!(map.is_empty(), "settings should be empty");
        }
    }

    #[tokio::test]
    async fn export_fails_on_unwritable_path() {
        let pool = test_pool().await;
        let svc = ExportService::new(pool);

        let result = svc.export_to("/nonexistent-directory/export.zip").await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        match err {
            ExportError::Write(_) => {} // expected
            other => panic!("expected Write error, got: {other}"),
        }
    }

    #[tokio::test]
    async fn export_writes_to_temp_path() {
        let pool = test_pool().await;
        seed_data(&pool).await;
        let svc = ExportService::new(pool);

        let tmp = tempfile::NamedTempFile::new().unwrap();
        let path = tmp.path().to_str().unwrap().to_string();

        let result = svc.export_to(&path).await.unwrap();
        assert!(result.success);
        assert!(result.size_bytes > 0);
        assert_eq!(result.file_count, 3);

        // Verify file exists on disk and is non-empty
        let metadata = std::fs::metadata(&path).unwrap();
        assert!(metadata.len() > 0, "file should exist and be non-empty");
    }

    /// RED: Validate that the export service works against the REAL migration
    /// schema (001→006), not an inline CREATE TABLE. Catches drift between
    /// migrations and service expectations.
    #[tokio::test]
    async fn export_works_against_real_migration_chain() {
        let pool = migrated_pool().await;
        seed_data(&pool).await;
        let svc = ExportService::new(pool);

        let tmp = tempfile::NamedTempFile::new().unwrap();
        let path = tmp.path().to_str().unwrap().to_string();

        let result = svc.export_to(&path).await.unwrap();
        assert!(result.success, "export must succeed against real schema");
        assert!(result.size_bytes > 0, "ZIP should have content");
        assert_eq!(result.file_count, 3);

        // Read back the ZIP and inspect the wishlist JSON — it should
        // reflect the real 10-column schema (id, sku, name, brand, price,
        // currency, image_url, product_url, notes, added_at).
        let zip_bytes = std::fs::read(&path).unwrap();
        let reader = std::io::Cursor::new(zip_bytes);
        let mut archive = zip::ZipArchive::new(reader).unwrap();

        let mut file = archive.by_name("wishlist.json").unwrap();
        let mut content = String::new();
        file.read_to_string(&mut content).unwrap();
        let items: Vec<serde_json::Value> =
            serde_json::from_str(&content).expect("valid JSON");

        assert_eq!(items.len(), 2, "expected 2 wishlist items");

        // The first item should expose the 10 real-schema column names
        let first = &items[0];
        for col in &[
            "id", "sku", "name", "brand", "price", "currency",
            "image_url", "product_url", "notes", "added_at",
        ] {
            assert!(
                first.get(col).is_some(),
                "wishlist JSON missing real-schema column '{col}', got: {first}"
            );
        }
    }
}
