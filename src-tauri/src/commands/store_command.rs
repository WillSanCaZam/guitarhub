// SPDX-License-Identifier: GPL-3.0-or-later

use crate::domain::store::Connection;
use crate::services::sync::SyncResult;
use crate::services::user_sync::UserSyncService;
use crate::AppError;
use crate::AppState;
use tauri::State;

// ── Tauri command wrappers ───────────────────────────────────────────────

/// Connect a store account: validate token, encrypt, persist, and auto-sync.
#[tauri::command]
pub async fn connect_store(
    store_id: String,
    token: String,
    state: State<'_, AppState>,
) -> Result<Connection, AppError> {
    connect_store_inner(&state, &store_id, &token).await
}

/// Disconnect a store: remove connection and soft-delete user products.
#[tauri::command]
pub async fn disconnect_store(
    store_id: String,
    state: State<'_, AppState>,
) -> Result<(), AppError> {
    disconnect_store_inner(&state, &store_id).await
}

/// List all store connections (token_encrypted NEVER included).
#[tauri::command]
pub async fn list_connections(
    state: State<'_, AppState>,
) -> Result<Vec<Connection>, AppError> {
    list_connections_inner(&state).await
}

/// Validate a token against the store's API without storing it.
#[tauri::command]
pub async fn validate_store_token(
    store_id: String,
    token: String,
    state: State<'_, AppState>,
) -> Result<String, AppError> {
    validate_token_inner(&state, &store_id, &token).await
}

/// Trigger a sync for a store connection's listings.
#[tauri::command]
pub async fn sync_user_listings(
    store_id: String,
    state: State<'_, AppState>,
) -> Result<SyncResult, AppError> {
    sync_user_listings_inner(&state, &store_id).await
}

// ── Testable inner implementations ───────────────────────────────────────

/// Inner implementation of `connect_store` — testable without Tauri runtime.
///
/// Validates the token, encrypts and persists it, then auto-triggers
/// a sync of the user's listings.
pub(crate) async fn connect_store_inner(
    state: &AppState,
    store_id: &str,
    token: &str,
) -> Result<Connection, AppError> {
    let conn = state.connection_manager.connect(store_id, token).await?;

    // Auto-trigger sync after connect (best-effort — errors logged, not returned)
    match sync_user_listings_inner(state, store_id).await {
        Ok(result) => {
            tracing::info!(
                "Auto-sync after connect: {} loaded, {} updated, {} delisted",
                result.products_loaded,
                result.products_updated,
                result.delisted,
            );
        }
        Err(e) => {
            tracing::warn!("Auto-sync after connect failed (connection still active): {e}");
        }
    }

    Ok(conn)
}

/// Inner implementation of `disconnect_store`.
pub(crate) async fn disconnect_store_inner(
    state: &AppState,
    store_id: &str,
) -> Result<(), AppError> {
    state.connection_manager.disconnect(store_id).await
}

/// Inner implementation of `list_connections`.
pub(crate) async fn list_connections_inner(
    state: &AppState,
) -> Result<Vec<Connection>, AppError> {
    state.connection_manager.list().await
}

/// Inner implementation of `validate_store_token`.
pub(crate) async fn validate_token_inner(
    state: &AppState,
    store_id: &str,
    token: &str,
) -> Result<String, AppError> {
    state.connection_manager.validate_token(store_id, token).await
}

