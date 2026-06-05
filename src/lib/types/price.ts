// SPDX-License-Identifier: GPL-3.0-or-later
//
// TypeScript mirror of the Rust `PriceInsight` IPC payload from
// `src-tauri/src/commands/price_command.rs`. Keep field names and
// types in sync — drift here surfaces as runtime `undefined` in the UI.

export type InsightLevel = "green" | "amber" | "hidden";

export type ConfidenceTier = "high" | "medium" | "low";

/**
 * Price insight payload delivered by the `get_price_insight` Tauri command.
 * `confidence` is `0..=100`; `tier` is derived in the UI, not stored.
 */
export interface PriceInsight {
  level: InsightLevel;
  /** Server-computed reliability score in `[0, 100]`. */
  confidence: number;
  /** Signed percentage relative to the reference (min_30d for green, avg_90d for amber). */
  pct: number;
  current_price: number;
  min_30d: number;
  avg_90d: number;
}
