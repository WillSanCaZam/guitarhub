// SPDX-License-Identifier: GPL-3.0-or-later

use crate::AppError;
use crate::AppState;
use tauri::State;

/// Core logic for `get_total_products`, extracted for testability.
pub async fn get_total_products_cmd(pool: &sqlx::SqlitePool) -> Result<u32, AppError> {
    let count: u32 = sqlx::query_scalar("SELECT COUNT(*) FROM products_meta")
        .fetch_one(pool)
        .await?;
    Ok(count)
}

/// Return the total number of products in the catalog.
#[tauri::command]
pub async fn get_total_products(state: State<'_, AppState>) -> Result<u32, AppError> {
    get_total_products_cmd(&state.pool).await
}

/// Core logic for `get_wishlist_count`, extracted for testability.
pub async fn get_wishlist_count_cmd(pool: &sqlx::SqlitePool) -> Result<u32, AppError> {
    let count: u32 = sqlx::query_scalar("SELECT COUNT(*) FROM wishlist")
        .fetch_one(pool)
        .await?;
    Ok(count)
}

/// Return the total number of items in the wishlist.
#[tauri::command]
pub async fn get_wishlist_count(state: State<'_, AppState>) -> Result<u32, AppError> {
    get_wishlist_count_cmd(&state.pool).await
}

/// Core logic for `get_recent_searches`, extracted for testability.
pub async fn get_recent_searches_cmd(pool: &sqlx::SqlitePool) -> Result<Vec<String>, AppError> {
    let rows: Vec<String> = sqlx::query_scalar(
        "SELECT query FROM recent_searches ORDER BY searched_at DESC LIMIT 10",
    )
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

/// Core logic for `record_search`, extracted for testability.
pub async fn record_search_cmd(pool: &sqlx::SqlitePool, query: &str) -> Result<(), AppError> {
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
    .execute(pool)
    .await?;
    Ok(())
}

/// Return recent search queries.
#[tauri::command]
pub async fn get_recent_searches(state: State<'_, AppState>) -> Result<Vec<String>, AppError> {
    get_recent_searches_cmd(&state.pool).await
}

/// Record a search query for recent-searches tracking.
#[tauri::command]
pub async fn record_search(query: String, state: State<'_, AppState>) -> Result<(), AppError> {
    record_search_cmd(&state.pool, &query).await
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Create an in-memory pool with the tables needed for dashboard counts.
    async fn memory_pool() -> sqlx::SqlitePool {
        let pool = sqlx::SqlitePool::connect("sqlite::memory:")
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
        let pool = memory_pool().await;
        let count = get_total_products_cmd(&pool).await.unwrap();
        assert_eq!(count, 0);
    }

    #[tokio::test]
    async fn get_total_products_with_rows_returns_count() {
        let pool = memory_pool().await;
        for i in 0..5 {
            sqlx::query(
                "INSERT INTO products_meta (sku, source_id, url, synced_at)
                 VALUES (?1, ?2, ?3, ?4)",
            )
            .bind(format!("SKU-{i}"))
            .bind("test")
            .bind("https://example.com")
            .bind(0i64)
            .execute(&pool)
            .await
            .unwrap();
        }
        let count = get_total_products_cmd(&pool).await.unwrap();
        assert_eq!(count, 5);
    }

    #[tokio::test]
    async fn get_wishlist_count_empty_table_returns_zero() {
        let pool = memory_pool().await;
        let count = get_wishlist_count_cmd(&pool).await.unwrap();
        assert_eq!(count, 0);
    }

    #[tokio::test]
    async fn get_wishlist_count_with_rows_returns_count() {
        let pool = memory_pool().await;
        for i in 0..3 {
            sqlx::query(
                "INSERT INTO wishlist (sku, added_at) VALUES (?1, ?2)",
            )
            .bind(format!("WISH-{i}"))
            .bind(0i64)
            .execute(&pool)
            .await
            .unwrap();
        }
        let count = get_wishlist_count_cmd(&pool).await.unwrap();
        assert_eq!(count, 3);
    }

    #[tokio::test]
    async fn get_total_products_large_count() {
        let pool = memory_pool().await;
        for i in 0..100 {
            sqlx::query(
                "INSERT INTO products_meta (sku, source_id, url, synced_at)
                 VALUES (?1, ?2, ?3, ?4)",
            )
            .bind(format!("SKU-{i}"))
            .bind("test")
            .bind("https://example.com")
            .bind(0i64)
            .execute(&pool)
            .await
            .unwrap();
        }
        let count = get_total_products_cmd(&pool).await.unwrap();
        assert_eq!(count, 100);
    }

    #[tokio::test]
    async fn get_wishlist_count_after_delete() {
        let pool = memory_pool().await;
        sqlx::query("INSERT INTO wishlist (sku, added_at) VALUES ('A', 0)")
            .execute(&pool)
            .await
            .unwrap();
        sqlx::query("INSERT INTO wishlist (sku, added_at) VALUES ('B', 0)")
            .execute(&pool)
            .await
            .unwrap();
        assert_eq!(get_wishlist_count_cmd(&pool).await.unwrap(), 2);

        sqlx::query("DELETE FROM wishlist WHERE sku = 'A'")
            .execute(&pool)
            .await
            .unwrap();
        assert_eq!(get_wishlist_count_cmd(&pool).await.unwrap(), 1);
    }

    #[tokio::test]
    async fn get_recent_searches_empty_table_returns_empty_vec() {
        let pool = memory_pool().await;
        let result = get_recent_searches_cmd(&pool).await.unwrap();
        assert!(result.is_empty());
    }

    #[tokio::test]
    async fn record_search_then_get_recent_searches_returns_query() {
        let pool = memory_pool().await;
        record_search_cmd(&pool, "fender stratocaster").await.unwrap();
        let result = get_recent_searches_cmd(&pool).await.unwrap();
        assert_eq!(result, vec!["fender stratocaster"]);
    }

    #[tokio::test]
    async fn record_search_updates_existing_query() {
        let pool = memory_pool().await;
        record_search_cmd(&pool, "gibson les paul").await.unwrap();
        // Re-recording the same query should not duplicate it
        record_search_cmd(&pool, "gibson les paul").await.unwrap();
        let result = get_recent_searches_cmd(&pool).await.unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], "gibson les paul");
    }

    #[tokio::test]
    async fn get_recent_searches_limits_to_ten() {
        let pool = memory_pool().await;
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
            .execute(&pool)
            .await
            .unwrap();
        }
        let result = get_recent_searches_cmd(&pool).await.unwrap();
        assert_eq!(result.len(), 10);
        // query-14 has the largest timestamp so it should be first in DESC order
        assert_eq!(result[0], "query-14");
        assert_eq!(result[9], "query-5");
    }

    #[tokio::test]
    async fn get_recent_searches_orders_by_searched_at_desc() {
        let pool = memory_pool().await;
        let base = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;
        sqlx::query(
            "INSERT INTO recent_searches (query, searched_at) VALUES (?1, ?2)",
        )
        .bind("older")
        .bind(base)
        .execute(&pool)
        .await
        .unwrap();
        sqlx::query(
            "INSERT INTO recent_searches (query, searched_at) VALUES (?1, ?2)",
        )
        .bind("newer")
        .bind(base + 1)
        .execute(&pool)
        .await
        .unwrap();
        let result = get_recent_searches_cmd(&pool).await.unwrap();
        assert_eq!(result, vec!["newer", "older"]);
    }
}
