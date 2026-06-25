// SPDX-License-Identifier: GPL-3.0-or-later

use tauri::State;

use crate::repository::collection::{
    CollectionItem, CollectionItemInput, CollectionItemUpdates, CollectionRepo, CollectionStats,
};
use crate::AppError;
use crate::AppState;

// ── Core logic (extracted for testability without Tauri runtime) ─────────

pub async fn add_to_collection_cmd(
    pool: &sqlx::SqlitePool,
    input: CollectionItemInput,
) -> Result<i64, AppError> {
    let repo = CollectionRepo::new(pool.clone());
    let id = repo.add(&input).await.map_err(|e| AppError::Database(e.to_string()))?;
    Ok(id)
}

pub async fn remove_from_collection_cmd(
    pool: &sqlx::SqlitePool,
    id: i64,
) -> Result<(), AppError> {
    let repo = CollectionRepo::new(pool.clone());
    repo.remove(id).await.map_err(|e| AppError::Database(e.to_string()))?;
    Ok(())
}

pub async fn get_collection_cmd(
    pool: &sqlx::SqlitePool,
) -> Result<Vec<CollectionItem>, AppError> {
    let repo = CollectionRepo::new(pool.clone());
    let items = repo.get_all().await.map_err(|e| AppError::Database(e.to_string()))?;
    Ok(items)
}

pub async fn update_collection_item_cmd(
    pool: &sqlx::SqlitePool,
    id: i64,
    updates: CollectionItemUpdates,
) -> Result<(), AppError> {
    let repo = CollectionRepo::new(pool.clone());
    repo.update(id, &updates)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
    Ok(())
}

pub async fn get_collection_stats_cmd(
    pool: &sqlx::SqlitePool,
) -> Result<CollectionStats, AppError> {
    let repo = CollectionRepo::new(pool.clone());
    let stats = repo.get_stats().await.map_err(|e| AppError::Database(e.to_string()))?;
    Ok(stats)
}

// ── Tauri IPC Commands ────────────────────────────────────────────────────

#[tauri::command]
pub async fn add_to_collection(
    input: CollectionItemInput,
    state: State<'_, AppState>,
) -> Result<i64, AppError> {
    add_to_collection_cmd(&state.pool, input).await
}

#[tauri::command]
pub async fn remove_from_collection(
    id: i64,
    state: State<'_, AppState>,
) -> Result<(), AppError> {
    remove_from_collection_cmd(&state.pool, id).await
}

#[tauri::command]
pub async fn get_collection(
    state: State<'_, AppState>,
) -> Result<Vec<CollectionItem>, AppError> {
    get_collection_cmd(&state.pool).await
}

#[tauri::command]
pub async fn update_collection_item(
    id: i64,
    updates: CollectionItemUpdates,
    state: State<'_, AppState>,
) -> Result<(), AppError> {
    update_collection_item_cmd(&state.pool, id, updates).await
}

#[tauri::command]
pub async fn get_collection_stats(
    state: State<'_, AppState>,
) -> Result<CollectionStats, AppError> {
    get_collection_stats_cmd(&state.pool).await
}

