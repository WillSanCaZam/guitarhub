// SPDX-License-Identifier: GPL-3.0-or-later

use serde::Serialize;
use sqlx::SqlitePool;

/// A single price data point for a specific source.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct PricePoint {
    pub source_id: String,
    pub recorded_at: i64,
    pub price: f64,
}

/// Raw rolling stats from the price_history table.
/// All fields use Option because aggregate queries return NULL
/// when no matching rows exist.
pub struct PriceInsightRow {
    pub min_30d: f64,
    pub avg_90d: f64,
    pub cnt_30d: i64,
    /// None when the SKU has no price_history rows at all.
    pub current_price: Option<f64>,
    /// Max price within the 30-day window (used for stability CoV).
    pub max_30d: f64,
    /// Mean price within the 30-day window (CoV denominator).
    pub avg_30d: f64,
    /// Distinct sources in the 30-day window.
    pub source_count_30d: i64,
    /// Newest `recorded_at` for the SKU (i64 epoch seconds, 0 if no rows).
    pub last_recorded_at: i64,
    /// `epoch_seconds()` captured at query time — used to derive
    /// `days_since_last` so repo and caller share one `now`.
    pub now_epoch: i64,
}

/// SQL queries against the `price_history` table with outlier filtering.
///
/// Follows the same concrete-struct pattern as `ImageCacheRepo` — no trait.
/// All methods operate against a `SqlitePool` and expect the `price_history`
/// table to already exist with the `source_id` column (migration 004).
#[derive(Debug, Clone)]
pub struct PriceHistoryRepo {
    pool: SqlitePool,
}

impl PriceHistoryRepo {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// Get price points within the window, filtered by 5σ per source.
    ///
    /// If a source has < 30 points, its data is returned unfiltered.
    /// Results are ordered by `recorded_at ASC`.
    ///
    /// Outlier filtering is done in Rust (not SQL) because SQLite has no
    /// built-in SQRT function and loading the math extension adds complexity.
    /// With <1000 points per SKU, the Rust overhead is negligible.
    pub async fn get_history(
        &self,
        sku: &str,
        window_days: u32,
    ) -> Result<Vec<PricePoint>, sqlx::Error> {
        let now = epoch_seconds();
        let min_recorded_at = now - (window_days as i64 * 86_400);

        // Fetch ALL rows within the window — typically <1000 per SKU
        let rows: Vec<(String, i64, f64)> = sqlx::query_as(
            r#"
            SELECT source_id, recorded_at, price
            FROM price_history
            WHERE sku = ?1 AND recorded_at >= ?2
            ORDER BY recorded_at ASC
            "#,
        )
        .bind(sku)
        .bind(min_recorded_at)
        .fetch_all(&self.pool)
        .await?;

        // Group by source_id
        let mut by_source: std::collections::HashMap<String, Vec<(i64, f64)>> =
            std::collections::HashMap::new();
        for (source_id, recorded_at, price) in rows {
            by_source
                .entry(source_id)
                .or_default()
                .push((recorded_at, price));
        }

        // Apply 5σ filter per source in Rust
        let mut result: Vec<PricePoint> = Vec::new();
        for (source_id, points) in &by_source {
            if points.len() < 30 {
                // Include all points unfiltered
                for &(recorded_at, price) in points {
                    result.push(PricePoint {
                        source_id: source_id.clone(),
                        recorded_at,
                        price,
                    });
                }
            } else {
                // Compute population mean and stddev
                let count = points.len() as f64;
                let mean: f64 = points.iter().map(|&(_, p)| p).sum::<f64>() / count;
                let variance: f64 = points
                    .iter()
                    .map(|&(_, p)| (p - mean) * (p - mean))
                    .sum::<f64>()
                    / count;
                let stddev = variance.sqrt();

                if stddev == 0.0 {
                    // All values equal — include all points
                    for &(recorded_at, price) in points {
                        result.push(PricePoint {
                            source_id: source_id.clone(),
                            recorded_at,
                            price,
                        });
                    }
                } else {
                    let threshold = 5.0 * stddev;
                    for &(recorded_at, price) in points {
                        if (price - mean).abs() <= threshold {
                            result.push(PricePoint {
                                source_id: source_id.clone(),
                                recorded_at,
                                price,
                            });
                        }
                    }
                }
            }
        }

        // Sort by recorded_at ASC across all sources (re-sort since we merged per-source)
        result.sort_by_key(|p| p.recorded_at);
        Ok(result)
    }

