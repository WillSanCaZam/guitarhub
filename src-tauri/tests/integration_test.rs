// SPDX-License-Identifier: GPL-3.0-or-later
//
// Integration tests for the GuitarHub service layer.
//
// These tests exercise the REAL service stack — sync, search, price-drop
// detection, and notification dispatch — against an ephemeral in-memory
// SQLite database. No mocks; every code path runs through the actual
// service and repository implementations.

use guitarhub_lib::domain::product::{CatalogFile, SearchFilters, SortOrder, SyncState};
use guitarhub_lib::repository::price_drop_notifications::PriceDropNotificationsRepo;
use guitarhub_lib::repository::price_history::PriceHistoryRepo;
use guitarhub_lib::services::alert_service::{AlertDispatcher, AppNotificationAlert};
use guitarhub_lib::services::search::FtsSearchService;
use guitarhub_lib::services::sync::CatalogSyncService;
use sqlx::sqlite::SqlitePoolOptions;
use sqlx::SqlitePool;
use std::io::Write;

// ── Test database helper ────────────────────────────────────────────────────

/// Create an ephemeral in-memory SQLite pool with all tables needed for
/// integration testing. Uses `max_connections(1)` so every query hits
/// the same in-memory database.
async fn setup_integration_db() -> SqlitePool {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .expect("in-memory pool");

    // schema_meta
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS schema_meta (
            key   TEXT PRIMARY KEY,
            value TEXT NOT NULL
        )",
    )
    .execute(&pool)
    .await
    .unwrap();

    // products_meta
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

    // products_fts (FTS5 virtual table)
    sqlx::query(
        "CREATE VIRTUAL TABLE IF NOT EXISTS products_fts USING fts5(
            sku, source_id, name, brand, model, category, subcategory, specs_json,
            tokenize = 'trigram',
            content = 'products_meta',
            content_rowid = 'rowid'
        )",
    )
    .execute(&pool)
    .await
    .unwrap();

    // FTS sync triggers
    sqlx::query(
        "CREATE TRIGGER IF NOT EXISTS products_fts_ai AFTER INSERT ON products_meta BEGIN
            INSERT INTO products_fts(rowid, sku, source_id, name, brand, model, category, subcategory, specs_json)
            VALUES (new.rowid, new.sku, new.source_id, new.name, new.brand, new.model, new.category, new.subcategory, new.specs_json);
        END",
    )
    .execute(&pool)
    .await
    .unwrap();

    sqlx::query(
        "CREATE TRIGGER IF NOT EXISTS products_fts_ad AFTER DELETE ON products_meta BEGIN
            INSERT INTO products_fts(products_fts, rowid, sku, source_id, name, brand, model, category, subcategory, specs_json)
            VALUES ('delete', old.rowid, old.sku, old.source_id, old.name, old.brand, old.model, old.category, old.subcategory, old.specs_json);
        END",
    )
    .execute(&pool)
    .await
    .unwrap();

    sqlx::query(
        "CREATE TRIGGER IF NOT EXISTS products_fts_au AFTER UPDATE ON products_meta BEGIN
            INSERT INTO products_fts(products_fts, rowid, sku, source_id, name, brand, model, category, subcategory, specs_json)
            VALUES ('delete', old.rowid, old.sku, old.source_id, old.name, old.brand, old.model, old.category, old.subcategory, old.specs_json);
            INSERT INTO products_fts(rowid, sku, source_id, name, brand, model, category, subcategory, specs_json)
            VALUES (new.rowid, new.sku, new.source_id, new.name, new.brand, new.model, new.category, new.subcategory, new.specs_json);
        END",
    )
    .execute(&pool)
    .await
    .unwrap();

    // user_id index (migration 012)
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_products_meta_user_id ON products_meta(user_id)",
    )
    .execute(&pool)
    .await
    .unwrap();

    // sync_state
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

    // price_history
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

    // price_drop_notifications
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

    // settings
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

