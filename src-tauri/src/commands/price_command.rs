use serde::Serialize;
use tauri::State;
use sqlx::SqlitePool;

use crate::repository::price_history::{PriceHistoryRepo, PricePoint};
use crate::AppError;
use crate::AppState;

// ── Confidence scoring constants ─────────────────────────────────────────
//
// Single source of truth for confidence thresholds. Tuned via these names
// rather than scattered magic numbers — see REQ-PI-7 in the spec.

/// Upper bound of the Low tier / floor of the Medium tier.
pub const CONFIDENCE_TIER_MEDIUM: u8 = 50;
/// Upper bound of the Medium tier / floor of the High tier.
pub const CONFIDENCE_TIER_HIGH: u8 = 80;
/// Recency factor reaches 0 at this many days since the last observation.
pub const RECENT_DATA_DAYS: i64 = 7;
/// `cnt_30d` floor — below this, quantity factor is 0.
pub const MIN_DATA_POINTS: i64 = 30;
/// CoV below this yields full 10 stability points.
pub const STABLE_COV_THRESHOLD: f64 = 0.05;
/// CoV at or above this yields 0 stability points.
pub const VOLATILE_COV_THRESHOLD: f64 = 0.20;

/// Price insight with classification (green/amber/hidden).
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct PriceInsight {
    pub level: String,
    pub confidence: u8,
    pub pct: f64,
    pub current_price: f64,
    pub min_30d: f64,
    pub avg_90d: f64,
}

/// Validate that a SKU is non-empty.
pub fn validate_sku(sku: &str) -> Result<(), String> {
    if sku.is_empty() {
        Err("sku_required".to_string())
    } else {
        Ok(())
    }
}

/// Core logic for `get_price_history`, extracted for testability without Tauri runtime.
pub async fn get_price_history_cmd(
    pool: &SqlitePool,
    sku: &str,
    window_days: u32,
) -> Result<Vec<PricePoint>, AppError> {
    validate_sku(sku).map_err(AppError::InvalidInput)?;
    let repo = PriceHistoryRepo::new(pool.clone());
    repo.get_history(sku, window_days)
        .await
        .map_err(|e| AppError::Database(e.to_string()))
}

/// Core logic for `get_price_insight`, extracted for testability.
pub async fn get_price_insight_cmd(
    pool: &SqlitePool,
    sku: &str,
) -> Result<Option<PriceInsight>, AppError> {
    validate_sku(sku).map_err(AppError::InvalidInput)?;
    let repo = PriceHistoryRepo::new(pool.clone());
    let row = repo.get_insight(sku).await.map_err(|e| AppError::Database(e.to_string()))?;

    let current_price = match row.current_price {
        Some(p) => p,
        None => return Ok(None),
    };

    // Build a single confidence value reused at every return site.
    let days_since_last = (row.now_epoch - row.last_recorded_at) / 86_400;
    let price_range_30d = row.max_30d - row.min_30d;
    let confidence = compute_confidence(
        row.cnt_30d,
        days_since_last,
        row.source_count_30d,
        price_range_30d,
        row.avg_30d,
    );

    // Classification logic per REQ-PI-4
    if row.cnt_30d < 30 {
        return Ok(Some(PriceInsight {
            level: "hidden".to_string(),
            confidence,
            pct: 0.0,
            current_price,
            min_30d: row.min_30d,
            avg_90d: row.avg_90d,
        }));
    }

    // Green: current price is near 30-day low
    if current_price <= row.min_30d * 1.05 {
        let pct = round_pct(((current_price - row.min_30d) / row.min_30d) * 100.0);
        return Ok(Some(PriceInsight {
            level: "green".to_string(),
            confidence,
            pct,
            current_price,
            min_30d: row.min_30d,
            avg_90d: row.avg_90d,
        }));
    }

    // Amber: current price exceeds 90-day average by 20%+
    if current_price >= row.avg_90d * 1.20 {
        let pct = round_pct(((current_price - row.avg_90d) / row.avg_90d) * 100.0);
        return Ok(Some(PriceInsight {
            level: "amber".to_string(),
            confidence,
            pct,
            current_price,
            min_30d: row.min_30d,
            avg_90d: row.avg_90d,
        }));
    }

    // Default: hidden (price is between green and amber thresholds)
    Ok(Some(PriceInsight {
        level: "hidden".to_string(),
        confidence,
        pct: 0.0,
        current_price,
        min_30d: row.min_30d,
        avg_90d: row.avg_90d,
    }))
}