    /// Compute rolling stats for price insight.
    ///
    /// Returns `Ok(PriceInsightRow)` with `current_price = None` when
    /// the SKU has no price_history rows at all. Otherwise all fields
    /// are populated from the most recent 30/90 day windows.
    pub async fn get_insight(&self, sku: &str) -> Result<PriceInsightRow, sqlx::Error> {
        self.get_insight_at(sku, epoch_seconds()).await
    }

    /// Like `get_insight` but accepts an explicit `now` timestamp for
    /// deterministic testing. Not public — tests access it via a `#[doc(hidden)]`
    /// re-export or direct `pub(crate)`.
    pub(crate) async fn get_insight_at(
        &self,
        sku: &str,
        now: i64,
    ) -> Result<PriceInsightRow, sqlx::Error> {
        let window_30d = now - 30 * 86_400;
        let window_90d = now - 90 * 86_400;

        // Single-row aggregate; existing idx_price_history_sku_recorded
        // covers all four new aggregates — no new index needed.
        type InsightAggregate = (
            Option<f64>, // min_30d
            Option<f64>, // avg_30d
            Option<f64>, // avg_90d
            Option<i64>, // cnt_30d
            Option<f64>, // max_30d
            Option<i64>, // last_recorded_at
            Option<i64>, // source_count_30d
            Option<f64>, // current_price
        );
        let (min_30d, avg_30d, avg_90d, cnt_30d, max_30d, last_recorded_at, source_count_30d, current_price): InsightAggregate = sqlx::query_as(
            r#"
            SELECT
                MIN(CASE WHEN recorded_at >= ?2 THEN price END)  AS min_30d,
                AVG(CASE WHEN recorded_at >= ?2 THEN price END)  AS avg_30d,
                AVG(CASE WHEN recorded_at >= ?3 THEN price END)  AS avg_90d,
                COUNT(CASE WHEN recorded_at >= ?2 THEN 1 END)    AS cnt_30d,
                MAX(CASE WHEN recorded_at >= ?2 THEN price END)  AS max_30d,
                MAX(recorded_at)                                  AS last_recorded_at,
                COUNT(DISTINCT CASE WHEN recorded_at >= ?2 THEN source_id END) AS source_count_30d,
                (SELECT price FROM price_history
                 WHERE sku = ?1 ORDER BY recorded_at DESC LIMIT 1) AS current_price
            FROM price_history
            WHERE sku = ?1
            "#,
        )
        .bind(sku)
        .bind(window_30d)
        .bind(window_90d)
        .fetch_one(&self.pool)
        .await?;

        Ok(PriceInsightRow {
            min_30d: min_30d.unwrap_or(0.0),
            avg_30d: avg_30d.unwrap_or(0.0),
            avg_90d: avg_90d.unwrap_or(0.0),
            cnt_30d: cnt_30d.unwrap_or(0),
            max_30d: max_30d.unwrap_or(0.0),
            source_count_30d: source_count_30d.unwrap_or(0),
            last_recorded_at: last_recorded_at.unwrap_or(0),
            current_price,
            now_epoch: now,
        })
    }

    /// Return the most recent price for a SKU across all sources, or `None`
    /// if the SKU has no `price_history` rows.
    ///
    /// "Most recent" is defined by `recorded_at DESC` (timestamp), not by
    /// row insertion order — the latest tick wins, regardless of source.
    /// Used by the price-drop detector to compute the baseline before
    /// writing a new `record_price` row.
    pub async fn get_last_price(&self, sku: &str) -> Result<Option<f64>, sqlx::Error> {
        let row: Option<(f64,)> = sqlx::query_as(
            "SELECT price FROM price_history
             WHERE sku = ?1
             ORDER BY recorded_at DESC
             LIMIT 1",
        )
        .bind(sku)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(|(p,)| p))
    }

    /// Persist a new `price_history` row for `sku` at the given `now`
    /// (unix epoch seconds). One row per (sku, source_id, recorded_at).
    ///
    /// Returns `sqlx::Error` on DB failure — callers (the sync loop) MUST
    /// log + continue so a single failed write does not abort the sync.
    pub async fn record_price(
        &self,
        sku: &str,
        price: f64,
        source_id: &str,
        now: i64,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT INTO price_history (sku, price, recorded_at, source_id)
             VALUES (?1, ?2, ?3, ?4)",
        )
        .bind(sku)
        .bind(price)
        .bind(now)
        .bind(source_id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}

/// Current Unix epoch time in seconds.
fn epoch_seconds() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

