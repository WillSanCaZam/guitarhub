// SPDX-License-Identifier: GPL-3.0-or-later

use tauri::State;

use crate::repository::wishlist::{WishlistItem, WishlistItemInput, WishlistRepo};
use crate::AppError;
use crate::AppState;

// ── Core logic (extracted for testability without Tauri runtime) ─────────

pub async fn add_to_wishlist_cmd(
    pool: &sqlx::SqlitePool,
    input: WishlistItemInput,
) -> Result<i64, AppError> {
    let repo = WishlistRepo::new(pool.clone());
    let id = repo.add(&input).await.map_err(|e| AppError::Database(e.to_string()))?;
    Ok(id)
}

pub async fn remove_from_wishlist_cmd(
    pool: &sqlx::SqlitePool,
    id: i64,
) -> Result<(), AppError> {
    let repo = WishlistRepo::new(pool.clone());
    repo.remove(id).await.map_err(|e| AppError::Database(e.to_string()))?;
    Ok(())
}

pub async fn get_wishlist_cmd(
    pool: &sqlx::SqlitePool,
) -> Result<Vec<WishlistItem>, AppError> {
    let repo = WishlistRepo::new(pool.clone());
    let items = repo.get_all().await.map_err(|e| AppError::Database(e.to_string()))?;
    Ok(items)
}

// ── Tauri IPC Commands ────────────────────────────────────────────────────

#[tauri::command]
pub async fn add_to_wishlist(
    input: WishlistItemInput,
    state: State<'_, AppState>,
) -> Result<i64, AppError> {
    add_to_wishlist_cmd(&state.pool, input).await
}

#[tauri::command]
pub async fn remove_from_wishlist(
    id: i64,
    state: State<'_, AppState>,
) -> Result<(), AppError> {
    remove_from_wishlist_cmd(&state.pool, id).await
}

#[tauri::command]
pub async fn get_wishlist(
    state: State<'_, AppState>,
) -> Result<Vec<WishlistItem>, AppError> {
    get_wishlist_cmd(&state.pool).await
}

// ── Tests ─────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repository::wishlist::create_wishlist_table;

    async fn memory_pool() -> sqlx::SqlitePool {
        sqlx::SqlitePool::connect("sqlite::memory:").await.unwrap()
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

    // ── add_to_wishlist_cmd ───────────────────────────────────────────────

    #[tokio::test]
    async fn add_to_wishlist_cmd_returns_positive_id() {
        let pool = memory_pool().await;
        create_wishlist_table(&pool).await;

        let id = add_to_wishlist_cmd(&pool, sample_input()).await.unwrap();
        assert!(id > 0, "add_to_wishlist_cmd should return positive id");
    }

    // ── remove_from_wishlist_cmd ──────────────────────────────────────────

    #[tokio::test]
    async fn remove_from_wishlist_cmd_deletes_item() {
        let pool = memory_pool().await;
        create_wishlist_table(&pool).await;

        let id = add_to_wishlist_cmd(&pool, sample_input()).await.unwrap();
        let before = get_wishlist_cmd(&pool).await.unwrap();
        assert_eq!(before.len(), 1);

        remove_from_wishlist_cmd(&pool, id).await.unwrap();
        let after = get_wishlist_cmd(&pool).await.unwrap();
        assert!(after.is_empty(), "item should be removed after delete");
    }

    // ── get_wishlist_cmd ──────────────────────────────────────────────────

    #[tokio::test]
    async fn get_wishlist_cmd_empty_returns_empty_vec() {
        let pool = memory_pool().await;
        create_wishlist_table(&pool).await;

        let items = get_wishlist_cmd(&pool).await.unwrap();
        assert!(items.is_empty(), "empty wishlist should return empty vec");
    }

    #[tokio::test]
    async fn get_wishlist_cmd_returns_items_after_add() {
        let pool = memory_pool().await;
        create_wishlist_table(&pool).await;

        add_to_wishlist_cmd(&pool, sample_input()).await.unwrap();

        let items = get_wishlist_cmd(&pool).await.unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].sku, Some("FENDER-TELE".to_string()));
        assert_eq!(items[0].name, Some("Telecaster".to_string()));
        assert_eq!(items[0].brand, Some("Fender".to_string()));
    }
}