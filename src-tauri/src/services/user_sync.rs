// SPDX-License-Identifier: GPL-3.0-or-later

use crate::domain::product::{RawProduct, SyncState};
use crate::services::connection_manager::ConnectionManager;
use crate::services::reverb_api::{normalize_reverb_condition, ReverbApiClient, ReverbListing};
use crate::services::sync::SyncResult;
use crate::AppError;
use reqwest::Client;
use sqlx::SqlitePool;

const REVERB_BASE_URL: &str = "https://api.reverb.com";

/// Service for syncing user-authenticated store listings into `products_meta`.
///
/// Fetches all pages of listings from the Reverb API, maps them to `RawProduct`
/// with `user_id` set, and batch upserts into `products_meta`. Products that
/// are no longer in the remote listing are soft-deleted.
pub struct UserSyncService {
    pool: SqlitePool,
    http_client: Client,
    connection_manager: ConnectionManager,
    /// Overridable base URL for Reverb API (defaults to production; tests inject mock).
    reverb_base_url: String,
}

impl UserSyncService {
    /// Create a new service pointing at the production Reverb API.
    pub fn new(pool: SqlitePool, http_client: Client, connection_manager: ConnectionManager) -> Self {
        Self {
            pool,
            http_client,
            connection_manager,
            reverb_base_url: REVERB_BASE_URL.to_string(),
        }
    }

    /// Create a service with a custom Reverb API base URL (for testing with httpmock).
    #[cfg(test)]
    pub fn new_with_url(
        pool: SqlitePool,
        http_client: Client,
        connection_manager: ConnectionManager,
        base_url: String,
    ) -> Self {
        Self {
            pool,
            http_client,
            connection_manager,
            reverb_base_url: base_url,
        }
    }

    /// Fetch all paginated listings for a connection, upsert with `user_id`,
    /// delist absent products, and update `synced_at`.
    pub async fn sync(&self, conn_id: i64) -> Result<SyncResult, AppError> {
        // 1. Decrypt the stored token
        let token = self.connection_manager.decrypt_token(conn_id).await?;

        // 2. Create Reverb API client (respects mock base URL in tests)
        let reverb = ReverbApiClient::new_with_url(self.http_client.clone(), self.reverb_base_url.clone());

        // 3. Fetch ALL pages of listings (cursor-based via _links.next.href)
        let mut all_listings: Vec<ReverbListing> = Vec::new();
        let mut page = 1u32;
        loop {
            let response = reverb.fetch_listings(&token, page).await?;
            let count = response.listings.len();
            all_listings.extend(response.listings);

            // Check for next page via _links.next.href
            match response._links.next {
                Some(next) if next.href.is_some() => {
                    page += 1;
                }
                _ => break,
            }

            // Safety: prevent infinite loop if API keeps returning data
            if count == 0 {
                break;
            }
        }

        let loaded = all_listings.len() as u32;
        let synced_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;

        // 4. Map listings → RawProduct
        let products: Vec<RawProduct> = all_listings
            .iter()
            .map(map_listing_to_product)
            .collect();

        // 5. Batch upsert with user_id = conn_id
        let updated = self
            .batch_upsert_user_products(&products, conn_id, synced_at)
            .await?;

        // 6. Delist products no longer in Reverb for this user
        let delisted = self
            .delist_absent_products(conn_id, &products, synced_at)
            .await?;

        // 7. Update synced_at on the connection
        sqlx::query("UPDATE store_connections SET synced_at = ?1 WHERE id = ?2")
            .bind(synced_at)
            .bind(conn_id)
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        let source_id = format!("user-{}", conn_id);

        Ok(SyncResult {
            source_id,
            products_loaded: loaded,
            products_updated: updated,
            delisted,
            state: SyncState::Done,
            progress: 1.0,
            drops: vec![],
            drops_sent: 0,
        })
    }

