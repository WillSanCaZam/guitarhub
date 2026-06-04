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
    use std::io::Read as _;

    /// Create an in-memory pool with all tables needed for export.
    async fn test_pool() -> SqlitePool {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();

        // Create wishlist table
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS wishlist (
                id          INTEGER PRIMARY KEY AUTOINCREMENT,
                sku         TEXT,
                name        TEXT,
                brand       TEXT,
                price       REAL,
                currency    TEXT,
                image_url   TEXT,
                product_url TEXT,
                notes       TEXT,
                added_at    INTEGER
            )",
        )
        .execute(&pool)
        .await
        .unwrap();

        // Create price_history table
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS price_history (
                id          INTEGER PRIMARY KEY AUTOINCREMENT,
                sku         TEXT NOT NULL,
                price       REAL NOT NULL,
                recorded_at INTEGER NOT NULL,
                source_id   TEXT NOT NULL DEFAULT ''
            )",
        )
        .execute(&pool)
        .await
        .unwrap();

        // Create settings table
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS settings (
                key   TEXT PRIMARY KEY,
                value TEXT NOT NULL
            )",
        )
        .execute(&pool)
        .await
        .unwrap();

        pool
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
}
