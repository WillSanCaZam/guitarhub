// SPDX-License-Identifier: GPL-3.0-or-later

use crate::domain::product::{CatalogFile, RawProduct, SyncState};
use crate::repository::price_drop_notifications::PriceDropNotificationsRepo;
use crate::repository::price_history::PriceHistoryRepo;
use crate::services::price_drop::{is_price_drop, PriceDrop, Thresholds, COOLDOWN_SECS};
use crate::AppError;
use sqlx::SqlitePool;

/// Trait abstracting catalog synchronization from various sources.
#[async_trait::async_trait]
pub trait SyncService: Send + Sync {
    /// Fetch a remote catalog JSON and upsert all products into the database.
    async fn sync_catalog(&self, url: &str) -> Result<SyncResult, AppError>;
}

/// Result returned after a catalog sync operation.
///
/// `drops` lists every price drop detected during this sync that cleared
/// both the materiality check (`is_price_drop`) and the per-SKU cooldown.
/// `drops_sent` is populated by `sync_command` after the dispatch loop
/// runs (i.e. the number of drops whose `AlertDispatcher::send` returned Ok).
#[derive(Debug, Clone, serde::Serialize)]
pub struct SyncResult {
    pub source_id: String,
    pub products_loaded: u32,
    pub products_updated: u32,
    pub state: SyncState,
    pub progress: f32,
    pub drops: Vec<PriceDrop>,
    pub drops_sent: u32,
}

/// A `SyncService` that fetches a remote catalog JSON over HTTP, runs a
/// state machine (`idle → downloading → validating → sanitizing → inserting
/// → done | failed`), and upserts products into `products_meta`.
pub struct CatalogSyncService {
    pool: SqlitePool,
    http_client: reqwest::Client,
}

impl CatalogSyncService {
    pub fn new(pool: SqlitePool, http_client: reqwest::Client) -> Self {
        Self { pool, http_client }
    }

    /// Update the sync_state row for a source.
    async fn set_state(
        &self,
        source_id: &str,
        status: &str,
        error_msg: Option<&str>,
    ) -> Result<(), AppError> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        sqlx::query(
            r#"INSERT INTO sync_state (source_id, enabled, last_synced, status, error_msg)
               VALUES (?1, 1, ?2, ?3, ?4)
               ON CONFLICT(source_id) DO UPDATE SET
                 last_synced = excluded.last_synced,
                 status = excluded.status,
                 error_msg = excluded.error_msg"#,
        )
        .bind(source_id)
        .bind(now)
        .bind(status)
        .bind(error_msg)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(())
    }

    /// Check if a sync is already running for the given source.
    async fn check_not_running(&self, source_id: &str) -> Result<(), AppError> {
        let status: Option<String> = sqlx::query_scalar(
            "SELECT status FROM sync_state WHERE source_id = ?",
        )
        .bind(source_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        if let Some(ref s) = status {
            if let Some(state) = SyncState::from_label(s) {
                if state.is_running() {
                    return Err(AppError::SyncInProgress);
                }
            }
        }
        Ok(())
    }

    /// Insert or replace every product into `products_meta`, then
    /// write each new price to `price_history`, then detect drops
    /// that pass the materiality check AND the per-SKU cooldown.
    ///
    /// Returns `(loaded, updated, drops)`. `drops` is the list of
    /// `PriceDrop`s the caller should dispatch to the alert channel.
    async fn upsert_products(
        &self,
        source_id: &str,
        products: &[RawProduct],
    ) -> Result<(u32, u32, Vec<PriceDrop>), AppError> {
        let total = products.len() as u32;
        let mut updated = 0u32;
        let mut drops: Vec<PriceDrop> = Vec::new();
        let synced_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        let price_history = PriceHistoryRepo::new(self.pool.clone());
        let cooldown_repo = PriceDropNotificationsRepo::new(self.pool.clone());
        let thresholds = Thresholds::default();
        // Channel is hardcoded "app" for service-layer drops — the command
        // layer is responsible for switching to ntfy/webhook based on
        // settings.alert_channel. (See design.md decision: AppHandle bridge
        // lives in sync_command, not in the service layer.)
        let channel = "app";

        for p in products {
            // Read the previous price from price_history BEFORE we insert
            // the new row (so this sync's "previous" stays the OLD value).
            let prev_price = match price_history.get_last_price(&p.sku).await {
                Ok(p) => p,
                Err(e) => {
                    tracing::error!(
                        sku = %p.sku,
                        error = %e,
                        "failed to read previous price; skipping drop detection for this SKU"
                    );
                    None
                }
            };

            // Existing INSERT OR REPLACE into products_meta — unchanged.
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
            updated += result.rows_affected() as u32;

            // Write the new price to price_history. A failure here is logged
            // but does NOT abort the sync — the product is already in
            // products_meta. Subsequent syncs will still detect drops based
            // on whatever history rows exist.
            if let Err(e) = price_history
                .record_price(&p.sku, p.price, source_id, synced_at)
                .await
            {
                tracing::error!(
                    sku = %p.sku,
                    error = %e,
                    "failed to write price_history row; continuing sync"
                );
                continue;
            }

            // Run the pure drop detector. `prev_price = None` (first obs)
            // short-circuits to None — we never fire on first observation.
            if let Some(drop) =
                is_price_drop(&p.sku, Some(p.price), prev_price, &thresholds, channel)
            {
                // Cooldown check: skip if we've notified for this SKU within
                // the cooldown window.
                let in_cooldown = match cooldown_repo.get_last_notified(&p.sku).await {
                    Ok(Some(last)) => synced_at - last < COOLDOWN_SECS,
                    Ok(None) => false,
                    Err(e) => {
                        tracing::error!(
                            sku = %p.sku,
                            error = %e,
                            "failed to read cooldown row; assuming not in cooldown"
                        );
                        false
                    }
                };
                if !in_cooldown {
                    drops.push(drop);
                }
            }
        }
        Ok((total, updated, drops))
    }
}