// ── Helpers (test support, compiled unconditionally so integration tests can reuse) ───────

/// Create the `price_history` table (post-migration 004 state) in an `:memory:` database.
pub async fn create_price_history_table(pool: &SqlitePool) {
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS price_history (
            id          INTEGER PRIMARY KEY AUTOINCREMENT,
            sku         TEXT NOT NULL,
            price       REAL NOT NULL,
            recorded_at INTEGER NOT NULL,
            source_id   TEXT NOT NULL DEFAULT ''
        )",
    )
    .execute(pool)
    .await
    .unwrap();

    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_price_history_sku_recorded
         ON price_history(sku, recorded_at)",
    )
    .execute(pool)
    .await
    .unwrap();
}

pub async fn make_memory_pool() -> SqlitePool {
    SqlitePool::connect("sqlite::memory:").await.unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── get_history: empty data ──────────────────────────────────────────

    #[tokio::test]
    async fn get_history_empty_table_returns_empty_vec() {
        let pool = make_memory_pool().await;
        create_price_history_table(&pool).await;
        let repo = PriceHistoryRepo::new(pool);

        let result = repo.get_history("NONEXISTENT", 365).await.unwrap();
        assert!(
            result.is_empty(),
            "expected empty vec for SKU with no data"
        );
    }

    // ── get_history: single source ───────────────────────────────────────

    #[tokio::test]
    async fn get_history_returns_points_within_window() {
        let pool = make_memory_pool().await;
        create_price_history_table(&pool).await;
        let repo = PriceHistoryRepo::new(pool);
        let now = epoch_seconds();

        // Insert 3 points: 2 within window (different times to test ASC ordering), 1 outside
        insert_point(&repo, "SKU1", "reverb", 200.0, now - 10).await;   // later
        insert_point(&repo, "SKU1", "reverb", 100.0, now - 20).await;   // earlier
        insert_point(&repo, "SKU1", "reverb", 300.0, now - 400 * 86_400).await; // outside 365d window

        let result = repo.get_history("SKU1", 365).await.unwrap();
        assert_eq!(result.len(), 2, "expected 2 points within window");
        // Results should be ASC by recorded_at: now-20 (100) first, then now-10 (200)
        assert_eq!(result[0].source_id, "reverb");
        assert_eq!(result[0].recorded_at, now - 20, "first should be earlier");
        assert_eq!(result[0].price, 100.0, "earlier point has price 100");
        assert_eq!(result[1].recorded_at, now - 10, "second should be later");
        assert_eq!(result[1].price, 200.0, "later point has price 200");
    }

    // ── get_history: 5σ outlier filter ───────────────────────────────────

    #[tokio::test]
    async fn get_history_filters_5sigma_outliers() {
        let pool = make_memory_pool().await;
        create_price_history_table(&pool).await;
        let repo = PriceHistoryRepo::new(pool);
        let now = epoch_seconds();

        // Source A: 35 points clustered around 100, plus one extreme outlier at 10000
        let a_mean = 100.0;
        for i in 0..35 {
            let jitter = (i as f64 - 17.0) * 0.5; // prices: ~91.5 to ~108.5
            insert_point(&repo, "SKU2", "source_a", a_mean + jitter, now - i as i64 * 86_400).await;
        }
        // Outlier
        insert_point(&repo, "SKU2", "source_a", 10_000.0, now - 1).await;

        // Source B: also 35 points clustered around 200, no outlier
        for i in 0..35 {
            let jitter = (i as f64 - 17.0) * 0.5;
            insert_point(&repo, "SKU2", "source_b", 200.0 + jitter, now - i as i64 * 86_400).await;
        }

        let result = repo.get_history("SKU2", 365).await.unwrap();

        // Count per source
        let count_a = result.iter().filter(|p| p.source_id == "source_a").count();
        let count_b = result.iter().filter(|p| p.source_id == "source_b").count();

        // Source A: outlier excluded, so 35 points remain
        assert_eq!(
            count_a, 35,
            "expected 35 points from source_a (outlier filtered)"
        );
        // Source B: all 35 remain
        assert_eq!(count_b, 35, "expected 35 points from source_b");

        // Verify no point has the outlier price
        assert!(
            !result.iter().any(|p| (p.price - 10_000.0).abs() < f64::EPSILON),
            "outlier at 10000 should be excluded"
        );
    }

    // ── get_history: < 30 points returns unfiltered ──────────────────────

    #[tokio::test]
    async fn get_history_returns_all_when_under_30_points() {
        let pool = make_memory_pool().await;
        create_price_history_table(&pool).await;
        let repo = PriceHistoryRepo::new(pool);
        let now = epoch_seconds();

        // 25 points from one source, includes an extreme value
        for i in 0..24 {
            insert_point(&repo, "SKU3", "src_a", 100.0, now - i as i64 * 86_400).await;
        }
        insert_point(&repo, "SKU3", "src_a", 9999.0, now - 25 * 86_400).await;

        let result = repo.get_history("SKU3", 365).await.unwrap();
        assert_eq!(
            result.len(),
            25,
            "expected all 25 points unfiltered (cnt < 30)"
        );
        assert!(
            result.iter().any(|p| (p.price - 9999.0).abs() < f64::EPSILON),
            "extreme value should be included when cnt < 30"
        );
    }

    // ── get_history: multi-source ────────────────────────────────────────

    #[tokio::test]
    async fn get_history_multi_source_returns_all_sources() {
        let pool = make_memory_pool().await;
        create_price_history_table(&pool).await;
        let repo = PriceHistoryRepo::new(pool);
        let now = epoch_seconds();

        insert_point(&repo, "SKU4", "reverb", 200.0, now - 10).await;
        insert_point(&repo, "SKU4", "ebay", 190.0, now - 10).await;
        insert_point(&repo, "SKU4", "guitarcenter", 210.0, now - 10).await;

        let result = repo.get_history("SKU4", 365).await.unwrap();
        assert_eq!(result.len(), 3, "expected 3 points from 3 sources");
        let sources: std::collections::HashSet<&str> =
            result.iter().map(|p| p.source_id.as_str()).collect();
        assert!(sources.contains("reverb"));
        assert!(sources.contains("ebay"));
        assert!(sources.contains("guitarcenter"));
    }

    // ── get_history: single point ────────────────────────────────────────

    #[tokio::test]
    async fn get_history_single_point_returns_it() {
        let pool = make_memory_pool().await;
        create_price_history_table(&pool).await;
        let repo = PriceHistoryRepo::new(pool);
        let now = epoch_seconds();

        insert_point(&repo, "SKU5", "reverb", 150.0, now - 10).await;

        let result = repo.get_history("SKU5", 365).await.unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].price, 150.0);
        assert_eq!(result[0].source_id, "reverb");
    }

    // ── get_insight: returns data when rows exist ────────────────────────

    #[tokio::test]
    async fn get_insight_returns_stats_within_windows() {
        let pool = make_memory_pool().await;
        create_price_history_table(&pool).await;
        let repo = PriceHistoryRepo::new(pool);
        // Use a deterministic timestamp — no wall-clock drift between insert and query
        let now = 2_000_000_000;

        // Insert 35 points over 35 days: min = 90, prices range from 90 to 124
        for i in 0..35 {
            let price = 90.0 + (i as f64); // 90..124
            insert_point(
                &repo,
                "SKU6",
                "reverb",
                price,
                now - (i as i64 * 86_400),
            )
            .await;
        }

        let row = repo.get_insight_at("SKU6", now).await.unwrap();

        assert!(
            row.current_price.is_some(),
            "expected current_price for SKU with data"
        );
        assert!(
            row.min_30d > 0.0,
            "expected positive min_30d"
        );
        assert!(
            row.avg_90d > 0.0,
            "expected positive avg_90d"
        );
        // 31 points within 30-day window (i=0..30 of 0..34)
        assert_eq!(row.cnt_30d, 31, "expected 31 points in 30-day window");
        // min_30d within last 30 days: i=0 has price=90
        assert!(
            (row.min_30d - 90.0).abs() < 1.0,
            "expected min_30d close to 90, got {}",
            row.min_30d
        );
    }

    // ── get_insight: empty table returns current_price = None ────────────

    #[tokio::test]
    async fn get_insight_no_data_returns_current_price_none() {
        let pool = make_memory_pool().await;
        create_price_history_table(&pool).await;
        let repo = PriceHistoryRepo::new(pool);

        let row = repo.get_insight("NONEXISTENT").await.unwrap();
        assert!(
            row.current_price.is_none(),
            "expected current_price=None for SKU with no data"
        );
        assert_eq!(row.cnt_30d, 0, "expected cnt_30d=0");
        assert_eq!(row.min_30d, 0.0, "expected min_30d=0.0");
        assert_eq!(row.avg_90d, 0.0, "expected avg_90d=0.0");
    }

    // ── get_insight: all prices equal → stddev=0 ─────────────────────────

    #[tokio::test]
    async fn get_insight_all_prices_equal() {
        let pool = make_memory_pool().await;
        create_price_history_table(&pool).await;
        let repo = PriceHistoryRepo::new(pool);
        // Deterministic timestamp — no wall-clock drift
        let now = 2_000_000_000;

        // 35 points, all at 100.0. i=0..30 are within 30-day window (31 points)
        for i in 0..35 {
            insert_point(&repo, "SKU7", "reverb", 100.0, now - i as i64 * 86_400).await;
        }

        let row = repo.get_insight_at("SKU7", now).await.unwrap();
        assert!(row.current_price.is_some());
        assert!((row.min_30d - 100.0).abs() < f64::EPSILON);
        assert!((row.avg_90d - 100.0).abs() < f64::EPSILON);
        // i=0..30 are within 30-day window (31 points)
        assert_eq!(row.cnt_30d, 31, "expected 31/35 within 30-day window");
        // All 35 within 90-day window
        assert!((row.avg_90d - 100.0).abs() < f64::EPSILON);
    }

    // ── Helper: insert a price history point ─────────────────────────────

    async fn insert_point(
        repo: &PriceHistoryRepo,
        sku: &str,
        source_id: &str,
        price: f64,
        recorded_at: i64,
    ) {
        sqlx::query(
            "INSERT INTO price_history (sku, price, recorded_at, source_id)
             VALUES (?1, ?2, ?3, ?4)",
        )
        .bind(sku)
        .bind(price)
        .bind(recorded_at)
        .bind(source_id)
        .execute(&repo.pool)
        .await
        .unwrap();
    }

    // ── get_last_price + record_price: new methods for price-drop detector ─

    #[tokio::test]
    async fn get_last_price_returns_none_when_empty() {
        let pool = make_memory_pool().await;
        create_price_history_table(&pool).await;
        let repo = PriceHistoryRepo::new(pool);

        let result = repo.get_last_price("NONEXISTENT").await.unwrap();
        assert!(
            result.is_none(),
            "expected None for SKU with no price history, got {:?}",
            result
        );
    }

    #[tokio::test]
    async fn record_price_then_get_last_price_returns_it() {
        let pool = make_memory_pool().await;
        create_price_history_table(&pool).await;
        let repo = PriceHistoryRepo::new(pool);
        let now = epoch_seconds();

        repo.record_price("SKU1", 100.0, "reverb", now - 100).await.unwrap();
        repo.record_price("SKU1", 80.0, "reverb", now - 50).await.unwrap();

        let last = repo.get_last_price("SKU1").await.unwrap();
        assert_eq!(
            last,
            Some(80.0),
            "expected most recent price 80.0, got {:?}",
            last
        );
    }

    #[tokio::test]
    async fn get_last_price_returns_most_recent_across_sources() {
        let pool = make_memory_pool().await;
        create_price_history_table(&pool).await;
        let repo = PriceHistoryRepo::new(pool);
        let now = epoch_seconds();

        // Same SKU, different sources at different times. Most recent across ALL sources wins.
        repo.record_price("SKU2", 100.0, "reverb", now - 300).await.unwrap();
        repo.record_price("SKU2", 110.0, "ebay", now - 200).await.unwrap();
        repo.record_price("SKU2", 95.0, "guitarcenter", now - 100).await.unwrap();

        let last = repo.get_last_price("SKU2").await.unwrap();
        assert_eq!(
            last,
            Some(95.0),
            "expected most recent price 95.0 across all sources, got {:?}",
            last
        );
    }

    #[tokio::test]
    async fn record_price_persists_source_id() {
        let pool = make_memory_pool().await;
        create_price_history_table(&pool).await;
        let repo = PriceHistoryRepo::new(pool);
        let now = epoch_seconds();

        repo.record_price("SKU3", 250.0, "sweetwater", now).await.unwrap();

        let (price, source_id): (f64, String) = sqlx::query_as(
            "SELECT price, source_id FROM price_history WHERE sku = ?1",
        )
        .bind("SKU3")
        .fetch_one(&repo.pool)
        .await
        .unwrap();
        assert_eq!(price, 250.0);
        assert_eq!(source_id, "sweetwater");
    }

    // ── get_insight: confidence aggregates (RED — fields don't exist yet) ─

    #[tokio::test]
    async fn get_insight_no_data_returns_zero_confidence_aggregates() {
        let pool = make_memory_pool().await;
        create_price_history_table(&pool).await;
        let repo = PriceHistoryRepo::new(pool);

        let row = repo.get_insight("NONEXISTENT").await.unwrap();
        assert_eq!(row.max_30d, 0.0, "expected max_30d=0 for empty SKU");
        assert_eq!(row.avg_30d, 0.0, "expected avg_30d=0 for empty SKU");
        assert_eq!(row.source_count_30d, 0, "expected source_count_30d=0");
        assert_eq!(row.last_recorded_at, 0, "expected last_recorded_at=0");
    }

    #[tokio::test]
    async fn get_insight_max_30d_is_highest_price_in_window() {
        let pool = make_memory_pool().await;
        create_price_history_table(&pool).await;
        let repo = PriceHistoryRepo::new(pool);
        // Use a deterministic timestamp — no wall-clock drift between insert and query
        let now = 2_000_000_000;

        // 35 points: i=0..30 within 30d (31 points, prices 90..120)
        for i in 0..35 {
            insert_point(&repo, "SKU_MAX", "reverb", 90.0 + i as f64, now - i as i64 * 86_400)
                .await;
        }

        let row = repo.get_insight_at("SKU_MAX", now).await.unwrap();
        assert!(
            (row.max_30d - 120.0).abs() < f64::EPSILON,
            "expected max_30d=120 (i=30 at boundary), got {}",
            row.max_30d
        );
    }

    #[tokio::test]
    async fn get_insight_avg_30d_is_mean_in_window() {
        let pool = make_memory_pool().await;
        create_price_history_table(&pool).await;
        let repo = PriceHistoryRepo::new(pool);
        // Deterministic timestamp — no wall-clock drift
        let now = 2_000_000_000;

        // 35 points: i=0..30 within 30d, prices 100, 101, ..., 130
        for i in 0..35 {
            insert_point(&repo, "SKU_AVG", "reverb", 100.0 + i as f64, now - i as i64 * 86_400)
                .await;
        }

        let row = repo.get_insight_at("SKU_AVG", now).await.unwrap();
        // Window contains i=0..30 → 31 points, prices 100..130 → mean = 115
        let expected = (100.0 + 130.0) / 2.0;
        assert!(
            (row.avg_30d - expected).abs() < 0.01,
            "expected avg_30d≈{expected}, got {}",
            row.avg_30d
        );
    }

    #[tokio::test]
    async fn get_insight_source_count_30d_counts_distinct_sources() {
        let pool = make_memory_pool().await;
        create_price_history_table(&pool).await;
        let repo = PriceHistoryRepo::new(pool);
        // Deterministic timestamp — no wall-clock drift
        let now = 2_000_000_000;

        // 3 distinct sources within 30d
        insert_point(&repo, "SKU_SRC", "reverb", 100.0, now - 86_400).await;
        insert_point(&repo, "SKU_SRC", "ebay", 110.0, now - 2 * 86_400).await;
        insert_point(&repo, "SKU_SRC", "guitarcenter", 120.0, now - 3 * 86_400).await;
        // 4th source but older than 30d → must not be counted
        insert_point(&repo, "SKU_SRC", "oldshop", 130.0, now - 60 * 86_400).await;

        let row = repo.get_insight_at("SKU_SRC", now).await.unwrap();
        assert_eq!(
            row.source_count_30d, 3,
            "expected 3 distinct sources within 30d, got {}",
            row.source_count_30d
        );
    }

    #[tokio::test]
    async fn get_insight_last_recorded_at_is_maximum() {
        let pool = make_memory_pool().await;
        create_price_history_table(&pool).await;
        let repo = PriceHistoryRepo::new(pool);
        // Deterministic timestamp — no wall-clock drift
        let now = 2_000_000_000;

        // Insert at scattered times — newest is 100s from now
        let newest = now - 100;
        insert_point(&repo, "SKU_LAST", "reverb", 100.0, now - 5 * 86_400).await;
        insert_point(&repo, "SKU_LAST", "reverb", 110.0, now - 86_400).await;
        insert_point(&repo, "SKU_LAST", "reverb", 120.0, newest).await;
        insert_point(&repo, "SKU_LAST", "reverb", 130.0, now - 50 * 86_400).await;

        let row = repo.get_insight_at("SKU_LAST", now).await.unwrap();
        assert_eq!(
            row.last_recorded_at, newest,
            "expected last_recorded_at to be the newest inserted timestamp"
        );
    }
}
