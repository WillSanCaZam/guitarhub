use crate::domain::product::{CatalogFile, RawProduct};
use crate::AppError;
use sqlx::SqlitePool;

/// Trait abstracting catalog synchronization from various sources.
#[async_trait::async_trait]
pub trait SyncService: Send + Sync {
    /// Read a JSON catalog file and upsert all products into the database.
    async fn sync_from_json(&self, path: &str) -> Result<SyncResult, AppError>;
}

/// Result returned after a successful sync operation.
#[derive(Debug, Clone, serde::Serialize)]
pub struct SyncResult {
    pub source_id: String,
    pub product_count: usize,
}

/// A stub `SyncService` that loads a JSON fixture file and upserts products
/// into the `products_meta` table. Intentionally minimal — no delta detection,
/// no incremental sync, no remote fetching.
pub struct JsonFixtureLoader {
    pool: SqlitePool,
}

impl JsonFixtureLoader {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// Insert or replace every product into `products_meta`.
    /// The FTS triggers on `products_meta` automatically keep `products_fts` in sync.
    async fn upsert_products(&self, source_id: &str, products: &[RawProduct]) -> Result<usize, AppError> {
        let mut count = 0;
        let synced_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        for p in products {
            let result = sqlx::query(
                r#"INSERT OR REPLACE INTO products_meta
                   (sku, source_id, name, brand, model, category, subcategory,
                    price, currency, condition, availability, url, image_url,
                    seller, location, synced_at)
                   VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"#,
            )
            .bind(&p.sku)
            .bind(source_id)
            .bind(&p.name)
            .bind(&p.brand)
            .bind(&p.model)
            .bind(&p.category)
            .bind(&p.subcategory)
            .bind(p.price)
            .bind(&p.currency)
            .bind(&p.condition)
            .bind(&p.availability)
            .bind(&p.url)
            .bind(&p.image_url)
            .bind(&p.seller)
            .bind(&p.location)
            .bind(synced_at)
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;
            count += result.rows_affected() as usize;
        }
        Ok(count)
    }
}

#[async_trait::async_trait]
impl SyncService for JsonFixtureLoader {
    async fn sync_from_json(&self, path: &str) -> Result<SyncResult, AppError> {
        let data = tokio::fs::read_to_string(path)
            .await
            .map_err(|e| AppError::InvalidInput(format!("Cannot read file: {e}")))?;
        let catalog: CatalogFile = serde_json::from_str(&data)
            .map_err(|e| AppError::InvalidInput(format!("Invalid JSON: {e}")))?;
        let count = self
            .upsert_products(&catalog.source_id, &catalog.products)
            .await?;
        Ok(SyncResult {
            source_id: catalog.source_id,
            product_count: count,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::SqlitePool;

    /// Create an in-memory pool with the products_meta table (matching 001_init.sql).
    async fn setup_db() -> SqlitePool {
        let pool = SqlitePool::connect("sqlite::memory:")
            .await
            .expect("in-memory pool");

        // Create schema_meta so any partial migration doesn't trip up
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS schema_meta (
                key   TEXT PRIMARY KEY,
                value TEXT NOT NULL
            )",
        )
        .execute(&pool)
        .await
        .unwrap();

        // Create products_meta matching the production schema
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS products_meta (
                sku          TEXT PRIMARY KEY,
                source_id    TEXT NOT NULL,
                name         TEXT NOT NULL DEFAULT '',
                brand        TEXT NOT NULL DEFAULT '',
                model        TEXT NOT NULL DEFAULT '',
                category     TEXT NOT NULL DEFAULT '',
                subcategory  TEXT NOT NULL DEFAULT '',
                specs_json   TEXT NOT NULL DEFAULT '{}',
                price        REAL,
                currency     TEXT,
                condition    TEXT CHECK(condition IN ('new','used','refurbished','unknown')),
                availability TEXT CHECK(availability IN ('in_stock','out_of_stock','unknown')),
                url          TEXT NOT NULL CHECK(url LIKE 'https://%'),
                image_url    TEXT CHECK(image_url = '' OR image_url LIKE 'https://%'),
                seller       TEXT,
                location     TEXT,
                synced_at    INTEGER NOT NULL
            )",
        )
        .execute(&pool)
        .await
        .unwrap();

        pool
    }

    /// Path to the sample_catalog.json fixture relative to the crate root.
    fn fixture_path() -> String {
        let crate_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
        crate_dir
            .join("tests/fixtures/sample_catalog.json")
            .to_str()
            .expect("valid UTF-8 path")
            .to_string()
    }

    #[tokio::test]
    async fn sync_from_json_inserts_products() {
        let pool = setup_db().await;
        let loader = JsonFixtureLoader::new(pool.clone());

        let result = loader.sync_from_json(&fixture_path()).await.expect("sync should succeed");

        assert_eq!(result.source_id, "reverb");
        assert_eq!(result.product_count, 3);

        // Verify products were actually inserted
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM products_meta")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(count, 3);

        // Spot-check one product's data
        let sku: String = sqlx::query_scalar(
            "SELECT sku FROM products_meta WHERE source_id = 'reverb' ORDER BY sku LIMIT 1",
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(sku, "FENDER-STRAT-001");
    }

    #[tokio::test]
    async fn sync_from_json_missing_file_returns_error() {
        let pool = setup_db().await;
        let loader = JsonFixtureLoader::new(pool.clone());

        let result = loader.sync_from_json("/tmp/nonexistent-catalog.json").await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(
            err.to_string().contains("Cannot read file"),
            "Expected file read error, got: {err}"
        );
    }

    #[tokio::test]
    async fn sync_from_json_invalid_json_returns_error() {
        let pool = setup_db().await;
        let loader = JsonFixtureLoader::new(pool.clone());

        // Write invalid JSON to a temp file
        let tmp = tempfile::NamedTempFile::new().unwrap();
        let path = tmp.path().to_str().unwrap().to_string();
        std::fs::write(&path, "this is not json").unwrap();

        let result = loader.sync_from_json(&path).await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(
            err.to_string().contains("Invalid JSON"),
            "Expected Invalid JSON error, got: {err}"
        );
    }
}