#[async_trait::async_trait]
impl SyncService for CatalogSyncService {
    async fn sync_catalog(&self, url: &str) -> Result<SyncResult, AppError> {
        // ── Download ─────────────────────────────────────────────────────
        let response = self
            .http_client
            .get(url)
            .send()
            .await
            .map_err(|e| AppError::Network(e.to_string()))?;

        if !response.status().is_success() {
            return Err(AppError::Network(format!("HTTP {}", response.status())));
        }

        let catalog: CatalogFile = response
            .json()
            .await
            .map_err(|e| AppError::InvalidInput(format!("Invalid catalog JSON: {e}")))?;

        let source_id = &catalog.source_id;

        // ── Check concurrent ─────────────────────────────────────────────
        self.check_not_running(source_id).await?;

        // ── State machine ────────────────────────────────────────────────
        self.set_state(source_id, SyncState::Downloading.as_str(), None)
            .await?;

        self.set_state(source_id, SyncState::Validating.as_str(), None)
            .await?;

        self.set_state(source_id, SyncState::Sanitizing.as_str(), None)
            .await?;

        self.set_state(source_id, SyncState::Inserting.as_str(), None)
            .await?;

        let (loaded, updated, drops) = self
            .upsert_products(source_id, &catalog.products)
            .await?;

        self.set_state(source_id, SyncState::Done.as_str(), None)
            .await?;

        Ok(SyncResult {
            source_id: source_id.clone(),
            products_loaded: loaded,
            products_updated: updated,
            state: SyncState::Done,
            progress: 1.0,
            drops,
            drops_sent: 0,
        })
    }
}

// ── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::price_drop::PriceDrop;
    use httpmock::prelude::*;
    use sqlx::SqlitePool;

    // ── SyncResult.drops field test (RED — SyncResult has no `drops` yet) ──

    #[tokio::test]
    async fn sync_result_has_drops_field_empty_initially() {
        // A fresh SyncResult must have an empty `drops` vec — no compiler
        // errors, no panics. The `drops_sent` counter must start at zero.
        let pool = setup_db().await;
        let _ = pool; // silence unused warning when this test is the only one running
        let r = SyncResult {
            source_id: "test".to_string(),
            products_loaded: 0,
            products_updated: 0,
            state: SyncState::Done,
            progress: 1.0,
            drops: Vec::<PriceDrop>::new(),
            drops_sent: 0,
        };
        assert!(r.drops.is_empty(), "fresh SyncResult.drops must be empty");
        assert_eq!(r.drops_sent, 0, "fresh SyncResult.drops_sent must be 0");
    }

    // ── upsert_products writes price_history rows (RED) ──────────────────

    /// After syncing 1 product, exactly 1 `price_history` row exists.
    /// `recorded_at` is close to "now" (within 5 seconds).
    #[tokio::test]
    async fn upsert_products_writes_price_history_rows() {
        let pool = setup_db().await;
        let svc = CatalogSyncService::new(pool.clone(), reqwest::Client::new());

        let products = vec![raw_product("SKU-HIST-1", 750.0)];
        let before = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        let (_loaded, _updated, drops) = svc
            .upsert_products("test-source", &products)
            .await
            .expect("upsert_products must succeed");
        let after = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        // 1 history row for SKU-HIST-1
        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM price_history WHERE sku = 'SKU-HIST-1'",
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(count, 1, "expected 1 price_history row");

        // recorded_at is within the test window
        let recorded_at: i64 = sqlx::query_scalar(
            "SELECT recorded_at FROM price_history WHERE sku = 'SKU-HIST-1'",
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert!(
            recorded_at >= before && recorded_at <= after + 5,
            "recorded_at {recorded_at} not within test window [{before}..{after}]"
        );

        // First-observation ⇒ no drop detected.
        assert!(
            drops.is_empty(),
            "first observation must not produce a drop, got {}",
            drops.len()
        );
    }

    // ── upsert_products detects a 15% drop (RED) ─────────────────────────

    /// Seed price_history with $1000, then sync a $850 catalog. The detector
    /// must produce exactly 1 drop with `previous_price == 1000.0`.
    #[tokio::test]
    async fn upsert_products_detects_15pct_drop() {
        use crate::repository::price_history::PriceHistoryRepo;

        let pool = setup_db().await;
        let price_history = PriceHistoryRepo::new(pool.clone());
        let svc = CatalogSyncService::new(pool.clone(), reqwest::Client::new());

        // Seed: write a $1000 history row, recorded 1 hour ago.
        let one_hour_ago = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64
            - 3600;
        price_history
            .record_price("SKU-DROP-1", 1000.0, "test-source", one_hour_ago)
            .await
            .unwrap();

        // Now sync: catalog has SKU-DROP-1 at $850 (15% drop).
        let products = vec![raw_product("SKU-DROP-1", 850.0)];
        let (_loaded, _updated, drops) = svc
            .upsert_products("test-source", &products)
            .await
            .expect("upsert_products must succeed");

        // Exactly 1 drop detected
        assert_eq!(drops.len(), 1, "expected exactly 1 drop, got {:?}", drops);
        let drop = &drops[0];
        assert_eq!(drop.sku, "SKU-DROP-1");
        assert!(
            (drop.previous_price - 1000.0).abs() < f64::EPSILON,
            "expected previous_price=1000.0, got {}",
            drop.previous_price
        );
        assert!(
            (drop.new_price - 850.0).abs() < f64::EPSILON,
            "expected new_price=850.0, got {}",
            drop.new_price
        );

        // price_history now has 2 rows for this SKU (seed + new).
        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM price_history WHERE sku = 'SKU-DROP-1'",
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(count, 2, "expected 2 history rows (seed + new)");
    }

    // ── upsert_products first-observation suppression (RED) ──────────────

    /// Empty price_history + first sync ⇒ no drop is ever reported.
    /// (Sanity test for the first-observation suppression branch.)
    #[tokio::test]
    async fn upsert_products_first_observation_no_drop() {
        let pool = setup_db().await;
        let svc = CatalogSyncService::new(pool.clone(), reqwest::Client::new());

        let products = vec![raw_product("SKU-FIRST-1", 500.0)];
        let (_loaded, _updated, drops) = svc
            .upsert_products("test-source", &products)
            .await
            .expect("upsert_products must succeed");

        assert!(
            drops.is_empty(),
            "first observation must not produce a drop, got {} drops",
            drops.len()
        );
    }

    /// Build a `RawProduct` with a single price override.
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

    /// Create an in-memory pool with the tables needed for sync tests.
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
                synced_at    INTEGER NOT NULL
            )",
        )
        .execute(&pool)
        .await
        .unwrap();

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS sync_state (
                source_id        TEXT PRIMARY KEY,
                enabled          INTEGER DEFAULT 1,
                last_synced      INTEGER,
                last_run_id      TEXT,
                status           TEXT CHECK(status IN
                                 ('idle','downloading','validating','sanitizing',
                                  'inserting','done',
                                  'failed_network','failed_schema','failed_db')),
                error_msg        TEXT
            )",
        )
        .execute(&pool)
        .await
        .unwrap();

        // price_history table — schema mirrors the production migration 004.
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

        sqlx::query(
            "CREATE INDEX IF NOT EXISTS idx_price_history_sku_recorded
             ON price_history(sku, recorded_at)",
        )
        .execute(&pool)
        .await
        .unwrap();

        // price_drop_notifications table — schema mirrors the production migration 007.
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS price_drop_notifications (
                sku           TEXT    PRIMARY KEY,
                last_notified INTEGER NOT NULL,
                last_price    REAL    NOT NULL,
                channel       TEXT    NOT NULL
            )",
        )
        .execute(&pool)
        .await
        .unwrap();

        pool
    }

    /// Minimal valid catalog JSON for tests.
    fn sample_catalog_json(source_id: &str, products: &[serde_json::Value]) -> String {
        serde_json::json!({
            "schema_version": "1.0",
            "source_id": source_id,
            "generated_at": "2026-06-01T12:00:00Z",
            "run_id": "test-run-001",
            "products": products,
        })
        .to_string()
    }

    fn single_product() -> serde_json::Value {
        serde_json::json!({
            "sku": "TEST-SKU-001",
            "name": "Test Product",
            "brand": "TestBrand",
            "model": "TM-100",
            "category": "Electric Guitars",
            "subcategory": "Solid Body",
            "price": 999.99,
            "currency": "USD",
            "condition": "new",
            "availability": "in_stock",
            "url": "https://example.com/item",
            "image_url": "https://example.com/img.jpg",
            "seller": "Test Seller",
            "location": "USA"
        })
    }

    // ── Test: full lifecycle succeeds ───────────────────────────────────────

    #[tokio::test]
    async fn sync_full_lifecycle_transitions_to_done() {
        let server = MockServer::start();
        let body = sample_catalog_json("test-source", &[single_product()]);
        let mock = server.mock(|when, then| {
            when.method(GET).path("/catalog.json");
            then.status(200)
                .header("Content-Type", "application/json")
                .body(body);
        });

        let pool = setup_db().await;
        let client = reqwest::Client::new();
        let svc = CatalogSyncService::new(pool.clone(), client);
        let url = format!("{}/catalog.json", server.base_url());

        let result = svc.sync_catalog(&url).await.expect("sync should succeed");
        assert_eq!(result.source_id, "test-source");
        assert_eq!(result.products_loaded, 1);
        assert_eq!(result.state, SyncState::Done);
        assert!((result.progress - 1.0).abs() < f32::EPSILON);
        mock.assert_calls(1);

        // Verify products inserted
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM products_meta")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(count, 1);

        // Verify sync_state
        let status: String = sqlx::query_scalar(
            "SELECT status FROM sync_state WHERE source_id = 'test-source'",
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(status, "done");
    }

    // ── Test: HTTP error handling ───────────────────────────────────────────

    #[tokio::test]
    async fn sync_http_404_returns_network_error() {
        let server = MockServer::start();
        let mock = server.mock(|when, then| {
            when.method(GET).path("/missing.json");
            then.status(404);
        });

        let pool = setup_db().await;
        let client = reqwest::Client::new();
        let svc = CatalogSyncService::new(pool, client);
        let url = format!("{}/missing.json", server.base_url());

        let err = svc.sync_catalog(&url).await.unwrap_err();
        assert!(
            err.to_string().contains("HTTP 404"),
            "Expected HTTP 404 error, got: {err}"
        );
        mock.assert_calls(1);
    }

    #[tokio::test]
    async fn sync_http_503_returns_network_error() {
        let server = MockServer::start();
        let mock = server.mock(|when, then| {
            when.method(GET).path("/unavailable.json");
            then.status(503);
        });

        let pool = setup_db().await;
        let client = reqwest::Client::new();
        let svc = CatalogSyncService::new(pool, client);
        let url = format!("{}/unavailable.json", server.base_url());

        let err = svc.sync_catalog(&url).await.unwrap_err();
        assert!(
            err.to_string().contains("HTTP 503"),
            "Expected HTTP 503 error, got: {err}"
        );
        mock.assert_calls(1);
    }

    #[tokio::test]
    async fn sync_invalid_json_returns_invalid_input_error() {
        let server = MockServer::start();
        let mock = server.mock(|when, then| {
            when.method(GET).path("/bad.json");
            then.status(200)
                .header("Content-Type", "application/json")
                .body("this is not json");
        });

        let pool = setup_db().await;
        let client = reqwest::Client::new();
        let svc = CatalogSyncService::new(pool, client);
        let url = format!("{}/bad.json", server.base_url());

        let err = svc.sync_catalog(&url).await.unwrap_err();
        assert!(
            err.to_string().contains("Invalid catalog JSON"),
            "Expected Invalid catalog JSON error, got: {err}"
        );
        mock.assert_calls(1);
    }

    // ── Test: concurrent sync rejection ─────────────────────────────────────

    #[tokio::test]
    async fn sync_rejects_concurrent_request() {
        let server = MockServer::start();
        let body = sample_catalog_json("concurrent-test", &[single_product()]);
        let mock = server.mock(|when, then| {
            when.method(GET).path("/catalog.json");
            then.status(200)
                .header("Content-Type", "application/json")
                .body(body);
        });

        let pool = setup_db().await;
        let client = reqwest::Client::new();
        let svc = CatalogSyncService::new(pool.clone(), client);
        let url = format!("{}/catalog.json", server.base_url());

        // First sync succeeds
        let r1 = svc.sync_catalog(&url).await;
        assert!(r1.is_ok(), "first sync should succeed: {:?}", r1.err());
        mock.assert_calls(1);

        // Second sync with same source_id — the source_id is now `done`,
        // so concurrent check passes (it's not running). Let's test with
        // an artificially set 'downloading' state instead.
        let _ = sqlx::query("UPDATE sync_state SET status = 'downloading' WHERE source_id = 'concurrent-test'")
            .execute(&pool)
            .await;

        let r2 = svc.sync_catalog(&url).await;
        assert!(
            r2.is_err(),
            "second sync should be rejected when state is downloading"
        );
        let err = r2.unwrap_err();
        assert!(
            err.to_string().contains("sync already in progress"),
            "Expected SyncInProgress, got: {err}"
        );
    }

    // ── Test: upsert counting ───────────────────────────────────────────────

    #[tokio::test]
    async fn sync_counts_products_loaded_and_updated() {
        let server = MockServer::start();
        let products = vec![
            single_product(),
            serde_json::json!({
                "sku": "TEST-SKU-002",
                "name": "Second Product",
                "brand": "TestBrand",
                "model": "TM-200",
                "category": "Electric Guitars",
                "subcategory": "Solid Body",
                "price": 1499.99,
                "currency": "USD",
                "condition": "new",
                "availability": "in_stock",
                "url": "https://example.com/item2",
                "image_url": "https://example.com/img2.jpg",
                "seller": "Test Seller",
                "location": "USA"
            }),
        ];
        let body = sample_catalog_json("upsert-test", &products);
        let mock = server.mock(|when, then| {
            when.method(GET).path("/catalog.json");
            then.status(200)
                .header("Content-Type", "application/json")
                .body(body);
        });

        let pool = setup_db().await;
        let client = reqwest::Client::new();
        let svc = CatalogSyncService::new(pool.clone(), client);
        let url = format!("{}/catalog.json", server.base_url());

        let result = svc.sync_catalog(&url).await.expect("sync should succeed");
        assert_eq!(result.products_loaded, 2);
        assert_eq!(result.state, SyncState::Done);
        assert!((result.progress - 1.0).abs() < f32::EPSILON);

        // Re-sync the same data — upsert should still report loaded: 2
        let result2 = svc.sync_catalog(&url).await.expect("second sync should succeed");
        assert_eq!(result2.products_loaded, 2);
        assert_eq!(result2.state, SyncState::Done);

        mock.assert_calls(2);

        // Verify 2 rows in DB
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM products_meta")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(count, 2);
    }

    // ── Test: sync_state transitions visible in DB ──────────────────────────

    #[tokio::test]
    async fn sync_writes_state_transitions() {
        let server = MockServer::start();
        let body = sample_catalog_json("state-test", &[single_product()]);
        let mock = server.mock(|when, then| {
            when.method(GET).path("/catalog.json");
            then.status(200)
                .header("Content-Type", "application/json")
                .body(body);
        });

        let pool = setup_db().await;
        let client = reqwest::Client::new();
        let svc = CatalogSyncService::new(pool.clone(), client);
        let url = format!("{}/catalog.json", server.base_url());

        svc.sync_catalog(&url).await.expect("sync should succeed");
        mock.assert_calls(1);

        // Verify final state is 'done'
        let status: String = sqlx::query_scalar(
            "SELECT status FROM sync_state WHERE source_id = 'state-test'",
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(status, "done");

        // Verify last_synced is set (non-zero)
        let last_synced: i64 = sqlx::query_scalar(
            "SELECT last_synced FROM sync_state WHERE source_id = 'state-test'",
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert!(last_synced > 0, "last_synced should be a positive timestamp");
    }
}
