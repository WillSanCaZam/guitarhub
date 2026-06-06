// SPDX-License-Identifier: GPL-3.0-or-later

//! Price-drop detection — pure, no I/O, unit-testable.
//!
//! Anti-spam is layered: this module handles **materiality** only
//! (was the drop big enough to care about?). The cooldown gate lives
//! in `repository/price_drop_notifications.rs` (PR 2) and is checked
//! by the caller in `services/sync.rs`.
//!
//! The detector is a pure function on purpose — no DB, no clock, no
//! side effects. Tests build a `Thresholds` and call `is_price_drop`
//! with literal numbers.

use serde::Serialize;

/// Default relative drop threshold: 10% (inclusive).
pub const RELATIVE_DROP_PCT: f64 = 0.10;

/// Default absolute drop threshold: $50 (inclusive).
pub const ABSOLUTE_DROP_USD: f64 = 50.0;

/// Default per-SKU cooldown: 24 hours in seconds.
pub const COOLDOWN_SECS: i64 = 86_400;

/// User-tunable drop thresholds.
///
/// Reads from `settings` in PR 2 (`drop_threshold_pct`, `drop_threshold_abs`,
/// `cooldown_hours`) and falls back to the module-level `pub const`
/// defaults below. Default-constructed `Thresholds` is what `sync.rs`
/// uses when settings are absent.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Thresholds {
    /// Relative drop, e.g. `0.10` = 10%.
    pub pct: f64,
    /// Absolute drop in USD, e.g. `50.0` = $50.
    pub abs: f64,
    /// Per-SKU cooldown in seconds.
    pub cooldown: i64,
}

impl Default for Thresholds {
    fn default() -> Self {
        Self {
            pct: RELATIVE_DROP_PCT,
            abs: ABSOLUTE_DROP_USD,
            cooldown: COOLDOWN_SECS,
        }
    }
}

/// Why a drop was reported. Set by `is_price_drop` based on which
/// threshold (relative or absolute) tripped first. When both would
/// fire, relative wins (pct is checked first in the detector).
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub enum DropReason {
    /// `pct_drop >= thresholds.pct` (relative threshold tripped).
    Relative,
    /// `abs_drop >= thresholds.abs` and the relative threshold did NOT trip
    /// (absolute threshold tripped alone).
    Absolute,
}

/// A detected price drop. `sku`, `channel`, and `reason` are populated
/// by the detector (or its caller) and surfaced in the dispatched
/// notification.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct PriceDrop {
    pub sku: String,
    pub previous_price: f64,
    pub new_price: f64,
    /// Which channel the dispatcher should use to notify
    /// ("app" | "ntfy" | "webhook"). Threaded through by the caller.
    pub channel: String,
    pub reason: DropReason,
}

/// Pure drop detector.
///
/// Returns `Some(PriceDrop)` if `previous_price` is `Some` AND
/// `new_price` is `Some` AND the new price is lower AND the drop
/// clears at least one of the thresholds (relative OR absolute,
/// both inclusive). Returns `None` otherwise.
///
/// Side-effect free: does not read clocks, databases, or globals.
/// Cooldown enforcement is the caller's responsibility
/// (`repository/price_drop_notifications.rs` in PR 2).
pub fn is_price_drop(
    sku: &str,
    new_price: Option<f64>,
    previous_price: Option<f64>,
    thresholds: &Thresholds,
    channel: &str,
) -> Option<PriceDrop> {
    let new = new_price?;
    let prev = previous_price?;
    if new >= prev {
        return None;
    }
    let abs_drop = prev - new;
    let pct_drop = abs_drop / prev;
    if pct_drop >= thresholds.pct {
        Some(PriceDrop {
            sku: sku.to_string(),
            previous_price: prev,
            new_price: new,
            channel: channel.to_string(),
            reason: DropReason::Relative,
        })
    } else if abs_drop >= thresholds.abs {
        Some(PriceDrop {
            sku: sku.to_string(),
            previous_price: prev,
            new_price: new,
            channel: channel.to_string(),
            reason: DropReason::Absolute,
        })
    } else {
        None
    }
}

// ── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn defaults() -> Thresholds {
        Thresholds::default()
    }

    // ── S1: Significant drop fires (20% / $200) ────────────────────────
    #[test]
    fn pdn_s1_significant_drop_fires() {
        let drop = is_price_drop("SKU-A", Some(800.0), Some(1000.0), &defaults(), "app");
        let drop = drop.expect("20% / $200 drop must fire");
        assert_eq!(drop.sku, "SKU-A");
        assert_eq!(drop.previous_price, 1000.0);
        assert_eq!(drop.new_price, 800.0);
        assert_eq!(drop.channel, "app");
        assert!(matches!(drop.reason, DropReason::Relative));
    }

    // ── S2: Small drop is suppressed (3% / $3) ──────────────────────────
    #[test]
    fn pdn_s2_small_drop_suppressed() {
        let drop = is_price_drop("SKU-B", Some(97.0), Some(100.0), &defaults(), "app");
        assert!(
            drop.is_none(),
            "3% / $3 drop must be suppressed, got {:?}",
            drop
        );
    }

    // ── S3: Boundary — exactly 10% fires (inclusive) ───────────────────
    #[test]
    fn pdn_s3_exact_ten_percent_fires() {
        let drop = is_price_drop("SKU-C", Some(90.0), Some(100.0), &defaults(), "app");
        assert!(
            drop.is_some(),
            "exact 10% drop must fire (inclusive boundary), got None"
        );
    }

    // ── S4: Boundary — exactly $50 fires (25% / $50) ───────────────────
    #[test]
    fn pdn_s4_exact_fifty_dollars_fires() {
        let drop = is_price_drop("SKU-D", Some(150.0), Some(200.0), &defaults(), "app");
        assert!(
            drop.is_some(),
            "exact $50 drop must fire (inclusive boundary), got None"
        );
    }

    // ── S5: Price increase is not a drop ───────────────────────────────
    #[test]
    fn pdn_s5_price_increase_no_drop() {
        let drop = is_price_drop("SKU-E", Some(120.0), Some(100.0), &defaults(), "app");
        assert!(
            drop.is_none(),
            "price increase must not be a drop, got {:?}",
            drop
        );
    }

    // ── S6: First observation (prev = None) is not a drop ──────────────
    #[test]
    fn pdn_s6_first_observation_no_drop() {
        let drop = is_price_drop("SKU-F", Some(100.0), None, &defaults(), "app");
        assert!(
            drop.is_none(),
            "first observation (prev=None) must not be a drop, got {:?}",
            drop
        );
    }

    // ── S7: new_price is None → no drop ────────────────────────────────
    #[test]
    fn pdn_s7_new_price_none_no_drop() {
        let drop = is_price_drop("SKU-G", None, Some(100.0), &defaults(), "app");
        assert!(
            drop.is_none(),
            "new_price=None must not be a drop, got {:?}",
            drop
        );
    }

    // ── S8: Both None → no drop ────────────────────────────────────────
    #[test]
    fn pdn_s8_both_none_no_drop() {
        let drop = is_price_drop("SKU-H", None, None, &defaults(), "app");
        assert!(
            drop.is_none(),
            "both None must not be a drop, got {:?}",
            drop
        );
    }

    // ── S9: 9.99% drop → just below relative threshold AND below $50 abs ──
    // Use a $200 baseline so 9.99% ≈ $19.98 abs — misses BOTH thresholds.
    #[test]
    fn pdn_s9_below_ten_percent_no_drop() {
        // prev=200, new=180.02 → 9.99% drop, $19.98 abs
        let drop = is_price_drop("SKU-I", Some(180.02), Some(200.0), &defaults(), "app");
        assert!(
            drop.is_none(),
            "9.99% / $19.98 drop must NOT fire (below both thresholds), got {:?}",
            drop
        );
    }

    // ── S10: 10.01% drop → just above threshold, fires ────────────────
    #[test]
    fn pdn_s10_just_above_ten_percent_fires() {
        // prev=200, new=179.98 → 10.01% drop, $20.02 abs
        let drop = is_price_drop("SKU-J", Some(179.98), Some(200.0), &defaults(), "app");
        assert!(
            drop.is_some(),
            "10.01% drop must fire (just above 10% threshold), got None"
        );
    }

    // ── S11: Absolute threshold trips when relative doesn't ────────────
    #[test]
    fn pdn_s11_absolute_threshold_fires() {
        let drop = is_price_drop("SKU-K", Some(1920.0), Some(2000.0), &defaults(), "ntfy");
        let drop = drop.expect("$80 drop on $2000 must fire (absolute threshold)");
        assert!(matches!(drop.reason, DropReason::Absolute));
        assert_eq!(drop.channel, "ntfy");
    }

    // ── S12: Custom thresholds — high pct + high abs both fail ──────────
    #[test]
    fn pdn_s12_custom_high_thresholds_suppress_normal_drop() {
        let high = Thresholds {
            pct: 0.50,
            abs: 5_000.0,
            cooldown: 86_400,
        };
        let drop = is_price_drop("SKU-L", Some(800.0), Some(1000.0), &high, "app");
        assert!(
            drop.is_none(),
            "20% / $200 drop must NOT fire with 50% / $5000 thresholds, got {:?}",
            drop
        );
    }

    // ── S13: Price unchanged (new == prev) → no drop ───────────────────
    #[test]
    fn pdn_s13_price_unchanged_no_drop() {
        let drop = is_price_drop("SKU-M", Some(100.0), Some(100.0), &defaults(), "app");
        assert!(
            drop.is_none(),
            "unchanged price must not be a drop, got {:?}",
            drop
        );
    }

    // ── S14: Constants are at the documented values ────────────────────
    #[test]
    fn pdn_s14_constants_match_design() {
        assert!((RELATIVE_DROP_PCT - 0.10).abs() < f64::EPSILON);
        assert!((ABSOLUTE_DROP_USD - 50.0).abs() < f64::EPSILON);
    }

    // ── S15: PriceDrop derives Serialize/Clone/PartialEq ───────────────
    #[test]
    fn pdn_s15_price_drop_traits_compile_and_compare() {
        let a = PriceDrop {
            sku: "X".to_string(),
            previous_price: 100.0,
            new_price: 80.0,
            channel: "app".to_string(),
            reason: DropReason::Relative,
        };
        let b = a.clone();
        assert_eq!(a, b, "Clone + PartialEq must round-trip");
    }
}
