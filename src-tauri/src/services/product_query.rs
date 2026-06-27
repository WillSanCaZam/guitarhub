// SPDX-License-Identifier: GPL-3.0-or-later

//! Product query service — provides discovery and detail queries against
//! `products_meta` (+ `price_history` for drops).
//!
//! Each method returns `RawProduct` values mapped from a private row struct
//! (mirroring the pattern established in `search.rs`).

use crate::domain::product::RawProduct;
use crate::AppError;
use sqlx::SqlitePool;

/// Internal row type for sqlx query_as deserialization from products_meta.
#[derive(Debug, sqlx::FromRow)]
#[allow(dead_code)]
struct ProductQueryRow {
    sku: String,
    source_id: String,
    name: String,
    brand: String,
    model: String,
    category: String,
    subcategory: String,
    price: f64,
    currency: String,
    condition: String,
    availability: String,
    url: String,
    image_url: String,
    specs_json: String,
    seller: String,
    location: String,
    synced_at: i64,
    user_id: Option<String>,
}

/// Convert a `ProductQueryRow` into a `RawProduct` for IPC serialisation.
fn row_to_product(row: ProductQueryRow) -> RawProduct {
    RawProduct {
        sku: row.sku,
        source_id: row.source_id,
        name: row.name,
        brand: row.brand,
        model: row.model,
        category: row.category,
        subcategory: row.subcategory,
        price: row.price,
        currency: row.currency,
        condition: row.condition,
        availability: row.availability,
        url: row.url,
        image_url: row.image_url,
        specs_json: row.specs_json,
        seller: row.seller,
        location: row.location,
        user_id: row.user_id,
    }
}

/// Service for querying active products via four read methods.
///
/// All discovery queries filter `is_active = 1` and return `Vec<RawProduct>`.
/// `get_by_sku` returns `AppError::NotFound` for missing/inactive products.
#[derive(Clone)]
pub struct ProductQueryService {
    pool: SqlitePool,
}

impl ProductQueryService {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// Return N random active products.
    ///
    /// When `user_id` is `Some(id)`, includes both public products (`user_id IS NULL`)
    /// and products belonging to that connection. When `None`, returns only public products.
    pub async fn get_featured(&self, limit: u32, user_id: Option<String>) -> Result<Vec<RawProduct>, AppError> {
        let limit = limit as i64;
        let mut sql = String::from(
            "SELECT sku, source_id, name, brand, model, category, subcategory, \
                    price, currency, condition, availability, url, image_url, \
                    specs_json, seller, location, synced_at, user_id \
             FROM products_meta \
             WHERE is_active = 1",
        );
        if user_id.is_some() {
            sql.push_str(" AND (user_id IS NULL OR user_id = ?)");
        } else {
            sql.push_str(" AND user_id IS NULL");
        }
        sql.push_str(" ORDER BY RANDOM() LIMIT ?");

        let mut query = sqlx::query_as::<_, ProductQueryRow>(sqlx::AssertSqlSafe(sql.as_str()));
        if let Some(ref uid) = user_id {
            query = query.bind(uid);
        }
        query = query.bind(limit);
        let rows = query.fetch_all(&self.pool).await?;

        Ok(rows.into_iter().map(row_to_product).collect())
    }

