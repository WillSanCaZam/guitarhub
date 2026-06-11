// SPDX-License-Identifier: GPL-3.0-or-later

//! Product repository for batch operations on `products_meta`.
//!
//! This module provides a trait-based repository pattern for product persistence,
//! with a focus on efficient batch upserts using SQLite transactions. The batch
//! approach significantly improves sync performance for large catalogs (1000+ products)
//! by wrapping all INSERT OR REPLACE operations in a single transaction.

use crate::domain::product::RawProduct;
use crate::AppError;
use sqlx::SqlitePool;

/// Repository trait for product batch operations.
///
/// Implementations must ensure atomicity: either all products in a batch are
/// successfully upserted, or none are (transaction rollback on error).
#[async_trait::async_trait]
pub trait ProductRepository: Send + Sync {
    /// Insert or replace a batch of products into `products_meta` within a transaction.
    ///
    /// # Arguments
    /// * `source_id` - The catalog source identifier
    /// * `products` - Slice of products to upsert
    /// * `synced_at` - Unix timestamp for the sync operation
    ///
    /// # Returns
    /// The number of rows affected (inserted or replaced).
    ///
    /// # Errors
    /// Returns `AppError::Database` if any product fails to upsert. The entire
    /// batch is rolled back on error to maintain consistency.
    async fn batch_upsert_products(
        &self,
        source_id: &str,
        products: &[RawProduct],
        synced_at: i64,
    ) -> Result<u32, AppError>;
}

/// SQLite implementation of `ProductRepository`.
///
/// Uses a transaction to wrap batch INSERT OR REPLACE operations, ensuring
/// atomicity. If any individual insert fails, the entire batch is rolled back.
#[derive(Debug, Clone)]
pub struct SqliteProductRepository {
    pool: SqlitePool,
}

impl SqliteProductRepository {
    /// Create a new repository backed by the given connection pool.
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl ProductRepository for SqliteProductRepository {
    async fn batch_upsert_products(
        &self,
        source_id: &str,
        products: &[RawProduct],
        synced_at: i64,
    ) -> Result<u32, AppError> {
        // Begin transaction — all inserts are atomic
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| AppError::Database(format!("failed to begin transaction: {e}")))?;

        let mut rows_affected = 0u32;

        for p in products {
            let result = sqlx::query(
                r#"INSERT OR REPLACE INTO products_meta
                   (sku, source_id, name, brand, model, category, subcategory,
                    price, currency, condition, availability, url, image_url,
                    seller, location, synced_at)
                   VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16)"#,
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
            .execute(&mut *tx)
            .await
            .map_err(|e| AppError::Database(format!("batch upsert failed for SKU {}: {e}", p.sku)))?;

            rows_affected += result.rows_affected() as u32;
        }

        // Commit transaction — all inserts succeeded
        tx.commit()
            .await
            .map_err(|e| AppError::Database(format!("failed to commit transaction: {e}")))?;

        Ok(rows_affected)
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn raw_product(sku: &str, price: f64) -> RawProduct {
        RawProduct {
            sku: sku.to_string(),
            name: format!("Test {sku}"),
            brand: "TestBrand".to_string(),
            model: "TM-100".to_string(),
            category: "Electric Guitars".to_string(),
            subcategory: "Solid Body".to_string(),
            price,
            currency: "USD".to_string(),
            condition: "new".to_string(),
            availability: "in_stock".to_string(),
            url: format!("https://example.com/{sku}"),
            image_url: format!("https://example.com/{sku}.jpg"),
            specs_json: "{}".to_string(),
            seller: "Test Seller".to_string(),
            location: "USA".to_string(),
        }
    }

    async fn setup_db() -> SqlitePool {
        let pool = SqlitePool::connect("sqlite::memory:")
            .await
            .expect("in-memory pool");

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

    #[tokio::test]
    async fn batch_upsert_inserts_all_products() {
        let pool = setup_db().await;
        let repo = SqliteProductRepository::new(pool.clone());

        let products: Vec<RawProduct> = (0..50)
            .map(|i| raw_product(&format!("SKU-{:03}", i), 500.0 + i as f64))
            .collect();

        let synced_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        let affected = repo
            .batch_upsert_products("test-source", &products, synced_at)
            .await
            .expect("batch upsert must succeed");

        assert_eq!(affected, 50, "expected 50 rows affected");

        // Verify all 50 rows exist
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM products_meta")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(count, 50, "expected 50 products in DB");
    }

    #[tokio::test]
    async fn batch_upsert_replaces_existing_products() {
        let pool = setup_db().await;
        let repo = SqliteProductRepository::new(pool.clone());

        let synced_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        // First insert
        let products = vec![raw_product("SKU-REPLACE", 1000.0)];
        repo.batch_upsert_products("test-source", &products, synced_at)
            .await
            .unwrap();

        // Verify initial price
        let price: Option<f64> =
            sqlx::query_scalar("SELECT price FROM products_meta WHERE sku = 'SKU-REPLACE'")
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(price, Some(1000.0));

        // Replace with new price
        let updated = vec![raw_product("SKU-REPLACE", 800.0)];
        repo.batch_upsert_products("test-source", &updated, synced_at + 1)
            .await
            .unwrap();

        // Verify price was updated
        let new_price: Option<f64> =
            sqlx::query_scalar("SELECT price FROM products_meta WHERE sku = 'SKU-REPLACE'")
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(new_price, Some(800.0), "price should be replaced");

        // Still only 1 row
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM products_meta")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(count, 1, "should still have 1 row after replace");
    }

    #[tokio::test]
    async fn batch_upsert_empty_slice_succeeds() {
        let pool = setup_db().await;
        let repo = SqliteProductRepository::new(pool.clone());

        let synced_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        let affected = repo
            .batch_upsert_products("test-source", &[], synced_at)
            .await
            .expect("empty batch must succeed");

        assert_eq!(affected, 0, "empty batch should affect 0 rows");
    }

    #[tokio::test]
    async fn batch_upsert_rollback_on_invalid_url() {
        let pool = setup_db().await;
        let repo = SqliteProductRepository::new(pool.clone());

        let synced_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        // First, insert a valid product
        let valid_products = vec![raw_product("SKU-VALID", 500.0)];
        repo.batch_upsert_products("test-source", &valid_products, synced_at)
            .await
            .unwrap();

        // Now try a batch with an invalid URL (violates CHECK constraint)
        let mut invalid = raw_product("SKU-INVALID", 600.0);
        invalid.url = "not-a-url".to_string(); // violates CHECK(url LIKE 'https://%')

        let result = repo
            .batch_upsert_products("test-source", &[valid_products[0].clone(), invalid], synced_at)
            .await;

        assert!(result.is_err(), "batch with invalid URL should fail");

        // The valid product from the FIRST batch should still exist
        // (the failed batch was rolled back)
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM products_meta")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(count, 1, "only the first valid batch should persist");
    }
}
