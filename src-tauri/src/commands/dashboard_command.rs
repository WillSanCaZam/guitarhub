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

/// Return recent search queries (placeholder — localStorage-backed in frontend).
#[tauri::command]
pub async fn get_recent_searches() -> Result<Vec<String>, AppError> {
    Ok(vec![])
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Create an in-memory pool with the two tables needed for dashboard counts.
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
    async fn get_recent_searches_returns_empty_vec() {
        let result = get_recent_searches().await.unwrap();
        assert!(result.is_empty());
    }
}