    /// Return active products with the largest absolute price drops.
    ///
    /// Uses a correlated-subquery approach: for each unique SKU in
    /// `price_history`, computes first and last recorded prices, then
    /// filters to only those where `last_price < first_price`.
    /// When `user_id` is `Some(id)`, includes user products alongside public ones.
    pub async fn get_price_drops(&self, limit: u32, user_id: Option<String>) -> Result<Vec<RawProduct>, AppError> {
        let limit = limit as i64;
        let mut sql = String::from(
            "SELECT m.sku, m.source_id, m.name, m.brand, m.model, m.category, \
                    m.subcategory, m.price, m.currency, m.condition, m.availability, \
                    m.url, m.image_url, m.specs_json, m.seller, m.location, m.synced_at, m.user_id \
             FROM products_meta m \
             JOIN ( \
                 SELECT ph.sku, \
                        (SELECT price FROM price_history \
                         WHERE sku = ph.sku ORDER BY recorded_at ASC LIMIT 1) AS first_price, \
                        (SELECT price FROM price_history \
                         WHERE sku = ph.sku ORDER BY recorded_at DESC LIMIT 1) AS last_price \
                 FROM price_history ph \
                 GROUP BY ph.sku \
                 HAVING last_price < first_price \
             ) drops ON m.sku = drops.sku \
             WHERE m.is_active = 1",
        );
        if user_id.is_some() {
            sql.push_str(" AND (m.user_id IS NULL OR m.user_id = ?)");
        } else {
            sql.push_str(" AND m.user_id IS NULL");
        }
        sql.push_str(" ORDER BY (drops.first_price - drops.last_price) DESC LIMIT ?");

        let mut query = sqlx::query_as::<_, ProductQueryRow>(sqlx::AssertSqlSafe(sql.as_str()));
        if let Some(ref uid) = user_id {
            query = query.bind(uid);
        }
        query = query.bind(limit);
        let rows = query.fetch_all(&self.pool).await?;

        Ok(rows.into_iter().map(row_to_product).collect())
    }

    /// Return active products ordered by most recently synced.
    ///
    /// When `user_id` is `Some(id)`, includes user products alongside public ones.
    pub async fn get_new_arrivals(&self, limit: u32, user_id: Option<String>) -> Result<Vec<RawProduct>, AppError> {
        let limit = limit as i64;
        let mut sql = String::from(
            "SELECT sku, source_id, name, brand, model, category, subcategory, \
                    price, currency, condition, availability, url, image_url, \
                    specs_json, seller, location, synced_at, user_id \
             FROM products_meta \
             WHERE is_active = 1",
        );
        if user_id.is_some() {
            sql.push_str(" AND (user_id IS NULL OR user_id = ?)");
        } else {
            sql.push_str(" AND user_id IS NULL");
        }
        sql.push_str(" ORDER BY synced_at DESC LIMIT ?");

        let mut query = sqlx::query_as::<_, ProductQueryRow>(sqlx::AssertSqlSafe(sql.as_str()));
        if let Some(ref uid) = user_id {
            query = query.bind(uid);
        }
        query = query.bind(limit);
        let rows = query.fetch_all(&self.pool).await?;

        Ok(rows.into_iter().map(row_to_product).collect())
    }