/// Load the sample catalog fixture from `tests/fixtures/sample_catalog.json`.
fn load_sample_catalog() -> CatalogFile {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let path = format!("{manifest_dir}/tests/fixtures/sample_catalog.json");
    let content = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("failed to read fixture {path}: {e}"));
    serde_json::from_str(&content).expect("failed to parse sample_catalog.json")
}

/// Write a `CatalogFile` to a temporary file and return the path.
fn catalog_to_tempfile(catalog: &CatalogFile) -> tempfile::NamedTempFile {
    let json = serde_json::to_string(catalog).expect("serialize catalog");
    let mut tmp = tempfile::NamedTempFile::new().expect("create tempfile");
    write!(tmp, "{json}").expect("write catalog to tempfile");
    tmp
}

// ── Test 1: Sync → Upsert cycle ─────────────────────────────────────────────

/// Load the sample catalog fixture (3 products), sync via `sync_local_catalog`,
/// and verify all 3 products are persisted in `products_meta`.
#[tokio::test]
async fn test_sync_upsert_products() {
    let pool = setup_integration_db().await;
    let svc = CatalogSyncService::new(pool.clone(), reqwest::Client::new());

    let catalog = load_sample_catalog();
    assert_eq!(catalog.products.len(), 3, "fixture must have 3 products");

    let tmp = catalog_to_tempfile(&catalog);
    let result = svc
        .sync_local_catalog(tmp.path().to_str().unwrap())
        .await
        .expect("sync_local_catalog must succeed");

    // Verify sync result
    assert_eq!(result.source_id, "reverb");
    assert_eq!(result.products_loaded, 3);
    assert_eq!(result.state, SyncState::Done);
    assert!((result.progress - 1.0).abs() < f32::EPSILON);

    // Verify products are in the database
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM products_meta")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(count, 3, "expected 3 products in DB after sync");

    // Verify specific SKUs exist
    let skus: Vec<String> = sqlx::query_scalar("SELECT sku FROM products_meta ORDER BY sku")
        .fetch_all(&pool)
        .await
        .unwrap();
    assert!(skus.contains(&"FENDER-STRAT-001".to_string()));
    assert!(skus.contains(&"GIBSON-LP-001".to_string()));
    assert!(skus.contains(&"PRS-C24-001".to_string()));

    // Verify price history rows were created (one per product)
    let history_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM price_history")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(history_count, 3, "expected 3 price_history rows");

    // Verify sync_state is 'done'
    let status: String = sqlx::query_scalar(
        "SELECT status FROM sync_state WHERE source_id = 'reverb'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(status, "done");
}

// ── Test 2: Search by brand ─────────────────────────────────────────────────

/// Upsert fixture data, then search for "Fender" and verify the correct
/// product is returned via the FTS5 search service.
#[tokio::test]
async fn test_search_by_brand() {
    let pool = setup_integration_db().await;
    let sync_svc = CatalogSyncService::new(pool.clone(), reqwest::Client::new());

    // Sync the fixture catalog
    let catalog = load_sample_catalog();
    let tmp = catalog_to_tempfile(&catalog);
    sync_svc
        .sync_local_catalog(tmp.path().to_str().unwrap())
        .await
        .expect("sync must succeed");

    // Search for "Fender" using the real FTS5 search service
    let search_svc = FtsSearchService::new(pool.clone());
    let filters = SearchFilters::default();
    let result = search_svc
        .search("Fender", &filters, SortOrder::Relevance, 1, 20)
        .await
        .expect("search must succeed");

    assert_eq!(result.total, 1, "expected 1 Fender product");
    assert_eq!(result.products.len(), 1);
    assert_eq!(result.products[0].sku, "FENDER-STRAT-001");
    assert_eq!(result.products[0].brand, "Fender");
    assert!(
        result.products[0].name.contains("Stratocaster"),
        "expected Stratocaster in name, got: {}",
        result.products[0].name
    );

    // Search for "Gibson" — should find the Les Paul
    let result_gibson = search_svc
        .search("Gibson", &filters, SortOrder::Relevance, 1, 20)
        .await
        .expect("search must succeed");
    assert_eq!(result_gibson.total, 1, "expected 1 Gibson product");
    assert_eq!(result_gibson.products[0].sku, "GIBSON-LP-001");

    // Search for a term that matches all products (e.g., "Electric" in category)
    // The FTS5 trigram index covers all text columns including category
    let result_all = search_svc
        .search("Electric", &filters, SortOrder::Relevance, 1, 20)
        .await
        .expect("search must succeed");
    assert_eq!(
        result_all.total, 3,
        "expected all 3 products to match 'Electric' in category"
    );
}

// ── Test 3: Price drop detection ────────────────────────────────────────────

/// Seed price_history with high prices, then sync a catalog with lower prices.
/// The sync service must detect price drops via the real `is_price_drop` pipeline.
#[tokio::test]
async fn test_price_drop_detection() {
    let pool = setup_integration_db().await;
    let price_history = PriceHistoryRepo::new(pool.clone());
    let svc = CatalogSyncService::new(pool.clone(), reqwest::Client::new());

    // Seed: write high prices for all 3 fixture SKUs, recorded 1 hour ago.
    let one_hour_ago = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
        - 3600;

    // Fender: $1599.99 → will drop to $1299.99 (18.75% drop, fires relative threshold)
    price_history
        .record_price("FENDER-STRAT-001", 1599.99, "reverb", one_hour_ago)
        .await
        .unwrap();

    // Gibson: $2499.99 → will drop to $2199.99 (12% drop, fires relative threshold)
    price_history
        .record_price("GIBSON-LP-001", 2499.99, "reverb", one_hour_ago)
        .await
        .unwrap();

    // PRS: $3299.99 → stays the same (no drop)
    price_history
        .record_price("PRS-C24-001", 3299.99, "reverb", one_hour_ago)
        .await
        .unwrap();

    // Build a modified catalog with lower prices for Fender and Gibson
    let mut catalog = load_sample_catalog();
    catalog.products[0].price = 1299.99; // Fender: 1599.99 → 1299.99 (18.75% drop)
    catalog.products[1].price = 2199.99; // Gibson: 2499.99 → 2199.99 (12% drop)
    // PRS stays at 3299.99 — no drop

    let tmp = catalog_to_tempfile(&catalog);
    let result = svc
        .sync_local_catalog(tmp.path().to_str().unwrap())
        .await
        .expect("sync must succeed");

    // Verify drops detected
    assert_eq!(
        result.drops.len(),
        2,
        "expected 2 price drops (Fender + Gibson), got {}: {:?}",
        result.drops.len(),
        result.drops
    );

    // Verify Fender drop
    let fender_drop = result
        .drops
        .iter()
        .find(|d| d.sku == "FENDER-STRAT-001")
        .expect("Fender drop must exist");
    assert!(
        (fender_drop.previous_price - 1599.99).abs() < 0.01,
        "Fender previous_price should be 1599.99, got {}",
        fender_drop.previous_price
    );
    assert!(
        (fender_drop.new_price - 1299.99).abs() < 0.01,
        "Fender new_price should be 1299.99, got {}",
        fender_drop.new_price
    );

    // Verify Gibson drop
    let gibson_drop = result
        .drops
        .iter()
        .find(|d| d.sku == "GIBSON-LP-001")
        .expect("Gibson drop must exist");
    assert!(
        (gibson_drop.previous_price - 2499.99).abs() < 0.01,
        "Gibson previous_price should be 2499.99, got {}",
        gibson_drop.previous_price
    );
    assert!(
        (gibson_drop.new_price - 2199.99).abs() < 0.01,
        "Gibson new_price should be 2199.99, got {}",
        gibson_drop.new_price
    );

    // Verify PRS did NOT produce a drop (price unchanged)
    assert!(
        !result.drops.iter().any(|d| d.sku == "PRS-C24-001"),
        "PRS must not produce a drop when price is unchanged"
    );

    // Verify price_history now has 6 rows (3 seed + 3 new)
    let history_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM price_history")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(history_count, 6, "expected 6 price_history rows (3 seed + 3 new)");
}

// ── Test 4: Notification dispatch on price drop ─────────────────────────────

/// Chain: sync → price drop detection → alert dispatch.
/// Verify that detected price drops can be dispatched through the
/// `AlertDispatcher` trait and that notification records are created
/// in the `price_drop_notifications` table.
#[tokio::test]
async fn test_notification_on_price_drop() {
    let pool = setup_integration_db().await;
    let price_history = PriceHistoryRepo::new(pool.clone());
    let notification_repo = PriceDropNotificationsRepo::new(pool.clone());
    let svc = CatalogSyncService::new(pool.clone(), reqwest::Client::new());

    // Seed: write a high price for Fender, 1 hour ago
    let one_hour_ago = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
        - 3600;

    price_history
        .record_price("FENDER-STRAT-001", 1599.99, "reverb", one_hour_ago)
        .await
        .unwrap();

    // Sync with a lower price — triggers a drop
    let mut catalog = load_sample_catalog();
    catalog.products[0].price = 1299.99; // 18.75% drop

    let tmp = catalog_to_tempfile(&catalog);
    let result = svc
        .sync_local_catalog(tmp.path().to_str().unwrap())
        .await
        .expect("sync must succeed");

    // Verify at least 1 drop detected
    assert!(
        !result.drops.is_empty(),
        "expected at least 1 price drop"
    );

    let fender_drop = result
        .drops
        .iter()
        .find(|d| d.sku == "FENDER-STRAT-001")
        .expect("Fender drop must exist");

    // Dispatch the drop through the AppNotificationAlert dispatcher
    let alert = AppNotificationAlert;
    let client = reqwest::Client::new();
    let title = format!("Price Drop: {}", fender_drop.sku);
    let message = format!(
        "Dropped from ${:.2} to ${:.2}",
        fender_drop.previous_price, fender_drop.new_price
    );

    let send_result = alert.send(&title, &message, &client).await;
    assert!(send_result.is_ok(), "alert dispatch must succeed");

    // Record the notification in the cooldown table (simulating what
    // sync_command::dispatch_drops does in production)
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;

    notification_repo
        .upsert(
            &fender_drop.sku,
            now,
            fender_drop.new_price,
            &fender_drop.channel,
        )
        .await
        .expect("notification upsert must succeed");

    // Verify the notification record exists in the database
    let last_notified = notification_repo
        .get_last_notified("FENDER-STRAT-001")
        .await
        .expect("get_last_notified must succeed");
    assert!(
        last_notified.is_some(),
        "notification record must exist after dispatch"
    );
    assert_eq!(last_notified.unwrap(), now);

    // Verify cooldown: a second sync with the same drop should NOT produce
    // a new drop because the SKU is now in cooldown.
    // Re-seed the price history with the old price so the detector sees
    // a drop again.
    price_history
        .record_price("FENDER-STRAT-001", 1599.99, "reverb", now - 10)
        .await
        .unwrap();

    // Sync again with the same low price
    let tmp2 = catalog_to_tempfile(&catalog);
    let result2 = svc
        .sync_local_catalog(tmp2.path().to_str().unwrap())
        .await
        .expect("second sync must succeed");

    // The Fender drop should be suppressed by the cooldown gate
    let fender_drops: Vec<_> = result2
        .drops
        .iter()
        .filter(|d| d.sku == "FENDER-STRAT-001")
        .collect();
    assert!(
        fender_drops.is_empty(),
        "Fender drop must be suppressed by cooldown, got {} drops",
        fender_drops.len()
    );
}
