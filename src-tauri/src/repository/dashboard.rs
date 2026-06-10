// SPDX-License-Identifier: GPL-3.0-or-later

use sqlx::SqlitePool;

use crate::AppError;

/// Repository for dashboard aggregate queries.
///
/// Centralises the COUNT and recent-searches queries that were previously
/// inlined in `dashboard_command.rs`, bringing them in line with the
/// hexagonal architecture used by the rest of the codebase.
#[derive(Debug, Clone)]
pub struct DashboardRepo {
    pool: SqlitePool,
}

impl DashboardRepo {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// Return the total number of products in the catalog.
    pub async fn get_total_products(&self) -> Result<u32, AppError> {
        let count: u32 = sqlx::query_scalar("SELECT COUNT(*) FROM products_meta")
            .fetch_one(&self.pool)
            .await?;
        Ok(count)
    }

    /// Return the total number of items in the wishlist.
    pub async fn get_wishlist_count(&self) -> Result<u32, AppError> {
        let count: u32 = sqlx::query_scalar("SELECT COUNT(*) FROM wishlist")
            .fetch_one(&self.pool)
            .await?;
        Ok(count)
    }

    /// Return the most recent search queries (max 10, newest first).
    pub async fn get_recent_searches(&self) -> Result<Vec<String>, AppError> {
        let rows: Vec<String> = sqlx::query_scalar(
            "SELECT query FROM recent_searches ORDER BY searched_at DESC LIMIT 10",
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows)
    }

    /// Record or update a search query's timestamp.
    pub async fn record_search(&self, query: &str) -> Result<(), AppError> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;
        sqlx::query(
            "INSERT INTO recent_searches (query, searched_at) VALUES (?1, ?2)
             ON CONFLICT(query) DO UPDATE SET searched_at = ?2",
        )
        .bind(query)
        .bind(now)
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Create an in-memory pool with the tables needed for dashboard queries.
    async fn memory_pool() -> SqlitePool {
        let pool = SqlitePool::connect("sqlite::memory:")
            .await
            .unwrap();
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
        .execute(&pool)
        .await
        .unwrap();
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
        .execute(&pool)
        .await
        .unwrap();
        sqlx::query(
            "CREATE TABLE recent_searches (
                query       TEXT PRIMARY KEY,
                searched_at INTEGER NOT NULL
            )",
        )
        .execute(&pool)
        .await
        .unwrap();
        pool
    }

    #[tokio::test]
    async fn get_total_products_empty_table_returns_zero() {
        let repo = DashboardRepo::new(memory_pool().await);
        assert_eq!(repo.get_total_products().await.unwrap(), 0);
    }

    #[tokio::test]
    async fn get_total_products_with_rows_returns_count() {
        let repo = DashboardRepo::new(memory_pool().await);
        for i in 0..5 {
            sqlx::query(
                "INSERT INTO products_meta (sku, source_id, url, synced_at)
                 VALUES (?1, ?2, ?3, ?4)",
            )
            .bind(format!("SKU-{i}"))
            .bind("test")
            .bind("https://example.com")
            .bind(0i64)
            .execute(&repo.pool)
            .await
            .unwrap();
        }
        assert_eq!(repo.get_total_products().await.unwrap(), 5);
    }

    #[tokio::test]
    async fn get_total_products_large_count() {
        let repo = DashboardRepo::new(memory_pool().await);
        for i in 0..100 {
            sqlx::query(
                "INSERT INTO products_meta (sku, source_id, url, synced_at)
                 VALUES (?1, ?2, ?3, ?4)",
            )
            .bind(format!("SKU-{i}"))
            .bind("test")
            .bind("https://example.com")
            .bind(0i64)
            .execute(&repo.pool)
            .await
            .unwrap();
        }
        assert_eq!(repo.get_total_products().await.unwrap(), 100);
    }

    #[tokio::test]
    async fn get_wishlist_count_empty_table_returns_zero() {
        let repo = DashboardRepo::new(memory_pool().await);
        assert_eq!(repo.get_wishlist_count().await.unwrap(), 0);
    }

    #[tokio::test]
    async fn get_wishlist_count_with_rows_returns_count() {
        let repo = DashboardRepo::new(memory_pool().await);
        for i in 0..3 {
            sqlx::query("INSERT INTO wishlist (sku, added_at) VALUES (?1, ?2)")
                .bind(format!("WISH-{i}"))
                .bind(0i64)
                .execute(&repo.pool)
                .await
                .unwrap();
        }
        assert_eq!(repo.get_wishlist_count().await.unwrap(), 3);
    }

    #[tokio::test]
    async fn get_wishlist_count_after_delete() {
        let repo = DashboardRepo::new(memory_pool().await);
        sqlx::query("INSERT INTO wishlist (sku, added_at) VALUES ('A', 0)")
            .execute(&repo.pool)
            .await
            .unwrap();
        sqlx::query("INSERT INTO wishlist (sku, added_at) VALUES ('B', 0)")
            .execute(&repo.pool)
            .await
            .unwrap();
        assert_eq!(repo.get_wishlist_count().await.unwrap(), 2);

        sqlx::query("DELETE FROM wishlist WHERE sku = 'A'")
            .execute(&repo.pool)
            .await
            .unwrap();
        assert_eq!(repo.get_wishlist_count().await.unwrap(), 1);
    }

    #[tokio::test]
    async fn get_recent_searches_empty_table_returns_empty_vec() {
        let repo = DashboardRepo::new(memory_pool().await);
        let result = repo.get_recent_searches().await.unwrap();
        assert!(result.is_empty());
    }

    #[tokio::test]
    async fn record_search_then_get_recent_searches_returns_query() {
        let repo = DashboardRepo::new(memory_pool().await);
        repo.record_search("fender stratocaster").await.unwrap();
        let result = repo.get_recent_searches().await.unwrap();
        assert_eq!(result, vec!["fender stratocaster"]);
    }

    #[tokio::test]
    async fn record_search_updates_existing_query() {
        let repo = DashboardRepo::new(memory_pool().await);
        repo.record_search("gibson les paul").await.unwrap();
        repo.record_search("gibson les paul").await.unwrap();
        let result = repo.get_recent_searches().await.unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], "gibson les paul");
    }

    #[tokio::test]
    async fn get_recent_searches_limits_to_ten() {
        let repo = DashboardRepo::new(memory_pool().await);
        let base = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;
        for i in 0..15 {
            sqlx::query(
                "INSERT INTO recent_searches (query, searched_at) VALUES (?1, ?2)",
            )
            .bind(format!("query-{i}"))
            .bind(base + i)
            .execute(&repo.pool)
            .await
            .unwrap();
        }
        let result = repo.get_recent_searches().await.unwrap();
        assert_eq!(result.len(), 10);
        assert_eq!(result[0], "query-14");
        assert_eq!(result[9], "query-5");
    }

    #[tokio::test]
    async fn get_recent_searches_orders_by_searched_at_desc() {
        let repo = DashboardRepo::new(memory_pool().await);
        let base = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;
        sqlx::query(
            "INSERT INTO recent_searches (query, searched_at) VALUES (?1, ?2)",
        )
        .bind("older")
        .bind(base)
        .execute(&repo.pool)
        .await
        .unwrap();
        sqlx::query(
            "INSERT INTO recent_searches (query, searched_at) VALUES (?1, ?2)",
        )
        .bind("newer")
        .bind(base + 1)
        .execute(&repo.pool)
        .await
        .unwrap();
        let result = repo.get_recent_searches().await.unwrap();
        assert_eq!(result, vec!["newer", "older"]);
    }
}