    /// Return a single active product by SKU (case-insensitive).
    ///
    /// Returns `AppError::NotFound` if no active product matches.
    /// Returns `AppError::InvalidInput` if `sku` is empty.
    pub async fn get_by_sku(&self, sku: &str) -> Result<RawProduct, AppError> {
        if sku.is_empty() {
            return Err(AppError::InvalidInput("sku is required".into()));
        }

        let row = sqlx::query_as::<_, ProductQueryRow>(
            "SELECT sku, source_id, name, brand, model, category, subcategory, \
                    price, currency, condition, availability, url, image_url, \
                    specs_json, seller, location, synced_at, user_id \
             FROM products_meta \
             WHERE LOWER(sku) = LOWER(?) \
               AND is_active = 1",
        )
        .bind(sku)
        .fetch_optional(&self.pool)
        .await?
        .ok_or(AppError::NotFound)?;

        Ok(row_to_product(row))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::SqlitePool;

    // ── Helpers ─────────────────────────────────────────────────────────

    /// Create an in-memory pool with products_meta and price_history tables
    /// matching the schema from migrations 001 through 011.
    async fn setup_db() -> SqlitePool {
        let pool = SqlitePool::connect("sqlite::memory:")
            .await
            .expect("in-memory pool");

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS schema_meta (
                key   TEXT PRIMARY KEY,
                value TEXT NOT NULL
            )",
        )
        .execute(&pool)
        .await
        .unwrap();

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
                synced_at    INTEGER NOT NULL,
                is_active    INTEGER DEFAULT 1,
                delisted_at  INTEGER,
                user_id      TEXT
            )",
        )
        .execute(&pool)
        .await
        .unwrap();

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS price_history (
                id          INTEGER PRIMARY KEY AUTOINCREMENT,
                sku         TEXT NOT NULL,
                price       REAL NOT NULL,
                recorded_at INTEGER NOT NULL
            )",
        )
        .execute(&pool)
        .await
        .unwrap();

        sqlx::query(
            "CREATE INDEX IF NOT EXISTS idx_price_history_sku ON price_history(sku)",
        )
        .execute(&pool)
        .await
        .unwrap();

        pool
    }

    /// Insert a product for testing.
    #[allow(dead_code, clippy::too_many_arguments)]
    async fn seed_product(
        pool: &SqlitePool,
        sku: &str,
        name: &str,
        brand: &str,
        category: &str,
        price: f64,
        synced_at: i64,
        is_active: bool,
    ) {
        sqlx::query(
            r#"INSERT INTO products_meta
               (sku, source_id, name, brand, model, category, subcategory,
                price, currency, condition, availability, url, image_url,
                seller, location, synced_at, is_active)
               VALUES (?1, ?2, ?3, ?4, '', ?5, '', ?6, 'USD', 'new', 'in_stock',
                       'https://example.com/' || ?1, '', 'Test Seller', 'USA', ?7, ?8)"#,
        )
        .bind(sku)
        .bind("test-source")
        .bind(name)
        .bind(brand)
        .bind(category)
        .bind(price)
        .bind(synced_at)
        .bind(is_active)
        .execute(pool)
        .await
        .unwrap();
    }

    /// Insert a price-history row for testing.
    #[allow(dead_code)]
    async fn seed_price(
        pool: &SqlitePool,
        sku: &str,
        price: f64,
        recorded_at: i64,
    ) {
        sqlx::query(
            "INSERT INTO price_history (sku, price, recorded_at) VALUES (?1, ?2, ?3)",
        )
        .bind(sku)
        .bind(price)
        .bind(recorded_at)
        .execute(pool)
        .await
        .unwrap();
    }

    // ── Test: get_featured ──────────────────────────────────────────────

    #[tokio::test]
    async fn get_featured_returns_empty_when_no_products() {
        let pool = setup_db().await;
        let svc = ProductQueryService::new(pool);

        let result = svc.get_featured(6, None).await.unwrap();
        assert!(result.is_empty(), "expected empty result for empty catalog");
    }

    #[tokio::test]
    async fn get_featured_returns_requested_number() {
        let pool = setup_db().await;

        for i in 1..=5 {
            seed_product(
                &pool,
                &format!("SKU-F{:03}", i),
                &format!("Product {}", i),
                "Brand",
                "Guitars",
                i as f64 * 100.0,
                1000 + i,
                true,
            )
            .await;
        }

        let svc = ProductQueryService::new(pool.clone());
        let result = svc.get_featured(3, None).await.unwrap();
        assert_eq!(result.len(), 3, "expected 3 products with limit=3");
    }

    #[tokio::test]
    async fn get_featured_returns_all_when_less_than_limit() {
        let pool = setup_db().await;

        seed_product(&pool, "SKU-A", "Alpha", "Brand", "Guitars", 100.0, 1000, true).await;
        seed_product(&pool, "SKU-B", "Beta", "Brand", "Guitars", 200.0, 1001, true).await;

        let svc = ProductQueryService::new(pool.clone());
        let result = svc.get_featured(6, None).await.unwrap();
        assert_eq!(result.len(), 2, "expected both products when fewer than limit");
    }

    #[tokio::test]
    async fn get_featured_excludes_inactive_products() {
        let pool = setup_db().await;

        seed_product(&pool, "SKU-ACTIVE", "Active Guitar", "Fender", "Guitars", 999.99, 1000, true).await;
        seed_product(&pool, "SKU-INACTIVE", "Inactive Guitar", "Gibson", "Guitars", 2499.99, 1001, false).await;

        let svc = ProductQueryService::new(pool.clone());
        let result = svc.get_featured(10, None).await.unwrap();
        assert_eq!(result.len(), 1, "expected only the active product");
        assert_eq!(result[0].sku, "SKU-ACTIVE");
    }

    // ── Test: get_price_drops ───────────────────────────────────────────

    #[tokio::test]
    async fn get_price_drops_returns_empty_when_no_products() {
        let pool = setup_db().await;
        let svc = ProductQueryService::new(pool);

        let result = svc.get_price_drops(6, None).await.unwrap();
        assert!(result.is_empty(), "expected empty result for empty catalog");
    }

    #[tokio::test]
    async fn get_price_drops_returns_biggest_drop_first() {
        let pool = setup_db().await;

        // Product A: dropped from 1000 to 500 (drop of 500)
        // Product B: dropped from 800 to 600 (drop of 200)
        // Product C: dropped from 500 to 300 (drop of 200)
        for sku in &["SKU-DROP-A", "SKU-DROP-B", "SKU-DROP-C"] {
            seed_product(&pool, sku, "Drop Guitar", "Brand", "Guitars", 0.0, 1000, true).await;
        }

        // Price history: oldest = first_recorded, newest = last_recorded
        // Product A: first_price=1000, last_price=500
        seed_price(&pool, "SKU-DROP-A", 1000.0, 100).await;
        seed_price(&pool, "SKU-DROP-A", 750.0, 200).await;
        seed_price(&pool, "SKU-DROP-A", 500.0, 300).await;

        // Product B: first_price=800, last_price=600
        seed_price(&pool, "SKU-DROP-B", 800.0, 100).await;
        seed_price(&pool, "SKU-DROP-B", 600.0, 300).await;

        // Product C: first_price=500, last_price=300
        seed_price(&pool, "SKU-DROP-C", 500.0, 100).await;
        seed_price(&pool, "SKU-DROP-C", 300.0, 300).await;

        let svc = ProductQueryService::new(pool.clone());
        let result = svc.get_price_drops(5, None).await.unwrap();
        assert_eq!(result.len(), 3, "expected all 3 products with drops");
        // Biggest drop first (A: 500 > both 200)
        assert_eq!(result[0].sku, "SKU-DROP-A", "biggest drop should be first");
    }

    #[tokio::test]
    async fn get_price_drops_excludes_products_without_history() {
        let pool = setup_db().await;

        // Active product without any price_history rows
        seed_product(&pool, "SKU-NO-HISTORY", "No History", "Brand", "Guitars", 100.0, 1000, true).await;

        let svc = ProductQueryService::new(pool.clone());
        let result = svc.get_price_drops(5, None).await.unwrap();
        assert!(result.is_empty(), "expected no drops when no price history exists");
    }

    #[tokio::test]
    async fn get_price_drops_returns_empty_when_no_drops_exist() {
        let pool = setup_db().await;

        // Product with price history where last_price >= first_price (no drop)
        seed_product(&pool, "SKU-NO-DROP", "No Drop", "Brand", "Guitars", 800.0, 1000, true).await;
        seed_price(&pool, "SKU-NO-DROP", 500.0, 100).await;
        seed_price(&pool, "SKU-NO-DROP", 800.0, 300).await; // price went UP

        let svc = ProductQueryService::new(pool.clone());
        let result = svc.get_price_drops(5, None).await.unwrap();
        assert!(result.is_empty(), "expected no drops when prices have not dropped");
    }

    #[tokio::test]
    async fn get_price_drops_excludes_inactive_products() {
        let pool = setup_db().await;

        seed_product(&pool, "SKU-INACTIVE-DROP", "Inactive Drop", "Brand", "Guitars", 0.0, 1000, false).await;
        seed_price(&pool, "SKU-INACTIVE-DROP", 1000.0, 100).await;
        seed_price(&pool, "SKU-INACTIVE-DROP", 500.0, 300).await;

        let svc = ProductQueryService::new(pool.clone());
        let result = svc.get_price_drops(5, None).await.unwrap();
        assert!(result.is_empty(), "expected no drops from inactive products");
    }

    // ── Test: get_new_arrivals ──────────────────────────────────────────

    #[tokio::test]
    async fn get_new_arrivals_returns_empty_when_no_products() {
        let pool = setup_db().await;
        let svc = ProductQueryService::new(pool);

        let result = svc.get_new_arrivals(6, None).await.unwrap();
        assert!(result.is_empty(), "expected empty result for empty catalog");
    }

    #[tokio::test]
    async fn get_new_arrivals_returns_newest_first() {
        let pool = setup_db().await;

        // synced_at: C=3000 (newest), B=2000, A=1000 (oldest)
        seed_product(&pool, "SKU-N-A", "Old Arrival", "Brand", "Guitars", 100.0, 1000, true).await;
        seed_product(&pool, "SKU-N-B", "Mid Arrival", "Brand", "Guitars", 200.0, 2000, true).await;
        seed_product(&pool, "SKU-N-C", "New Arrival", "Brand", "Guitars", 300.0, 3000, true).await;

        let svc = ProductQueryService::new(pool.clone());
        let result = svc.get_new_arrivals(6, None).await.unwrap();
        assert_eq!(result.len(), 3, "expected all 3 products");
        assert_eq!(result[0].sku, "SKU-N-C", "newest should be first");
        assert_eq!(result[1].sku, "SKU-N-B", "middle should be second");
        assert_eq!(result[2].sku, "SKU-N-A", "oldest should be last");
    }

    #[tokio::test]
    async fn get_new_arrivals_excludes_inactive() {
        let pool = setup_db().await;

        seed_product(&pool, "SKU-ACTIVE-NEW", "Active", "Brand", "Guitars", 100.0, 2000, true).await;
        seed_product(&pool, "SKU-INACTIVE-NEW", "Inactive", "Brand", "Guitars", 200.0, 3000, false).await;

        let svc = ProductQueryService::new(pool.clone());
        let result = svc.get_new_arrivals(6, None).await.unwrap();
        assert_eq!(result.len(), 1, "expected only the active product");
        assert_eq!(result[0].sku, "SKU-ACTIVE-NEW");
    }

    // ── Test: get_by_sku ────────────────────────────────────────────────

    #[tokio::test]
    async fn get_by_sku_returns_product() {
        let pool = setup_db().await;

        seed_product(&pool, "FENDER-STRAT-001", "Fender Stratocaster", "Fender", "Electric Guitars", 1599.99, 1000, true).await;

        let svc = ProductQueryService::new(pool.clone());
        let product = svc.get_by_sku("FENDER-STRAT-001").await.unwrap();
        assert_eq!(product.sku, "FENDER-STRAT-001");
        assert_eq!(product.name, "Fender Stratocaster");
        assert_eq!(product.brand, "Fender");
        assert_eq!(product.price, 1599.99);
    }

    #[tokio::test]
    async fn get_by_sku_matches_case_insensitively() {
        let pool = setup_db().await;

        seed_product(&pool, "FENDER-001", "Fender Guitar", "Fender", "Guitars", 999.99, 1000, true).await;

        let svc = ProductQueryService::new(pool.clone());
        let product = svc.get_by_sku("fender-001").await.unwrap();
        assert_eq!(product.sku, "FENDER-001", "should match via LOWER");
    }

    #[tokio::test]
    async fn get_by_sku_returns_not_found_for_invalid_sku() {
        let pool = setup_db().await;
        let svc = ProductQueryService::new(pool.clone());

        let err = svc.get_by_sku("NONEXISTENT").await.unwrap_err();
        assert!(
            matches!(err, AppError::NotFound),
            "expected NotFound, got: {err}"
        );
    }

    #[tokio::test]
    async fn get_by_sku_returns_not_found_for_inactive_product() {
        let pool = setup_db().await;

        seed_product(&pool, "SKU-INACTIVE-DETAIL", "Inactive", "Brand", "Guitars", 100.0, 1000, false).await;

        let svc = ProductQueryService::new(pool.clone());
        let err = svc.get_by_sku("SKU-INACTIVE-DETAIL").await.unwrap_err();
        assert!(
            matches!(err, AppError::NotFound),
            "expected NotFound for inactive product, got: {err}"
        );
    }

    #[tokio::test]
    async fn get_by_sku_returns_invalid_input_for_empty_sku() {
        let pool = setup_db().await;
        let svc = ProductQueryService::new(pool.clone());

        let err = svc.get_by_sku("").await.unwrap_err();
        assert!(
            matches!(err, AppError::InvalidInput(_)),
            "expected InvalidInput for empty sku, got: {err}"
        );
    }
}
