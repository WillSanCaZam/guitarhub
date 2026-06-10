// SPDX-License-Identifier: GPL-3.0-or-later

use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

/// Input for adding a new wishlist item.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct WishlistItemInput {
    pub sku: Option<String>,
    pub name: Option<String>,
    pub brand: Option<String>,
    pub price: Option<f64>,
    pub currency: Option<String>,
    pub image_url: Option<String>,
    pub product_url: Option<String>,
    pub notes: Option<String>,
}

/// A wishlist item as stored in the database.
#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct WishlistItem {
    pub id: i64,
    pub sku: Option<String>,
    pub name: Option<String>,
    pub brand: Option<String>,
    pub price: Option<f64>,
    pub currency: Option<String>,
    pub image_url: Option<String>,
    pub product_url: Option<String>,
    pub notes: Option<String>,
    pub added_at: Option<i64>,
}

/// Repository for wishlist CRUD operations.
#[derive(Debug, Clone)]
pub struct WishlistRepo {
    pool: SqlitePool,
}

impl WishlistRepo {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// Insert a new wishlist item and return its autoincrement id.
    pub async fn add(&self, input: &WishlistItemInput) -> Result<i64, sqlx::Error> {
        let added_at = epoch_seconds();
        let id: i64 = sqlx::query_scalar(
            "INSERT INTO wishlist (sku, name, brand, price, currency, image_url, product_url, notes, added_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
             RETURNING id",
        )
        .bind(&input.sku)
        .bind(&input.name)
        .bind(&input.brand)
        .bind(input.price)
        .bind(&input.currency)
        .bind(&input.image_url)
        .bind(&input.product_url)
        .bind(&input.notes)
        .bind(added_at)
        .fetch_one(&self.pool)
        .await?;
        Ok(id)
    }

    /// Remove a wishlist item by id. Idempotent — no error if id doesn't exist.
    pub async fn remove(&self, id: i64) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM wishlist WHERE id = ?1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    /// Get all wishlist items ordered by added_at descending (newest first).
    pub async fn get_all(&self) -> Result<Vec<WishlistItem>, sqlx::Error> {
        let items: Vec<WishlistItem> = sqlx::query_as(
            "SELECT id, sku, name, brand, price, currency, image_url, product_url, notes, added_at
             FROM wishlist ORDER BY added_at DESC",
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(items)
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────

fn epoch_seconds() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

/// Create the wishlist table in an in-memory database (for tests).
#[cfg(test)]
pub async fn create_wishlist_table(pool: &SqlitePool) {
    sqlx::query(
        "CREATE TABLE wishlist (
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
    .execute(pool)
    .await
    .unwrap();
}

// ── Tests ─────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    async fn make_memory_pool() -> SqlitePool {
        SqlitePool::connect("sqlite::memory:").await.unwrap()
    }

    fn sample_input() -> WishlistItemInput {
        WishlistItemInput {
            sku: Some("FENDER-TELE".to_string()),
            name: Some("Telecaster".to_string()),
            brand: Some("Fender".to_string()),
            price: Some(1500.0),
            currency: Some("USD".to_string()),
            image_url: Some("https://example.com/img.jpg".to_string()),
            product_url: None,
            notes: None,
        }
    }

    // ── add ────────────────────────────────────────────────────────────────

    #[tokio::test]
    async fn add_returns_positive_id() {
        let pool = make_memory_pool().await;
        create_wishlist_table(&pool).await;
        let repo = WishlistRepo::new(pool);

        let id = repo.add(&sample_input()).await.unwrap();
        assert!(id > 0, "add should return a positive id");
    }

    #[tokio::test]
    async fn add_inserts_all_fields() {
        let pool = make_memory_pool().await;
        create_wishlist_table(&pool).await;
        let repo = WishlistRepo::new(pool);

        let input = WishlistItemInput {
            sku: Some("FENDER-TELE".to_string()),
            name: Some("Telecaster".to_string()),
            brand: Some("Fender".to_string()),
            price: Some(1500.0),
            currency: Some("USD".to_string()),
            image_url: Some("https://example.com/img.jpg".to_string()),
            product_url: Some("https://example.com/product".to_string()),
            notes: Some("birthday present".to_string()),
        };

        repo.add(&input).await.unwrap();
        let items = repo.get_all().await.unwrap();
        assert_eq!(items.len(), 1);
        let item = &items[0];
        assert_eq!(item.sku, Some("FENDER-TELE".to_string()));
        assert_eq!(item.name, Some("Telecaster".to_string()));
        assert_eq!(item.brand, Some("Fender".to_string()));
        assert_eq!(item.price, Some(1500.0));
        assert_eq!(item.currency, Some("USD".to_string()));
        assert_eq!(item.image_url, Some("https://example.com/img.jpg".to_string()));
        assert_eq!(item.product_url, Some("https://example.com/product".to_string()));
        assert_eq!(item.notes, Some("birthday present".to_string()));
        assert!(item.added_at.is_some(), "added_at should be set");
    }

    // ── remove ─────────────────────────────────────────────────────────────

    #[tokio::test]
    async fn remove_deletes_item() {
        let pool = make_memory_pool().await;
        create_wishlist_table(&pool).await;
        let repo = WishlistRepo::new(pool);

        let id = repo.add(&sample_input()).await.unwrap();
        let items_before = repo.get_all().await.unwrap();
        assert_eq!(items_before.len(), 1);

        repo.remove(id).await.unwrap();
        let items_after = repo.get_all().await.unwrap();
        assert!(items_after.is_empty(), "item should be removed");
    }

    #[tokio::test]
    async fn remove_nonexistent_id_is_idempotent() {
        let pool = make_memory_pool().await;
        create_wishlist_table(&pool).await;
        let repo = WishlistRepo::new(pool);

        // Removing a nonexistent id should not error
        repo.remove(9999).await.unwrap();
        let items = repo.get_all().await.unwrap();
        assert!(items.is_empty(), "should still be empty after removing nonexistent id");
    }

    // ── get_all ────────────────────────────────────────────────────────────

    #[tokio::test]
    async fn get_all_returns_empty_vec_when_table_empty() {
        let pool = make_memory_pool().await;
        create_wishlist_table(&pool).await;
        let repo = WishlistRepo::new(pool);

        let items = repo.get_all().await.unwrap();
        assert!(items.is_empty(), "expected empty vec for empty wishlist");
    }

    #[tokio::test]
    async fn get_all_returns_items_ordered_by_added_at_desc() {
        let pool = make_memory_pool().await;
        create_wishlist_table(&pool).await;
        let repo = WishlistRepo::new(pool.clone());

        let input1 = WishlistItemInput {
            sku: Some("SKU-A".to_string()),
            name: Some("First".to_string()),
            ..Default::default()
        };
        repo.add(&input1).await.unwrap();

        // Sleep 1s because added_at has second-level granularity
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;

        let input2 = WishlistItemInput {
            sku: Some("SKU-B".to_string()),
            name: Some("Second".to_string()),
            ..Default::default()
        };
        repo.add(&input2).await.unwrap();

        let items = repo.get_all().await.unwrap();
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].name, Some("Second".to_string()), "most recent first");
        assert_eq!(items[1].name, Some("First".to_string()), "oldest last");
    }
}