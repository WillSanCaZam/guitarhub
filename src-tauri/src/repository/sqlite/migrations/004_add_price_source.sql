-- Migration 004: Add source_id to price_history
-- Enables multi-source per-SKU price chart lines.

ALTER TABLE price_history ADD COLUMN source_id TEXT NOT NULL DEFAULT '';

CREATE INDEX IF NOT EXISTS idx_price_history_sku_recorded
  ON price_history(sku, recorded_at);