// ── Tests ─────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repository::collection::{
        CollectionItemInput, CollectionItemUpdates,
    };
    use crate::repository::price_history::create_price_history_table;

    async fn memory_pool() -> sqlx::SqlitePool {
        sqlx::SqlitePool::connect("sqlite::memory:").await.unwrap()
    }

    async fn create_collection_items_table(pool: &sqlx::SqlitePool) {
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
    }

    async fn create_products_meta_table(pool: &sqlx::SqlitePool) {
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
                synced_at    INTEGER NOT NULL,
                is_active    INTEGER DEFAULT 1,
                delisted_at  INTEGER
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

    // ── add_to_collection_cmd ─────────────────────────────────────────────

    #[tokio::test]
    async fn add_to_collection_cmd_returns_positive_id() {
        let pool = memory_pool().await;
        create_collection_items_table(&pool).await;
        create_price_history_table(&pool).await;
        create_products_meta_table(&pool).await;

        let id = add_to_collection_cmd(&pool, sample_input("Strat")).await.unwrap();
        assert!(id > 0);
    }

    // ── remove_from_collection_cmd ──────────────────────────────────────

    #[tokio::test]
    async fn remove_from_collection_cmd_deletes_item() {
        let pool = memory_pool().await;
        create_collection_items_table(&pool).await;
        create_price_history_table(&pool).await;
        create_products_meta_table(&pool).await;

        let id = add_to_collection_cmd(&pool, sample_input("ToRemove")).await.unwrap();
        let before = get_collection_cmd(&pool).await.unwrap();
        assert_eq!(before.len(), 1);

        remove_from_collection_cmd(&pool, id).await.unwrap();
        let after = get_collection_cmd(&pool).await.unwrap();
        assert!(after.is_empty());
    }

    // ── get_collection_cmd ────────────────────────────────────────────────

    #[tokio::test]
    async fn get_collection_cmd_empty_returns_empty_vec() {
        let pool = memory_pool().await;
        create_collection_items_table(&pool).await;
        create_price_history_table(&pool).await;
        create_products_meta_table(&pool).await;

        let items = get_collection_cmd(&pool).await.unwrap();
        assert!(items.is_empty());
    }

    #[tokio::test]
    async fn get_collection_cmd_returns_items_with_estimated_value() {
        let pool = memory_pool().await;
        create_collection_items_table(&pool).await;
        create_price_history_table(&pool).await;
        create_products_meta_table(&pool).await;
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;

        let mut input = sample_input("WithValue");
        input.sku = Some("VAL-SKU".to_string());
        add_to_collection_cmd(&pool, input).await.unwrap();

        // Seed price history
        sqlx::query(
            "INSERT INTO price_history (sku, price, recorded_at, source_id) VALUES (?1, ?2, ?3, ?4)",
        )
        .bind("VAL-SKU")
        .bind(1500.0)
        .bind(now - 10)
        .bind("reverb")
        .execute(&pool)
        .await
        .unwrap();

        let items = get_collection_cmd(&pool).await.unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].name, "WithValue");
        assert!(
            (items[0].estimated_value.unwrap() - 1500.0).abs() < 0.01,
            "expected estimated_value ~1500, got {:?}",
            items[0].estimated_value
        );
    }

    // ── update_collection_item_cmd ────────────────────────────────────────

    #[tokio::test]
    async fn update_collection_item_cmd_changes_price() {
        let pool = memory_pool().await;
        create_collection_items_table(&pool).await;
        create_price_history_table(&pool).await;
        create_products_meta_table(&pool).await;

        let id = add_to_collection_cmd(&pool, sample_input("UpdateMe")).await.unwrap();
        update_collection_item_cmd(
            &pool,
            id,
            CollectionItemUpdates {
                purchase_price: Some(2222.0),
                ..Default::default()
            },
        )
        .await
        .unwrap();

        let items = get_collection_cmd(&pool).await.unwrap();
        assert_eq!(items[0].purchase_price, Some(2222.0));
    }

    // ── get_collection_stats_cmd ──────────────────────────────────────────

    #[tokio::test]
    async fn get_collection_stats_cmd_empty_returns_zeros() {
        let pool = memory_pool().await;
        create_collection_items_table(&pool).await;
        create_price_history_table(&pool).await;
        create_products_meta_table(&pool).await;

        let stats = get_collection_stats_cmd(&pool).await.unwrap();
        assert_eq!(stats.total_items, 0);
        assert_eq!(stats.total_value, 0.0);
        assert_eq!(stats.top_item_name, None);
        assert_eq!(stats.top_item_value, 0.0);
    }

    #[tokio::test]
    async fn get_collection_stats_cmd_returns_aggregates() {
        let pool = memory_pool().await;
        create_collection_items_table(&pool).await;
        create_price_history_table(&pool).await;
        create_products_meta_table(&pool).await;
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;

        let mut input_a = sample_input("Strat A");
        input_a.sku = Some("SKU-A".to_string());
        add_to_collection_cmd(&pool, input_a).await.unwrap();
        sqlx::query(
            "INSERT INTO price_history (sku, price, recorded_at, source_id) VALUES (?1, ?2, ?3, ?4)",
        )
        .bind("SKU-A")
        .bind(1000.0)
        .bind(now - 10)
        .bind("reverb")
        .execute(&pool)
        .await
        .unwrap();

        let stats = get_collection_stats_cmd(&pool).await.unwrap();
        assert_eq!(stats.total_items, 1);
        assert!((stats.total_value - 1000.0).abs() < 0.01);
        assert_eq!(stats.top_item_name, Some("Strat A".to_string()));
        assert!((stats.top_item_value - 1000.0).abs() < 0.01);
    }
}