/// Round to 2 decimal places.
fn round_pct(value: f64) -> f64 {
    (value * 100.0).round() / 100.0
}

/// Compute a 0-100 confidence score for a price insight.
///
/// Pure function: caller passes `days_since_last` (already derived from
/// `epoch_seconds`) so the function has no time source of its own.
///
/// Factor weights: quantity (50) + recency (25) + sources (15) + stability (10).
///
/// * `cnt_30d`         — observations in the 30-day window.
/// * `days_since_last` — whole days between now and the most recent observation.
/// * `source_count`    — distinct sources in the 30-day window.
/// * `price_range_30d` — `max_30d - min_30d` (used for CoV proxy).
/// * `avg_30d`         — mean price in the 30-day window (guard against 0).
pub fn compute_confidence(
    cnt_30d: i64,
    days_since_last: i64,
    source_count: i64,
    price_range_30d: f64,
    avg_30d: f64,
) -> u8 {
    // Quantity: 0 below MIN_DATA_POINTS, linear ramp to 50 at 90 points.
    let q = if cnt_30d < MIN_DATA_POINTS {
        0.0
    } else {
        ((cnt_30d.min(90) - MIN_DATA_POINTS) as f64 / 60.0) * 50.0
    };

    // Recency: full credit at 0 days, decays to 0 at RECENT_DATA_DAYS.
    let r = (1.0 - (days_since_last.clamp(0, RECENT_DATA_DAYS) as f64
        / RECENT_DATA_DAYS as f64))
        * 25.0;

    // Source diversity: 0/1 collapse to 5 (defensive against empty source_id).
    let s = match source_count {
        0 | 1 => 5.0,
        2 => 10.0,
        _ => 15.0,
    };

    // Stability: CoV = range/mean; guarded against avg=0.
    let cov = if avg_30d > 0.0 {
        price_range_30d / avg_30d
    } else {
        0.0
    };
    let st = if cov < STABLE_COV_THRESHOLD {
        10.0
    } else if cov < VOLATILE_COV_THRESHOLD {
        (1.0 - (cov - STABLE_COV_THRESHOLD)
            / (VOLATILE_COV_THRESHOLD - STABLE_COV_THRESHOLD))
            * 10.0
    } else {
        0.0
    };

    (q + r + s + st).round().clamp(0.0, 100.0) as u8
}

// ── Tauri IPC Commands ──────────────────────────────────────────────────

#[tauri::command]
pub async fn get_price_history(
    sku: String,
    window_days: Option<u32>,
    state: State<'_, AppState>,
) -> Result<Vec<PricePoint>, AppError> {
    get_price_history_cmd(&state.pool, &sku, window_days.unwrap_or(365)).await
}

