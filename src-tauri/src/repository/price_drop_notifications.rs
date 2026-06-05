// SPDX-License-Identifier: GPL-3.0-or-later
//
// Repository for the `price_drop_notifications` table (migration 007).
//
// One row per SKU that has ever produced a price-drop notification.
// Absence of a row means "never notified", which is NOT the same as
// "in cooldown" — a fresh SKU is *not* in cooldown.
//
// Anti-spam semantics: a SKU is in cooldown when
// `now - last_notified < COOLDOWN_SECS` (24h by default, defined in
// `services/price_drop.rs`).
//
// Used by `CatalogSyncService::upsert_products` (read) and by
// `commands/sync_command::dispatch_drops` (write after a successful send).

use sqlx::SqlitePool;

/// Per-SKU cooldown state for price-drop notifications.
///
/// `(sku, last_notified, last_price, channel)` matches the migration 007 schema.
/// `last_notified` is unix epoch seconds.
#[derive(Debug, Clone, PartialEq)]
pub struct NotifiedRow {
    pub sku: String,
    pub last_notified: i64,
    pub last_price: f64,
    pub channel: String,
}

#[derive(Debug, Clone)]
pub struct PriceDropNotificationsRepo {
    pool: SqlitePool,
}

impl PriceDropNotificationsRepo {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// `INSERT OR REPLACE` into `price_drop_notifications`.
    ///
    /// Called from `sync_command` after a successful dispatch — records
    /// "we notified for this SKU at this time, at this price, on this channel".
    /// On the next sync, the cooldown check reads this row to decide
    /// whether to skip the dispatch.
    pub async fn upsert(
        &self,
        sku: &str,
        last_notified: i64,
        last_price: f64,
        channel: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT INTO price_drop_notifications
                (sku, last_notified, last_price, channel)
             VALUES (?1, ?2, ?3, ?4)
             ON CONFLICT(sku) DO UPDATE SET
                last_notified = excluded.last_notified,
                last_price    = excluded.last_price,
                channel       = excluded.channel",
        )
        .bind(sku)
        .bind(last_notified)
        .bind(last_price)
        .bind(channel)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// Read the most-recent `last_notified` timestamp for a SKU, or
    /// `None` if the SKU has never been notified.
    ///
    /// The caller (sync loop) decides what to do with the timestamp:
    /// typically `now - last < COOLDOWN_SECS` ⇒ skip the drop.
    pub async fn get_last_notified(
        &self,
        sku: &str,
    ) -> Result<Option<i64>, sqlx::Error> {
        let row: Option<(i64,)> = sqlx::query_as(
            "SELECT last_notified FROM price_drop_notifications
             WHERE sku = ?1",
        )
        .bind(sku)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(|(t,)| t))
    }
}

// ── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    async fn setup_pool() -> SqlitePool {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        sqlx::query(
            "CREATE TABLE price_drop_notifications (
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

    /// `upsert` then `get_last_notified` returns the timestamp we wrote.
    #[tokio::test]
    async fn price_drop_repo_upsert_and_get_last_notified_roundtrip() {
        let pool = setup_pool().await;
        let repo = PriceDropNotificationsRepo::new(pool);

        // Initially: no row
        let initial = repo.get_last_notified("SKU-TEST-1").await.unwrap();
        assert!(initial.is_none(), "no row initially");

        // upsert
        repo.upsert("SKU-TEST-1", 1_700_000_000, 850.0, "app")
            .await
            .expect("upsert must succeed");

        // get_last_notified returns the timestamp
        let last = repo.get_last_notified("SKU-TEST-1").await.unwrap();
        assert_eq!(last, Some(1_700_000_000), "got wrong timestamp");

        // upsert again (overwrite)
        repo.upsert("SKU-TEST-1", 1_700_086_400, 800.0, "ntfy")
            .await
            .expect("overwrite must succeed");

        let last = repo.get_last_notified("SKU-TEST-1").await.unwrap();
        assert_eq!(
            last,
            Some(1_700_086_400),
            "overwrite must update last_notified"
        );
    }

    /// `get_last_notified` for an unknown SKU returns `None` (not error).
    #[tokio::test]
    async fn price_drop_repo_get_last_notified_unknown_sku_returns_none() {
        let pool = setup_pool().await;
        let repo = PriceDropNotificationsRepo::new(pool);
        let result = repo.get_last_notified("NEVER-SEEN").await.unwrap();
        assert!(result.is_none());
    }
}