/// Inner implementation of `sync_user_listings`.
///
/// 1. Looks up the connection by store_id to get conn_id
/// 2. Creates a `UserSyncService` and runs sync(conn_id)
pub(crate) async fn sync_user_listings_inner(
    state: &AppState,
    store_id: &str,
) -> Result<SyncResult, AppError> {
    // Get the connection to find conn_id
    let conns = state.connection_manager.list().await?;
    let conn = conns
        .iter()
        .find(|c| c.store_id == store_id)
        .ok_or_else(|| AppError::StoreNotFound(store_id.to_string()))?;

    let sync_service = UserSyncService::new(
        state.pool.clone(),
        state.http_client.clone(),
        state.connection_manager.clone(),
    );
    sync_service.sync(conn.id).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::connection_manager::{derive_encryption_key, encrypt_token, ConnectionManager};
    use aes_gcm::{Aes256Gcm, KeyInit};
    use sqlx::SqlitePool;

    /// Create an in-memory SQLite pool with required tables.
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

        let _ = sqlx::query("ALTER TABLE products_meta ADD COLUMN user_id TEXT")
            .execute(&pool)
            .await;

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

    /// Build a minimal AppState with in-memory DB.
    async fn build_state(pool: SqlitePool) -> AppState {
        let http_client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(5))
            .build()
            .unwrap();
        let cm = ConnectionManager::new(pool.clone()).await;

        AppState {
            pool,
            http_client,
            connection_manager: cm,
            // These fields are not directly used in store command tests
            // but must be populated for compilation.
            image_cache_service: crate::services::image_cache::ImageCacheService::new_default(
                sqlx::SqlitePool::connect("sqlite::memory:").await.unwrap(),
                std::sync::Arc::new(crate::repository::sqlite::settings::SqliteSettingsRepository::new(
                    sqlx::SqlitePool::connect("sqlite::memory:").await.unwrap(),
                )),
            ),
            product_query: crate::services::product_query::ProductQueryService::new(
                sqlx::SqlitePool::connect("sqlite::memory:").await.unwrap(),
            ),
        }
    }

    /// Helper: seed an encrypted connection in the DB and return its id.
    async fn seed_connection(pool: &SqlitePool, store_id: &str, token: &str, username: &str) -> i64 {
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
        .bind(store_id)
        .bind(store_id)
        .bind(&encrypted)
        .bind(username)
        .bind(now)
        .bind(now)
        .bind(1_i32)
        .execute(pool)
        .await
        .unwrap();

        sqlx::query_scalar::<_, i64>("SELECT id FROM store_connections WHERE store_id = ?1")
            .bind(store_id)
            .fetch_one(pool)
            .await
            .unwrap()
    }

    // ── Inner function tests ─────────────────────────────────────────────
    //    These verify that the command → service routing is correct.
    //    Full service-level behavior is tested in the respective service
    //    modules (connection_manager, user_sync, reverb_api).

    /// `list_connections_inner` returns an empty list when no connections exist.
    #[tokio::test]
    async fn list_connections_inner_returns_empty_initially() {
        let pool = setup_db().await;
        let state = build_state(pool).await;

        let conns = list_connections_inner(&state).await.unwrap();
        assert!(
            conns.is_empty(),
            "expected empty list when no connections exist"
        );
    }

    /// `list_connections_inner` includes seeded connections.
    #[tokio::test]
    async fn list_connections_inner_includes_seeded_connection() {
        let pool = setup_db().await;
        seed_connection(&pool, "reverb", "pat_test", "@guitarist").await;
        let state = build_state(pool).await;

        let conns = list_connections_inner(&state).await.unwrap();
        assert_eq!(conns.len(), 1, "expected 1 connection");
        assert_eq!(conns[0].store_id, "reverb");
        assert_eq!(
            conns[0].username.as_deref(),
            Some("@guitarist"),
            "username should match"
        );
    }

    /// `disconnect_store_inner` removes a connection and delists products.
    #[tokio::test]
    async fn disconnect_store_inner_removes_connection_and_delists_products() {
        let pool = setup_db().await;
        let conn_id = seed_connection(&pool, "reverb", "pat_disc", "@guitarist").await;

        // Seed a user product
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        let user_id_str = conn_id.to_string();

        sqlx::query(
            "INSERT INTO products_meta (sku, source_id, name, brand, model, category, subcategory,
             price, currency, condition, availability, url, image_url, seller, location, synced_at, user_id, is_active)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, 1)",
        )
        .bind("reverb-user-disc-1")
        .bind("Reverb")
        .bind("Disconnect Test Product")
        .bind("Fender")
        .bind("Strat")
        .bind("Guitars")
        .bind("Electric")
        .bind(999.99_f64)
        .bind("USD")
        .bind("used")
        .bind("in_stock")
        .bind("https://reverb.com/item/disc-1")
        .bind("")
        .bind("Reverb")
        .bind("")
        .bind(now)
        .bind(&user_id_str)
        .execute(&pool)
        .await
        .unwrap();

        let state = build_state(pool.clone()).await;

        // Verify product is active before disconnect
        let active_before: i32 = sqlx::query_scalar(
            "SELECT is_active FROM products_meta WHERE sku = 'reverb-user-disc-1'",
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(active_before, 1);

        // Disconnect
        disconnect_store_inner(&state, "reverb").await.unwrap();

        // Connection should be removed
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM store_connections")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(count, 0, "connection should be deleted");

        // Product should be delisted
        let active_after: i32 = sqlx::query_scalar(
            "SELECT is_active FROM products_meta WHERE sku = 'reverb-user-disc-1'",
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(active_after, 0, "product should be delisted after disconnect");
    }

    /// `disconnect_store_inner` returns error for nonexistent store.
    #[tokio::test]
    async fn disconnect_store_inner_returns_error_for_nonexistent_store() {
        let pool = setup_db().await;
        let state = build_state(pool).await;

        let result = disconnect_store_inner(&state, "nonexistent").await;
        assert!(result.is_err(), "expected error for nonexistent store");
    }

    /// `sync_user_listings_inner` returns error when store is not connected.
    #[tokio::test]
    async fn sync_user_listings_inner_errors_when_store_not_connected() {
        let pool = setup_db().await;
        let state = build_state(pool).await;

        let result = sync_user_listings_inner(&state, "reverb").await;
        assert!(
            result.is_err(),
            "expected error when no connection exists"
        );
        let err = result.unwrap_err();
        assert!(
            matches!(&err, AppError::StoreNotFound(s) if s == "reverb"),
            "expected StoreNotFound, got: {err}"
        );
    }

    /// `sync_user_listings_inner` returns an error when trying to sync a
    /// connected store (the token was seeded with a fake value, so the
    /// Reverb API will reject it — proving the command routing reached
    /// the API layer rather than failing with StoreNotFound).
    #[tokio::test]
    async fn sync_user_listings_inner_reaches_api_layer() {
        let pool = setup_db().await;
        seed_connection(&pool, "reverb", "pat_sync_test", "@guitarist").await;

        let state = build_state(pool.clone()).await;

        let result = sync_user_listings_inner(&state, "reverb").await;

        // We expect an API-level error (token invalid or network), NOT
        // StoreNotFound — this proves the command correctly looked up the
        // connection and called through to UserSyncService::sync().
        match &result {
            Err(err) => {
                assert!(
                    !matches!(err, AppError::StoreNotFound(_)),
                    "sync must reach API layer, not fail with StoreNotFound"
                );
                // Accept any API-level error (token invalid / network).
            }
            Ok(_) => {
                // If by some chance it succeeds, that's fine too.
            }
        }
    }
}
