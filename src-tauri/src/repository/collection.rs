use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

/// Raw database row for `collection_items`.
#[derive(Debug, Clone, sqlx::FromRow, Serialize)]
pub struct CollectionItemRow {
    pub id: i64,
    pub sku: Option<String>,
    pub name: String,
    pub brand: Option<String>,
    pub purchase_price: Option<f64>,
    pub purchase_currency: String,
    pub purchase_date: Option<i64>,
    pub condition: Option<String>,
    pub serial_number: Option<String>,
    pub notes: Option<String>,
    pub image_url: Option<String>,
    pub added_at: i64,
}

/// Hydrated collection item with computed `estimated_value`.
#[derive(Debug, Clone, Serialize)]
pub struct CollectionItem {
    pub id: i64,
    pub sku: Option<String>,
    pub name: String,
    pub brand: Option<String>,
    pub purchase_price: Option<f64>,
    pub purchase_currency: String,
    pub purchase_date: Option<i64>,
    pub condition: String,
    pub serial_number: Option<String>,
    pub notes: Option<String>,
    pub image_url: Option<String>,
    pub added_at: i64,
    pub estimated_value: Option<f64>,
}

/// Input for adding a new collection item.
#[derive(Debug, Clone, Deserialize)]
pub struct CollectionItemInput {
    pub sku: Option<String>,
    pub name: String,
    pub brand: Option<String>,
    pub purchase_price: Option<f64>,
    pub purchase_currency: String,
    pub purchase_date: Option<i64>,
    pub condition: String,
    pub serial_number: Option<String>,
    pub notes: Option<String>,
    pub image_url: Option<String>,
}

/// Partial updates for an existing collection item.
#[derive(Debug, Clone, Deserialize, Default)]
pub struct CollectionItemUpdates {
    pub sku: Option<String>,
    pub name: Option<String>,
    pub brand: Option<String>,
    pub purchase_price: Option<f64>,
    pub purchase_currency: Option<String>,
    pub purchase_date: Option<i64>,
    pub condition: Option<String>,
    pub serial_number: Option<String>,
    pub notes: Option<String>,
    pub image_url: Option<String>,
}

/// Aggregated collection statistics.
#[derive(Debug, Clone, Serialize)]
pub struct CollectionStats {
    pub total_items: u32,
    pub total_value: f64,
    pub top_item_name: Option<String>,
    pub top_item_value: f64,
}

/// Repository for collection_items CRUD and value estimation.
#[derive(Debug, Clone)]
pub struct CollectionRepo {
    pool: SqlitePool,
}

