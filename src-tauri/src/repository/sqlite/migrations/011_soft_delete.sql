-- Add soft-delete columns to products_meta for delisting detection.
-- is_active: 1 (active, default), 0 (delisted — no longer present in the source).
-- delisted_at: Unix epoch timestamp when sync first detected absence. NULL while active.
-- Existing rows get is_active = 1, delisted_at = NULL — no data migration needed.

ALTER TABLE products_meta ADD COLUMN is_active    INTEGER DEFAULT 1;
ALTER TABLE products_meta ADD COLUMN delisted_at  INTEGER;