#[tauri::command]
pub async fn get_price_insight(
    sku: String,
    state: State<'_, AppState>,
) -> Result<Option<PriceInsight>, AppError> {
    get_price_insight_cmd(&state.pool, &sku).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repository::price_history::{create_price_history_table, make_memory_pool};
    use crate::AppError;

    /// Real current epoch seconds — needed because PriceHistoryRepo uses
    /// real `epoch_seconds()` internally for window computation.
    fn now() -> i64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64
    }

    // ── validate_sku pure function tests ─────────────────────────────────

    #[test]
    fn validate_sku_rejects_empty() {
        assert_eq!(validate_sku(""), Err("sku_required".to_string()));
    }

    #[test]
    fn validate_sku_maps_to_app_error() {
        let err = validate_sku("").unwrap_err();
        let app_err = AppError::InvalidInput(err);
        assert!(matches!(app_err, AppError::InvalidInput(_)));
    }

    #[test]
    fn validate_sku_accepts_non_empty() {
        assert_eq!(validate_sku("ABC123"), Ok(()));
    }

    // ── get_price_history_cmd ────────────────────────────────────────────

    #[tokio::test]
    async fn get_price_history_cmd_empty_sku_returns_error() {
        let pool = make_memory_pool().await;
        create_price_history_table(&pool).await;
        let result = get_price_history_cmd(&pool, "", 365).await;
        assert!(matches!(result, Err(AppError::InvalidInput(_))));
    }

    #[tokio::test]
    async fn get_price_history_cmd_valid_sku_returns_points() {
        let pool = make_memory_pool().await;
        create_price_history_table(&pool).await;
        let t = now();

        // Seed a recent point
        sqlx::query(
            "INSERT INTO price_history (sku, price, recorded_at, source_id)
             VALUES (?1, ?2, ?3, ?4)",
        )
        .bind("SKU100")
        .bind(299.99f64)
        .bind(t - 10)
        .bind("reverb")
        .execute(&pool)
        .await
        .unwrap();

        let result = get_price_history_cmd(&pool, "SKU100", 365).await.unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].price, 299.99);
        assert_eq!(result[0].source_id, "reverb");
    }

    #[tokio::test]
    async fn get_price_history_cmd_no_data_returns_empty_vec() {
        let pool = make_memory_pool().await;
        create_price_history_table(&pool).await;
        let result = get_price_history_cmd(&pool, "NONEXISTENT", 365).await.unwrap();
        assert!(result.is_empty());
    }

    // ── get_price_insight_cmd ────────────────────────────────────────────

    #[tokio::test]
    async fn get_price_insight_cmd_empty_sku_returns_error() {
        let pool = make_memory_pool().await;
        create_price_history_table(&pool).await;
        let result = get_price_insight_cmd(&pool, "").await;
        assert!(matches!(result, Err(AppError::InvalidInput(_))));
    }

    #[tokio::test]
    async fn get_price_insight_cmd_no_data_returns_none() {
        let pool = make_memory_pool().await;
        create_price_history_table(&pool).await;
        let result = get_price_insight_cmd(&pool, "NONEXISTENT").await.unwrap();
        assert!(result.is_none(), "expected None for SKU with no data");
    }

    #[tokio::test]
    async fn get_price_insight_cmd_green_level() {
        // Spec Scenario 1: current_price near 30-day min, ≥30 points → green
        let pool = make_memory_pool().await;
        create_price_history_table(&pool).await;
        let t = now();

        // 45 points all at 100 → current=100, min_30d=100, 100 ≤ 100*1.05 → GREEN
        for i in 0..45 {
            sqlx::query(
                "INSERT INTO price_history (sku, price, recorded_at, source_id)
                 VALUES (?1, ?2, ?3, ?4)",
            )
            .bind("GREEN_SKU")
            .bind(100.0f64)
            .bind(t - (i as i64 * 86_400))
            .bind("reverb")
            .execute(&pool)
            .await
            .unwrap();
        }

        let result = get_price_insight_cmd(&pool, "GREEN_SKU")
            .await
            .unwrap()
            .expect("expected Some");

        assert_eq!(result.level, "green", "should be green");
        assert_eq!(result.pct, 0.0, "pct=0 when current equals min");
        assert_eq!(result.current_price, 100.0);
        assert_eq!(result.min_30d, 100.0);
    }

    #[tokio::test]
    async fn get_price_insight_cmd_amber_level() {
        // Spec Scenario 2: current_price=250, avg_90d=200, 100 data points → amber
        let pool = make_memory_pool().await;
        create_price_history_table(&pool).await;
        let t = now();

        // 100 points: avg ≈ 200, current = 250 (25% above avg, > 20% threshold)
        for i in 0..100 {
            let price = if i == 0 {
                250.0
            } else {
                200.0 + ((i % 10) as f64 - 5.0) * 2.0
            };
            sqlx::query(
                "INSERT INTO price_history (sku, price, recorded_at, source_id)
                 VALUES (?1, ?2, ?3, ?4)",
            )
            .bind("AMBER_SKU")
            .bind(price)
            .bind(t - (i as i64 * 86_400))
            .bind("reverb")
            .execute(&pool)
            .await
            .unwrap();
        }

        let result = get_price_insight_cmd(&pool, "AMBER_SKU")
            .await
            .unwrap()
            .expect("expected Some");

        assert_eq!(result.level, "amber", "should be amber");
        assert!(result.pct > 0.0, "pct should be positive");
        assert_eq!(result.current_price, 250.0);
    }

    #[tokio::test]
    async fn get_price_insight_cmd_hidden_when_under_30_rows() {
        // Spec Scenario 3: only 20 rows in 30d window → hidden
        let pool = make_memory_pool().await;
        create_price_history_table(&pool).await;
        let t = now();

        for i in 0..20 {
            sqlx::query(
                "INSERT INTO price_history (sku, price, recorded_at, source_id)
                 VALUES (?1, ?2, ?3, ?4)",
            )
            .bind("HIDDEN_SKU")
            .bind(100.0 + (i as f64))
            .bind(t - (i as i64 * 86_400))
            .bind("reverb")
            .execute(&pool)
            .await
            .unwrap();
        }

        let result = get_price_insight_cmd(&pool, "HIDDEN_SKU")
            .await
            .unwrap()
            .expect("expected Some");

        assert_eq!(
            result.level, "hidden",
            "should be hidden when <30 rows in 30d"
        );
        assert_eq!(result.pct, 0.0);
    }

    #[tokio::test]
    async fn get_price_insight_cmd_exact_green_threshold_boundary() {
        // price exactly at min_30d * 1.05 → green (≤ boundary)
        let pool = make_memory_pool().await;
        create_price_history_table(&pool).await;
        let t = now();

        // 35 points: min_30d = 100, current_price = 105 (= 100 * 1.05)
        for i in 0..35 {
            let price = if i == 0 {
                105.0
            } else {
                // Mix of values so min is definitely 100
                match i % 3 {
                    0 => 100.0,
                    1 => 102.0,
                    _ => 104.0,
                }
            };
            sqlx::query(
                "INSERT INTO price_history (sku, price, recorded_at, source_id)
                 VALUES (?1, ?2, ?3, ?4)",
            )
            .bind("GREEN_BOUNDARY")
            .bind(price)
            .bind(t - (i as i64 * 86_400))
            .bind("reverb")
            .execute(&pool)
            .await
            .unwrap();
        }

        let result = get_price_insight_cmd(&pool, "GREEN_BOUNDARY")
            .await
            .unwrap()
            .expect("expected Some");
        assert_eq!(
            result.level, "green",
            "price at min_30d * 1.05 should be green"
        );
        assert!(result.pct > 0.0, "pct should be positive");
    }

    #[tokio::test]
    async fn get_price_insight_cmd_exact_amber_threshold_boundary() {
        // price at avg_90d * 1.20 boundary → amber (≥ boundary)
        let pool = make_memory_pool().await;
        create_price_history_table(&pool).await;
        let t = now();

        // 100 points: avg_90d ≈ 100, current_price = 125 (> 100 * 1.20 = 120)
        for i in 0..100 {
            let price = if i == 0 {
                125.0
            } else {
                100.0
            };
            sqlx::query(
                "INSERT INTO price_history (sku, price, recorded_at, source_id)
                 VALUES (?1, ?2, ?3, ?4)",
            )
            .bind("AMBER_BOUNDARY")
            .bind(price)
            .bind(t - (i as i64 * 86_400))
            .bind("reverb")
            .execute(&pool)
            .await
            .unwrap();
        }

        let result = get_price_insight_cmd(&pool, "AMBER_BOUNDARY")
            .await
            .unwrap()
            .expect("expected Some");
        assert_eq!(
            result.level, "amber",
            "price above avg_90d * 1.20 should be amber"
        );
        assert!(result.pct > 0.0, "pct should be positive");
    }

    #[tokio::test]
    async fn get_price_insight_cmd_hidden_when_price_between_thresholds() {
        // price between green and amber thresholds → hidden
        let pool = make_memory_pool().await;
        create_price_history_table(&pool).await;
        let t = now();

        // min_30d=100, avg_90d≈100, current=110 (between 105 and 120)
        for i in 0..100 {
            let price = if i == 0 {
                110.0
            } else {
                100.0
            };
            sqlx::query(
                "INSERT INTO price_history (sku, price, recorded_at, source_id)
                 VALUES (?1, ?2, ?3, ?4)",
            )
            .bind("MID_SKU")
            .bind(price)
            .bind(t - (i as i64 * 86_400))
            .bind("reverb")
            .execute(&pool)
            .await
            .unwrap();
        }

        let result = get_price_insight_cmd(&pool, "MID_SKU")
            .await
            .unwrap()
            .expect("expected Some");
        assert_eq!(
            result.level, "hidden",
            "price between thresholds should be hidden"
        );
    }

    // ── compute_confidence: pure function unit tests ─────────────────────
    //
    // Test isolation: each test sets the factor under test to a known input
    // and pins the OTHER factors at their maximum contribution so the test
    // reads the isolated effect of the targeted factor.
    //
    // Maximum baseline (cnt=90, days=0, sources=3, range=0, avg=100) → 100.
    // Expected subtotals at maximum: q=50, r=25, s=15, st=10.

    // ── Quantity factor (max weight 50) ───────────────────────────────────

    #[test]
    fn compute_confidence_quantity_below_30_contributes_zero() {
        // cnt=0 with everything else max → q=0, total = 0+25+15+10 = 50
        let result = compute_confidence(0, 0, 3, 0.0, 100.0);
        assert_eq!(result, 50, "cnt<30 must yield zero quantity contribution");
    }

    #[test]
    fn compute_confidence_quantity_at_30_boundary_contributes_zero() {
        // cnt=30 is the floor of the linear ramp; min(90)-30=0 → q=0
        let result = compute_confidence(30, 0, 3, 0.0, 100.0);
        assert_eq!(result, 50, "cnt=30 is the boundary and yields 0 quantity");
    }

    #[test]
    fn compute_confidence_quantity_at_60_contributes_25() {
        // cnt=60 → (60-30)/60 * 50 = 25
        let result = compute_confidence(60, 0, 3, 0.0, 100.0);
        assert_eq!(result, 75, "cnt=60 should yield 25 quantity (total 75)");
    }

    #[test]
    fn compute_confidence_quantity_at_90_contributes_50() {
        // cnt=90 → (90-30)/60 * 50 = 50 (saturated)
        let result = compute_confidence(90, 0, 3, 0.0, 100.0);
        assert_eq!(result, 100, "cnt=90 should saturate quantity at 50");
    }

    #[test]
    fn compute_confidence_quantity_above_90_caps_at_50() {
        // cnt=120 must clamp at the 90 ceiling
        let result = compute_confidence(120, 0, 3, 0.0, 100.0);
        assert_eq!(result, 100, "cnt>90 should cap at the 50 max");
    }

    // ── Recency factor (max weight 25) ───────────────────────────────────

    #[test]
    fn compute_confidence_recency_zero_days_yields_25() {
        // days=0 → (1 - 0/7) * 25 = 25 (full recency)
        let result = compute_confidence(90, 0, 3, 0.0, 100.0);
        assert_eq!(result, 100, "days=0 should give full recency");
    }

    #[test]
    fn compute_confidence_recency_seven_days_yields_0() {
        // days=7 → (1 - 7/7) * 25 = 0
        let result = compute_confidence(90, 7, 3, 0.0, 100.0);
        assert_eq!(result, 75, "days=7 should zero out recency");
    }

    #[test]
    fn compute_confidence_recency_beyond_seven_stays_zero() {
        // days=14 (stale) → clamp to 7, recency still 0
        let result = compute_confidence(90, 14, 3, 0.0, 100.0);
        assert_eq!(result, 75, "days>=7 should keep recency at 0");
    }

    #[test]
    fn compute_confidence_recency_negative_clamped_to_full() {
        // days=-5 (clock skew) → clamp to 0 → full recency
        let result = compute_confidence(90, -5, 3, 0.0, 100.0);
        assert_eq!(result, 100, "negative days must clamp to 0 (full recency)");
    }

    // ── Source diversity factor (max weight 15) ──────────────────────────

    #[test]
    fn compute_confidence_sources_zero_yields_5() {
        // match 0 | 1 => 5
        let result = compute_confidence(90, 0, 0, 0.0, 100.0);
        assert_eq!(result, 90, "0 sources should yield 5 source points");
    }

    #[test]
    fn compute_confidence_sources_one_yields_5() {
        // 0 and 1 collapse to the same bucket (defensive vs source_id='')
        let result = compute_confidence(90, 0, 1, 0.0, 100.0);
        assert_eq!(result, 90, "1 source should yield 5 source points");
    }

    #[test]
    fn compute_confidence_sources_two_yields_10() {
        let result = compute_confidence(90, 0, 2, 0.0, 100.0);
        assert_eq!(result, 95, "2 sources should yield 10 source points");
    }

    #[test]
    fn compute_confidence_sources_three_or_more_yields_15() {
        let result = compute_confidence(90, 0, 3, 0.0, 100.0);
        assert_eq!(result, 100, "3+ sources should yield 15 source points");
    }

    // ── Stability factor (max weight 10) ─────────────────────────────────

    #[test]
    fn compute_confidence_stability_zero_range_yields_10() {
        // range=0, avg=100 → CoV=0, stable → 10
        let result = compute_confidence(90, 0, 3, 0.0, 100.0);
        assert_eq!(result, 100, "zero range should yield 10 stability points");
    }

    #[test]
    fn compute_confidence_stability_very_stable_yields_10() {
        // CoV = 4/100 = 0.04 < 0.05 → 10
        let result = compute_confidence(90, 0, 3, 4.0, 100.0);
        assert_eq!(result, 100, "CoV<0.05 should yield 10 stability points");
    }

    #[test]
    fn compute_confidence_stability_volatile_yields_0() {
        // CoV = 25/100 = 0.25 >= 0.20 → 0
        let result = compute_confidence(90, 0, 3, 25.0, 100.0);
        assert_eq!(result, 90, "CoV>=0.20 should zero out stability");
    }

    #[test]
    fn compute_confidence_stability_moderate_yields_linear() {
        // CoV = 12.5/100 = 0.125 → (1 - (0.125-0.05)/0.15) * 10 = 5
        let result = compute_confidence(90, 0, 3, 12.5, 100.0);
        assert_eq!(result, 95, "CoV=0.125 should yield 5 stability points");
    }

    #[test]
    fn compute_confidence_stability_avg_zero_guard_returns_10() {
        // avg=0 → guard sets CoV=0 → stab=10
        let result = compute_confidence(90, 0, 3, 10.0, 0.0);
        assert_eq!(result, 100, "avg=0 must guard CoV to 0 (stab=10)");
    }

    // ── Acceptance / scenario tests ──────────────────────────────────────

    #[test]
    fn compute_confidence_all_factors_max_returns_100() {
        let result = compute_confidence(90, 0, 3, 0.0, 100.0);
        assert_eq!(result, 100, "all max inputs must sum to 100");
    }

    #[test]
    fn compute_confidence_b1_scenario_60pts_1d_3src_stable() {
        // Spec B1: cnt=60, days=1, sources=3, low CoV
        // q=25, r=(1-1/7)*25=21.43, s=15, st=10 → total=71
        let result = compute_confidence(60, 1, 3, 0.0, 100.0);
        assert!(
            (70..=72).contains(&result),
            "B1 expected ~71, got {result}"
        );
    }

    #[test]
    fn compute_confidence_b2_scenario_45pts_3d_2src() {
        // Spec B2: cnt=45, days=3, sources=2
        // q=(45-30)/60*50=12.5, r=(1-3/7)*25=14.29, s=10, st=10 → total=47
        let result = compute_confidence(45, 3, 2, 0.0, 100.0);
        assert!(
            (46..=48).contains(&result),
            "B2 expected ~47, got {result}"
        );
    }

    #[test]
    fn compute_confidence_b3_scenario_30pts_6d_1src() {
        // Spec B3: cnt=30, days=6, sources=1
        // q=0 (boundary), r=(1-6/7)*25=3.57, s=5, st=10 → total=19
        let result = compute_confidence(30, 6, 1, 0.0, 100.0);
        assert!(
            result < 25,
            "B3 expected <25 (low tier territory), got {result}"
        );
    }

    #[test]
    fn compute_confidence_clamp_lower_bound_zero() {
        // Pathological: no points, deep volatility → must clamp to 0, not negative
        let result = compute_confidence(0, 14, 0, 100.0, 1.0);
        // q=0, r=0, s=5, cov=100 (volatile) → st=0, total=5
        assert_eq!(result, 5, "minimum configuration should clamp ≥0");
    }

    #[test]
    fn compute_confidence_clamp_upper_bound_100() {
        // Pathological: far more than 90 points with all factors maxed
        let result = compute_confidence(1000, 0, 10, 0.0, 100.0);
        assert_eq!(result, 100, "overshoot must clamp to 100");
    }

    // ── get_price_insight_cmd: confidence integration (RED — wired next) ─

    #[tokio::test]
    async fn get_price_insight_cmd_returns_confidence_in_payload_high() {
        // ≥30 points, 3 sources, last point today, all at same price → high
        let pool = make_memory_pool().await;
        create_price_history_table(&pool).await;
        let t = now();

        // 90 points, 3 sources, all at 100.0, one per second so all 30d-window
        for i in 0..90i64 {
            let source = match i % 3 {
                0 => "reverb",
                1 => "ebay",
                _ => "guitarcenter",
            };
            sqlx::query(
                "INSERT INTO price_history (sku, price, recorded_at, source_id)
                 VALUES (?1, ?2, ?3, ?4)",
            )
            .bind("HIGH_CONF_SKU")
            .bind(100.0f64)
            .bind(t - i)
            .bind(source)
            .execute(&pool)
            .await
            .unwrap();
        }

        let result = get_price_insight_cmd(&pool, "HIGH_CONF_SKU")
            .await
            .unwrap()
            .expect("expected Some");

        assert!(
            result.confidence >= 80,
            "high scenario expected confidence>=80, got {}",
            result.confidence
        );
        // Field is present on every return site (hidden, green, amber)
        assert!(
            result.confidence <= 100,
            "confidence must be ≤100, got {}",
            result.confidence
        );
    }

    #[tokio::test]
    async fn get_price_insight_cmd_stale_data_yields_low_confidence() {
        // 30 points, 1 source, last point 14d ago → recency factor = 0
        let pool = make_memory_pool().await;
        create_price_history_table(&pool).await;
        let t = now();

        for i in 0..30i64 {
            sqlx::query(
                "INSERT INTO price_history (sku, price, recorded_at, source_id)
                 VALUES (?1, ?2, ?3, ?4)",
            )
            .bind("STALE_SKU")
            .bind(100.0f64)
            .bind(t - (14 * 86_400) - (i * 86_400))
            .bind("reverb")
            .execute(&pool)
            .await
            .unwrap();
        }

        let result = get_price_insight_cmd(&pool, "STALE_SKU")
            .await
            .unwrap()
            .expect("expected Some");

        // recency=0 (14d) + quantity=0 (boundary) + sources=5 + stab=10 = 15
        assert_eq!(
            result.confidence, 15,
            "stale 30-pt single-source expected confidence=15, got {}",
            result.confidence
        );
    }

    #[tokio::test]
    async fn get_price_insight_cmd_under_threshold_yields_low_confidence() {
        // 20 points (<30) → quantity=0; with stable, recent, multi-source the
        // remaining factors can still hit 50; this asserts the LOW tier is
        // achievable and that quantity is genuinely zero (i.e. a 20-pt result
        // cannot exceed a 60-pt baseline by more than 25).
        let pool = make_memory_pool().await;
        create_price_history_table(&pool).await;
        let t = now();

        for i in 0..20i64 {
            sqlx::query(
                "INSERT INTO price_history (sku, price, recorded_at, source_id)
                 VALUES (?1, ?2, ?3, ?4)",
            )
            .bind("UNDER_SKU")
            .bind(100.0f64)
            .bind(t - (i * 86_400))
            .bind("reverb")
            .execute(&pool)
            .await
            .unwrap();
        }

        let result = get_price_insight_cmd(&pool, "UNDER_SKU")
            .await
            .unwrap()
            .expect("expected Some");

        // 20 pts, 1 src, last today, range=0 → 0+25+5+10 = 40
        assert_eq!(
            result.confidence, 40,
            "20-pt single-source stable should yield 40, got {}",
            result.confidence
        );
    }
}