impl CollectionRepo {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// Insert a new collection item and return its autoincrement id.
    pub async fn add(&self, input: &CollectionItemInput) -> Result<i64, sqlx::Error> {
        let added_at = epoch_seconds();
        let id: i64 = sqlx::query_scalar(
            "INSERT INTO collection_items
             (sku, name, brand, purchase_price, purchase_currency, purchase_date, condition, serial_number, notes, image_url, added_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
             RETURNING id",
        )
        .bind(&input.sku)
        .bind(&input.name)
        .bind(&input.brand)
        .bind(input.purchase_price)
        .bind(&input.purchase_currency)
        .bind(input.purchase_date)
        .bind(&input.condition)
        .bind(&input.serial_number)
        .bind(&input.notes)
        .bind(&input.image_url)
        .bind(added_at)
        .fetch_one(&self.pool)
        .await?;
        Ok(id)
    }

    /// Remove an item by id.
    pub async fn remove(&self, id: i64) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM collection_items WHERE id = ?1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    /// Get a single item by id, hydrated with estimated_value.
    pub async fn get_by_id(&self, id: i64) -> Result<Option<CollectionItem>, sqlx::Error> {
        let row: Option<CollectionItemRow> = sqlx::query_as(
            "SELECT * FROM collection_items WHERE id = ?1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        match row {
            Some(r) => {
                let estimated_value = if let Some(ref sku) = r.sku {
                    estimated_value(sku, &self.pool).await?
                } else {
                    None
                };
                Ok(Some(row_to_item(r, estimated_value)))
            }
            None => Ok(None),
        }
    }

    /// Get all collection items, hydrated with estimated_value.
    pub async fn get_all(&self) -> Result<Vec<CollectionItem>, sqlx::Error> {
        let rows: Vec<CollectionItemRow> =
            sqlx::query_as("SELECT * FROM collection_items ORDER BY added_at DESC")
                .fetch_all(&self.pool)
                .await?;

        let mut items = Vec::with_capacity(rows.len());
        for row in rows {
            let estimated_value = if let Some(ref sku) = row.sku {
                estimated_value(sku, &self.pool).await?
            } else {
                None
            };
            items.push(row_to_item(row, estimated_value));
        }
        Ok(items)
    }

    /// Partial update: only sets fields that are `Some`.
    pub async fn update(
        &self,
        id: i64,
        updates: &CollectionItemUpdates,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            "UPDATE collection_items SET
                sku               = COALESCE(?1, sku),
                name              = COALESCE(?2, name),
                brand             = COALESCE(?3, brand),
                purchase_price    = COALESCE(?4, purchase_price),
                purchase_currency = COALESCE(?5, purchase_currency),
                purchase_date     = COALESCE(?6, purchase_date),
                condition         = COALESCE(?7, condition),
                serial_number     = COALESCE(?8, serial_number),
                notes             = COALESCE(?9, notes),
                image_url         = COALESCE(?10, image_url)
             WHERE id = ?11",
        )
        .bind(&updates.sku)
        .bind(&updates.name)
        .bind(&updates.brand)
        .bind(updates.purchase_price)
        .bind(&updates.purchase_currency)
        .bind(updates.purchase_date)
        .bind(&updates.condition)
        .bind(&updates.serial_number)
        .bind(&updates.notes)
        .bind(&updates.image_url)
        .bind(id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// Compute aggregate statistics for the collection.
    pub async fn get_stats(&self) -> Result<CollectionStats, sqlx::Error> {
        let rows: Vec<CollectionItemRow> =
            sqlx::query_as("SELECT * FROM collection_items")
                .fetch_all(&self.pool)
                .await?;

        let total_items = rows.len() as u32;
        if total_items == 0 {
            return Ok(CollectionStats {
                total_items: 0,
                total_value: 0.0,
                top_item_name: None,
                top_item_value: 0.0,
            });
        }

        let mut total_value = 0.0;
        let mut top_item_name = None;
        let mut top_item_value = 0.0;

        for row in &rows {
            let ev = if let Some(ref sku) = row.sku {
                estimated_value(sku, &self.pool).await?.unwrap_or(0.0)
            } else {
                0.0
            };
            total_value += ev;
            if ev > top_item_value {
                top_item_value = ev;
                top_item_name = Some(row.name.clone());
            }
        }

        Ok(CollectionStats {
            total_items,
            total_value,
            top_item_name,
            top_item_value,
        })
    }
}

/// Estimate the current market value for a SKU.
///
/// 1. Average price from `price_history` in the last 90 days.
/// 2. Fallback to `products_meta.price`.
/// 3. Final fallback to `0.0`.
pub async fn estimated_value(sku: &str, pool: &SqlitePool) -> Result<Option<f64>, sqlx::Error> {
    let now = epoch_seconds();
    let window_90d = now - 90 * 86_400;

    let avg: Option<f64> = sqlx::query_scalar(
        "SELECT AVG(price) FROM price_history WHERE sku = ?1 AND recorded_at >= ?2",
    )
    .bind(sku)
    .bind(window_90d)
    .fetch_one(pool)
    .await?;

    if let Some(val) = avg {
        return Ok(Some(val));
    }

    let fallback: Option<f64> = sqlx::query_scalar("SELECT price FROM products_meta WHERE sku = ?1")
        .bind(sku)
        .fetch_optional(pool)
        .await?;

    Ok(fallback.or(Some(0.0)))
}

// ── Helpers ───────────────────────────────────────────────────────────────

fn epoch_seconds() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

fn row_to_item(row: CollectionItemRow, estimated_value: Option<f64>) -> CollectionItem {
    CollectionItem {
        id: row.id,
        sku: row.sku,
        name: row.name,
        brand: row.brand,
        purchase_price: row.purchase_price,
        purchase_currency: row.purchase_currency,
        purchase_date: row.purchase_date,
        condition: row.condition.unwrap_or_default(),
        serial_number: row.serial_number,
        notes: row.notes,
        image_url: row.image_url,
        added_at: row.added_at,
        estimated_value,
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repository::price_history::create_price_history_table;

    async fn make_memory_pool() -> SqlitePool {
        SqlitePool::connect("sqlite::memory:").await.unwrap()
    }

    async fn create_collection_items_table(pool: &SqlitePool) {
        sqlx::query(
            "CREATE TABLE collection_items (
                id                INTEGER PRIMARY KEY AUTOINCREMENT,
                sku               TEXT,
                name              TEXT NOT NULL,
                brand             TEXT,
                purchase_price    REAL,
                purchase_currency TEXT DEFAULT 'USD',
                purchase_date     INTEGER,
                condition         TEXT CHECK(condition IN ('mint','excellent','good','fair','poor')),
                serial_number     TEXT,
                notes             TEXT,
                image_url         TEXT,
                added_at          INTEGER NOT NULL
            )",
        )
        .execute(pool)
        .await
        .unwrap();

        sqlx::query(
            "CREATE INDEX idx_collection_items_sku ON collection_items(sku)",
        )
        .execute(pool)
        .await
        .unwrap();
    }

    async fn create_products_meta_table(pool: &SqlitePool) {
        sqlx::query(
            "CREATE TABLE products_meta (
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
                condition    TEXT,
                availability TEXT,
                url          TEXT NOT NULL,
                image_url    TEXT,
                seller       TEXT,
                location     TEXT,
                synced_at    INTEGER NOT NULL
            )",
        )
        .execute(pool)
        .await
        .unwrap();
    }

    fn sample_input(name: &str) -> CollectionItemInput {
        CollectionItemInput {
            sku: Some("SKU-001".to_string()),
            name: name.to_string(),
            brand: Some("Fender".to_string()),
            purchase_price: Some(1000.0),
            purchase_currency: "USD".to_string(),
            purchase_date: Some(1_700_000_000),
            condition: "excellent".to_string(),
            serial_number: Some("SN123".to_string()),
            notes: Some("Test note".to_string()),
            image_url: Some("https://example.com/img.jpg".to_string()),
        }
    }

    // ── add / roundtrip ─────────────────────────────────────────────────

    #[tokio::test]
    async fn add_returns_positive_id() {
        let pool = make_memory_pool().await;
        create_collection_items_table(&pool).await;
        let repo = CollectionRepo::new(pool);

        let id = repo.add(&sample_input("Stratocaster")).await.unwrap();
        assert!(id > 0, "add should return a positive id");
    }

    #[tokio::test]
    async fn add_then_get_by_id_roundtrip() {
        let pool = make_memory_pool().await;
        create_collection_items_table(&pool).await;
        create_price_history_table(&pool).await;
        create_products_meta_table(&pool).await;
        let repo = CollectionRepo::new(pool);

        let id = repo.add(&sample_input("Les Paul")).await.unwrap();
        let item = repo.get_by_id(id).await.unwrap().expect("item should exist");

        assert_eq!(item.name, "Les Paul");
        assert_eq!(item.sku, Some("SKU-001".to_string()));
        assert_eq!(item.brand, Some("Fender".to_string()));
        assert_eq!(item.purchase_price, Some(1000.0));
        assert_eq!(item.purchase_currency, "USD");
        assert_eq!(item.condition, "excellent");
        assert_eq!(item.serial_number, Some("SN123".to_string()));
    }

    // ── get_all ───────────────────────────────────────────────────────────

    #[tokio::test]
    async fn get_all_returns_empty_vec_when_table_empty() {
        let pool = make_memory_pool().await;
        create_collection_items_table(&pool).await;
        let repo = CollectionRepo::new(pool);

        let items = repo.get_all().await.unwrap();
        assert!(items.is_empty(), "expected empty vec for empty table");
    }

    #[tokio::test]
    async fn get_all_returns_items_in_desc_added_at_order() {
        let pool = make_memory_pool().await;
        create_collection_items_table(&pool).await;
        create_price_history_table(&pool).await;
        create_products_meta_table(&pool).await;
        let repo = CollectionRepo::new(pool);

        let mut input1 = sample_input("First");
        input1.sku = Some("SKU-A".to_string());
        let mut input2 = sample_input("Second");
        input2.sku = Some("SKU-B".to_string());

        repo.add(&input1).await.unwrap();
        // Need >=1s sleep because added_at is second-level granularity
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        repo.add(&input2).await.unwrap();

        let items = repo.get_all().await.unwrap();
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].name, "Second", "most recent first");
        assert_eq!(items[1].name, "First");
    }

    // ── remove ────────────────────────────────────────────────────────────

    #[tokio::test]
    async fn remove_deletes_item() {
        let pool = make_memory_pool().await;
        create_collection_items_table(&pool).await;
        create_price_history_table(&pool).await;
        create_products_meta_table(&pool).await;
        let repo = CollectionRepo::new(pool);

        let id = repo.add(&sample_input("ToDelete")).await.unwrap();
        assert!(repo.get_by_id(id).await.unwrap().is_some());

        repo.remove(id).await.unwrap();
        assert!(repo.get_by_id(id).await.unwrap().is_none());
    }

    #[tokio::test]
    async fn remove_nonexistent_id_is_noop() {
        let pool = make_memory_pool().await;
        create_collection_items_table(&pool).await;
        let repo = CollectionRepo::new(pool);

        repo.remove(9999).await.unwrap();
        let items = repo.get_all().await.unwrap();
        assert!(items.is_empty());
    }

    // ── update ────────────────────────────────────────────────────────────

    #[tokio::test]
    async fn update_changes_purchase_price() {
        let pool = make_memory_pool().await;
        create_collection_items_table(&pool).await;
        create_price_history_table(&pool).await;
        create_products_meta_table(&pool).await;
        let repo = CollectionRepo::new(pool);

        let id = repo.add(&sample_input("UpdateMe")).await.unwrap();
        let before = repo.get_by_id(id).await.unwrap().unwrap();
        assert_eq!(before.purchase_price, Some(1000.0));

        repo.update(
            id,
            &CollectionItemUpdates {
                purchase_price: Some(1200.0),
                ..Default::default()
            },
        )
        .await
        .unwrap();

        let after = repo.get_by_id(id).await.unwrap().unwrap();
        assert_eq!(after.purchase_price, Some(1200.0));
        assert_eq!(after.name, "UpdateMe", "other fields unchanged");
        assert_eq!(after.brand, Some("Fender".to_string()));
    }

    #[tokio::test]
    async fn update_does_not_affect_other_rows() {
        let pool = make_memory_pool().await;
        create_collection_items_table(&pool).await;
        create_price_history_table(&pool).await;
        create_products_meta_table(&pool).await;
        let repo = CollectionRepo::new(pool);

        let id1 = repo.add(&sample_input("A")).await.unwrap();
        let id2 = repo.add(&sample_input("B")).await.unwrap();

        repo.update(
            id1,
            &CollectionItemUpdates {
                purchase_price: Some(999.0),
                ..Default::default()
            },
        )
        .await
        .unwrap();

        let item2 = repo.get_by_id(id2).await.unwrap().unwrap();
        assert_eq!(item2.purchase_price, Some(1000.0), "item2 should be unchanged");
    }

    // ── estimated_value ───────────────────────────────────────────────────

    #[tokio::test]
    async fn estimated_value_from_price_history_avg() {
        let pool = make_memory_pool().await;
        create_price_history_table(&pool).await;
        let now = epoch_seconds();

        for price in [1000.0, 1100.0, 1200.0] {
            sqlx::query(
                "INSERT INTO price_history (sku, price, recorded_at, source_id) VALUES (?1, ?2, ?3, ?4)",
            )
            .bind("SKU-X")
            .bind(price)
            .bind(now - 10)
            .bind("reverb")
            .execute(&pool)
            .await
            .unwrap();
        }

        let val = estimated_value("SKU-X", &pool).await.unwrap();
        assert!(
            (val.unwrap() - 1100.0).abs() < 0.01,
            "expected avg ~1100, got {:?}",
            val
        );
    }

    #[tokio::test]
    async fn estimated_value_fallback_to_products_meta() {
        let pool = make_memory_pool().await;
        create_price_history_table(&pool).await;
        create_products_meta_table(&pool).await;
        let now = epoch_seconds();

        // Insert old price history (>90 days)
        sqlx::query(
            "INSERT INTO price_history (sku, price, recorded_at, source_id) VALUES (?1, ?2, ?3, ?4)",
        )
        .bind("SKU-Y")
        .bind(500.0)
        .bind(now - 100 * 86_400)
        .bind("reverb")
        .execute(&pool)
        .await
        .unwrap();

        sqlx::query(
            "INSERT INTO products_meta (sku, source_id, name, url, synced_at, price) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        )
        .bind("SKU-Y")
        .bind("reverb")
        .bind("Product Y")
        .bind("https://example.com/y")
        .bind(0i64)
        .bind(800.0)
        .execute(&pool)
        .await
        .unwrap();

        let val = estimated_value("SKU-Y", &pool).await.unwrap();
        assert!(
            (val.unwrap() - 800.0).abs() < 0.01,
            "expected fallback 800, got {:?}",
            val
        );
    }

    #[tokio::test]
    async fn estimated_value_zero_when_no_data() {
        let pool = make_memory_pool().await;
        create_price_history_table(&pool).await;
        create_products_meta_table(&pool).await;

        let val = estimated_value("SKU-Z", &pool).await.unwrap();
        assert_eq!(val, Some(0.0), "expected 0.0 fallback when no data exists");
    }

    // ── get_stats ─────────────────────────────────────────────────────────

    #[tokio::test]
    async fn get_stats_empty_collection() {
        let pool = make_memory_pool().await;
        create_collection_items_table(&pool).await;
        let repo = CollectionRepo::new(pool);

        let stats = repo.get_stats().await.unwrap();
        assert_eq!(stats.total_items, 0);
        assert_eq!(stats.total_value, 0.0);
        assert_eq!(stats.top_item_name, None);
        assert_eq!(stats.top_item_value, 0.0);
    }

    #[tokio::test]
    async fn get_stats_populated_collection() {
        let pool = make_memory_pool().await;
        create_collection_items_table(&pool).await;
        create_price_history_table(&pool).await;
        create_products_meta_table(&pool).await;
        let repo = CollectionRepo::new(pool);
        let now = epoch_seconds();

        // Item A: avg price history = 1000
        let mut input_a = sample_input("Strat A");
        input_a.sku = Some("SKU-A".to_string());
        repo.add(&input_a).await.unwrap();
        for price in [900.0, 1000.0, 1100.0] {
            sqlx::query(
                "INSERT INTO price_history (sku, price, recorded_at, source_id) VALUES (?1, ?2, ?3, ?4)",
            )
            .bind("SKU-A")
            .bind(price)
            .bind(now - 10)
            .bind("reverb")
            .execute(&repo.pool)
            .await
            .unwrap();
        }

        // Item B: avg price history = 2000
        let mut input_b = sample_input("Les Paul B");
        input_b.sku = Some("SKU-B".to_string());
        repo.add(&input_b).await.unwrap();
        for price in [1900.0, 2000.0, 2100.0] {
            sqlx::query(
                "INSERT INTO price_history (sku, price, recorded_at, source_id) VALUES (?1, ?2, ?3, ?4)",
            )
            .bind("SKU-B")
            .bind(price)
            .bind(now - 10)
            .bind("reverb")
            .execute(&repo.pool)
            .await
            .unwrap();
        }

        let stats = repo.get_stats().await.unwrap();
        assert_eq!(stats.total_items, 2);
        assert!((stats.total_value - 3000.0).abs() < 0.1, "expected total ~3000, got {}", stats.total_value);
        assert_eq!(stats.top_item_name, Some("Les Paul B".to_string()));
        assert!((stats.top_item_value - 2000.0).abs() < 0.1);
    }
}