    /// Batch upsert user products into `products_meta` with `user_id` set.
    async fn batch_upsert_user_products(
        &self,
        products: &[RawProduct],
        conn_id: i64,
        synced_at: i64,
    ) -> Result<u32, AppError> {
        let user_id = conn_id.to_string();
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
                    seller, location, synced_at, user_id)
                   VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17)"#,
            )
            .bind(&p.sku)
            .bind(&p.seller)
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
            .bind(&user_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| {
                AppError::Database(format!("batch upsert failed for SKU {}: {e}", p.sku))
            })?;

            rows_affected += result.rows_affected() as u32;
        }

        tx.commit()
            .await
            .map_err(|e| AppError::Database(format!("failed to commit transaction: {e}")))?;

        Ok(rows_affected)
    }

    /// Soft-delete products for this user that are no longer in the current batch.
    async fn delist_absent_products(
        &self,
        conn_id: i64,
        products: &[RawProduct],
        synced_at: i64,
    ) -> Result<u32, AppError> {
        let user_id = conn_id.to_string();

        if products.is_empty() {
            let result = sqlx::query(
                "UPDATE products_meta SET is_active = 0, delisted_at = ?2
                 WHERE user_id = ?1 AND is_active = 1",
            )
            .bind(&user_id)
            .bind(synced_at)
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

            return Ok(result.rows_affected() as u32);
        }

        // Temp table to hold current batch SKUs for diff
        sqlx::query("CREATE TABLE IF NOT EXISTS _sync_user_skus (sku TEXT PRIMARY KEY)")
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        sqlx::query("DELETE FROM _sync_user_skus")
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        for p in products {
            sqlx::query("INSERT OR IGNORE INTO _sync_user_skus (sku) VALUES (?1)")
                .bind(&p.sku)
                .execute(&self.pool)
                .await
                .map_err(|e| AppError::Database(e.to_string()))?;
        }

        let result = sqlx::query(
            "UPDATE products_meta
             SET is_active = 0, delisted_at = ?2
             WHERE user_id = ?1
               AND is_active = 1
               AND sku NOT IN (SELECT sku FROM _sync_user_skus)",
        )
        .bind(&user_id)
        .bind(synced_at)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        sqlx::query("DROP TABLE IF EXISTS _sync_user_skus")
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(result.rows_affected() as u32)
    }
}

/// Map a `ReverbListing` to `RawProduct` with SKU prefixed `reverb-user-`.
///
/// Fields mapped per spec: title → name, price.amount → price,
/// price.currency → currency, _links.web.href → url,
/// photos[]._links.small.href → image_url.
pub(crate) fn map_listing_to_product(listing: &ReverbListing) -> RawProduct {
    let condition = match &listing.condition {
        Some(cond) => normalize_reverb_condition(cond),
        None => "unknown",
    };

    let image_url = listing
        .photos
        .first()
        .and_then(|p| p._links.small.as_ref())
        .map(|l| l.href.clone())
        .unwrap_or_default();

    let url = listing
        ._links
        .web
        .as_ref()
        .map(|w| w.href.clone())
        .unwrap_or_default();

    let brand = listing.make.clone().unwrap_or_else(|| "Reverb".to_string());
    let model = listing.model.clone().unwrap_or_default();

    RawProduct {
        sku: format!("reverb-user-{}", listing.id),
        name: listing.title.clone(),
        brand,
        model,
        category: "Unknown".to_string(),
        subcategory: "Unknown".to_string(),
        price: listing.price.amount,
        currency: listing.price.currency.clone(),
        condition: condition.to_string(),
        availability: match listing.state.as_deref() {
            Some("published") => "in_stock",
            _ => "unknown",
        }
        .to_string(),
        url,
        image_url,
        specs_json: "{}".to_string(),
        seller: "Reverb".to_string(),
        location: String::new(),
    }
}

// ── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::connection_manager::{derive_encryption_key, encrypt_token};
    use aes_gcm::{Aes256Gcm, KeyInit};
    use httpmock::prelude::*;

    /// Create an in-memory SQLite pool with the tables needed for user_sync tests.
    async fn setup_db() -> SqlitePool {
        let pool = SqlitePool::connect("sqlite::memory:")
            .await
            .expect("in-memory pool");

        // schema_meta (needed by checks in some code paths)
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS schema_meta (
                key   TEXT PRIMARY KEY,
                value TEXT NOT NULL
            )",
        )
        .execute(&pool)
        .await
        .unwrap();

        // products_meta with user_id column
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
                url          TEXT NOT NULL CHECK(url LIKE 'https://%' OR url = ''),
                image_url    TEXT CHECK(image_url = '' OR image_url LIKE 'https://%'),
                seller       TEXT,
                location     TEXT,
                synced_at    INTEGER NOT NULL,
                is_active    INTEGER DEFAULT 1,
                delisted_at  INTEGER
            )",
        )
        .execute(&pool)
        .await
        .unwrap();

        // Add user_id column (idempotent)
        let _ = sqlx::query("ALTER TABLE products_meta ADD COLUMN user_id TEXT")
            .execute(&pool)
            .await;

        // store_connections table (migration 012)
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS store_connections (
                id              INTEGER PRIMARY KEY AUTOINCREMENT,
                store_id        TEXT NOT NULL,
                label           TEXT NOT NULL DEFAULT '',
                token_encrypted TEXT NOT NULL,
                username        TEXT,
                connected_at    INTEGER NOT NULL,
                synced_at       INTEGER,
                is_active       INTEGER DEFAULT 1,
                UNIQUE(store_id)
            )",
        )
        .execute(&pool)
        .await
        .unwrap();

        pool
    }

    /// Encrypt a token using the same key derivation as ConnectionManager,
    /// insert a connection row, and return the connection id.
    async fn seed_connection(pool: &SqlitePool, token: &str) -> i64 {
        let key = derive_encryption_key();
        let cipher = Aes256Gcm::new_from_slice(&key).unwrap();
        let encrypted = encrypt_token(&cipher, token).unwrap();

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        sqlx::query(
            r#"INSERT INTO store_connections
               (store_id, label, token_encrypted, username, connected_at, synced_at, is_active)
               VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)"#,
        )
        .bind("reverb")
        .bind("Reverb")
        .bind(&encrypted)
        .bind("@guitarist")
        .bind(now)
        .bind(now)
        .bind(1_i32)
        .execute(pool)
        .await
        .unwrap();

        sqlx::query_scalar("SELECT id FROM store_connections")
            .fetch_one(pool)
            .await
            .unwrap()
    }

    fn sample_listing_json(id: i64, title: &str, price: f64) -> serde_json::Value {
        serde_json::json!({
            "id": id,
            "title": title,
            "price": {"amount": price, "currency": "USD"},
            "condition": {"slug": "excellent", "display_name": "Excellent"},
            "photos": [{"_links": {"small": {"href": format!("https://img.test/{id}.jpg")}}}],
            "_links": {"web": {"href": format!("https://reverb.com/item/{id}")}},
            "state": "published",
            "make": "Fender",
            "model": "Stratocaster"
        })
    }

    // ── RED phase: Test that user_sync.sync() fetches listings and
    //    upserts with user_id. This code does NOT exist yet — the test
    //    is written first, and Will FAIL until production code is added.
    // ────────────────────────────────────────────────────────────────────

    #[tokio::test]
    async fn user_sync_inserts_products_with_user_id_on_first_sync() {
        let pool = setup_db().await;
        let cm = crate::services::connection_manager::ConnectionManager::new(pool.clone()).await;
        let conn_id = seed_connection(&pool, "test_pat_token").await;

        // Setup httpmock for a single page of listings
        let server = MockServer::start();
        let mock = server.mock(|when, then| {
            when.method(GET)
                .path("/api/my/listings")
                .query_param("page", "1")
                .query_param("per", "50");
            then.status(200)
                .header("Content-Type", "application/json")
                .body(serde_json::json!({
                    "listings": [
                        sample_listing_json(100, "Fender Stratocaster", 1599.99),
                        sample_listing_json(101, "Gibson Les Paul", 2499.99)
                    ],
                    "_links": {},
                    "total": 2
                }).to_string());
        });

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(5))
            .build()
            .unwrap();

        let svc = UserSyncService::new_with_url(
            pool.clone(),
            client,
            cm,
            server.base_url(),
        );

        let result = svc.sync(conn_id).await.expect("sync should succeed");

        // ── Assertions (TDD RED phase validates the contract) ──────────

        // 2 products loaded
        assert_eq!(result.products_loaded, 2, "expected 2 products loaded");
        assert_eq!(result.products_updated, 2, "expected 2 rows affected");
        assert_eq!(result.delisted, 0, "first sync should not delist anything");
        assert_eq!(result.state, SyncState::Done);

        // Verify products exist in DB with correct user_id
        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM products_meta WHERE user_id = ?1",
        )
        .bind(conn_id.to_string())
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(count, 2, "expected 2 products with user_id = {conn_id}");

        // Verify SKUs match the reverb-user-{id} format
        let sku1: String = sqlx::query_scalar(
            "SELECT sku FROM products_meta WHERE user_id = ?1 ORDER BY sku LIMIT 1",
        )
        .bind(conn_id.to_string())
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(sku1, "reverb-user-100", "SKU must use reverb-user- prefix");

        // Verify synced_at was updated on the connection
        let synced_at: Option<i64> = sqlx::query_scalar(
            "SELECT synced_at FROM store_connections WHERE id = ?1",
        )
        .bind(conn_id)
        .fetch_one(&pool)
        .await
        .unwrap();
        assert!(
            synced_at.is_some() && synced_at.unwrap() > 0,
            "synced_at must be set after sync"
        );

        mock.assert_calls(1);
    }

    // ── Second test: pagination — 2 pages of listings ─────────────────────
    //    (TRIANGULATE phase: forces real pagination logic)

    #[tokio::test]
    async fn user_sync_fetches_paginated_listings_across_multiple_pages() {
        let pool = setup_db().await;
        let cm = crate::services::connection_manager::ConnectionManager::new(pool.clone()).await;
        let conn_id = seed_connection(&pool, "pat_paginated").await;

        let server = MockServer::start();

        // Page 1: 2 listings + next link
        let _page1 = server.mock(|when, then| {
            when.method(GET)
                .path("/api/my/listings")
                .query_param("page", "1")
                .query_param("per", "50");
            then.status(200)
                .header("Content-Type", "application/json")
                .body(serde_json::json!({
                    "listings": [
                        sample_listing_json(200, "Fender Telecaster", 1299.99),
                        sample_listing_json(201, "PRS Custom 24", 3499.99)
                    ],
                    "_links": {"next": {"href": format!("{}/api/my/listings?page=2&per=50", server.base_url())}},
                    "total": 3
                }).to_string());
        });

        // Page 2: 1 listing + no next link
        let _page2 = server.mock(|when, then| {
            when.method(GET)
                .path("/api/my/listings")
                .query_param("page", "2")
                .query_param("per", "50");
            then.status(200)
                .header("Content-Type", "application/json")
                .body(serde_json::json!({
                    "listings": [
                        sample_listing_json(202, "Ibanez RG", 899.99)
                    ],
                    "_links": {},
                    "total": 3
                }).to_string());
        });

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(5))
            .build()
            .unwrap();

        let svc = UserSyncService::new_with_url(
            pool.clone(),
            client,
            cm,
            server.base_url(),
        );

        let result = svc.sync(conn_id).await.expect("paginated sync should succeed");

        assert_eq!(result.products_loaded, 3, "expected 3 products across 2 pages");
        assert_eq!(result.products_updated, 3, "expected 3 rows affected");

        // Verify all 3 products in DB
        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM products_meta WHERE user_id = ?1",
        )
        .bind(conn_id.to_string())
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(count, 3, "expected 3 products in DB");
    }

    // ── Third test: re-sync delists absent products ───────────────────────
    //    (TRIANGULATE: exercises delist_absent_products)

    #[tokio::test]
    async fn user_sync_delists_products_no_longer_in_reverb() {
        let pool = setup_db().await;
        let cm = crate::services::connection_manager::ConnectionManager::new(pool.clone()).await;
        let conn_id = seed_connection(&pool, "pat_delist").await;

        // Seed existing products in DB that are NOT in the current batch
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        let user_id_str = conn_id.to_string();

        // This product will be absent from the new sync batch
        sqlx::query(
            "INSERT INTO products_meta (sku, source_id, name, brand, model, category, subcategory, 
             price, currency, condition, availability, url, image_url, seller, location, synced_at, user_id, is_active)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, 1)",
        )
        .bind("reverb-user-999")
        .bind("Reverb")
        .bind("Delisted Product")
        .bind("Fender")
        .bind("Jazzmaster")
        .bind("Guitars")
        .bind("Electric")
        .bind(1999.99_f64)
        .bind("USD")
        .bind("used")
        .bind("in_stock")
        .bind("https://reverb.com/item/999")
        .bind("")
        .bind("Reverb")
        .bind("")
        .bind(now)
        .bind(&user_id_str)
        .execute(&pool)
        .await
        .unwrap();

        // Also seed a product from a different user to verify it's NOT delisted
        sqlx::query(
            "INSERT INTO products_meta (sku, source_id, name, brand, model, category, subcategory,
             price, currency, condition, availability, url, image_url, seller, location, synced_at, user_id, is_active)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, 1)",
        )
        .bind("other-user-sku-1")
        .bind("Reverb")
        .bind("Other User Product")
        .bind("Gibson")
        .bind("Les Paul")
        .bind("Guitars")
        .bind("Electric")
        .bind(2999.99_f64)
        .bind("USD")
        .bind("new")
        .bind("in_stock")
        .bind("https://reverb.com/item/other")
        .bind("")
        .bind("Reverb")
        .bind("")
        .bind(now)
        .bind("99999") // different user_id
        .execute(&pool)
        .await
        .unwrap();

        // Setup httpmock: current batch has only listing 300
        let server = MockServer::start();
        let _mock = server.mock(|when, then| {
            when.method(GET)
                .path("/api/my/listings")
                .query_param("page", "1")
                .query_param("per", "50");
            then.status(200)
                .header("Content-Type", "application/json")
                .body(serde_json::json!({
                    "listings": [
                        sample_listing_json(300, "Fender Stratocaster", 1599.99)
                    ],
                    "_links": {},
                    "total": 1
                }).to_string());
        });

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(5))
            .build()
            .unwrap();

        let svc = UserSyncService::new_with_url(pool.clone(), client, cm, server.base_url());
        let result = svc.sync(conn_id).await.expect("re-sync should succeed");

        assert_eq!(result.products_loaded, 1, "1 product in current batch");
        assert_eq!(result.delisted, 1, "old product should be delisted");

        // Verify the absent product is now inactive
        let is_active: i32 = sqlx::query_scalar(
            "SELECT is_active FROM products_meta WHERE sku = 'reverb-user-999'",
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(is_active, 0, "absent product should be delisted");

        // Verify the other user's product is still active
        let other_active: i32 = sqlx::query_scalar(
            "SELECT is_active FROM products_meta WHERE sku = 'other-user-sku-1'",
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(other_active, 1, "other user's product must remain active");
    }
}
