use serde::Serialize;
use tauri::State;
use sqlx::SqlitePool;

use crate::repository::price_history::{PriceHistoryRepo, PricePoint};
use crate::AppError;
use crate::AppState;

/// Price insight with classification (green/amber/hidden).
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct PriceInsight {
    pub level: String,
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

    // Classification logic per REQ-PI-4
    if row.cnt_30d < 30 {
        return Ok(Some(PriceInsight {
            level: "hidden".to_string(),
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
            pct,
            current_price,
            min_30d: row.min_30d,
            avg_90d: row.avg_90d,
        }));
    }

    // Default: hidden (price is between green and amber thresholds)
    Ok(Some(PriceInsight {
        level: "hidden".to_string(),
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
}
