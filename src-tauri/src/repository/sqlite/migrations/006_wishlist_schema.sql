-- Migration 006: Recreate wishlist table with full schema
--
-- The original wishlist table (from 001) has only 4 columns:
--   sku, added_at, price_at_add, notes
-- The export_service WishlistRow struct expects 10 columns:
--   id, sku, name, brand, price, currency, image_url, product_url, notes, added_at
--
-- This migration recreates the table with all 10 columns, migrating
-- existing data (sku, added_at, notes) and setting new columns to NULL.
-- PK changes from sku to id (autoincrement) to match WishlistRow.

-- Recreate with full schema (PK changes from sku to id)
CREATE TABLE IF NOT EXISTS wishlist_v2 (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    sku         TEXT,
    name        TEXT,
    brand       TEXT,
    price       REAL,
    currency    TEXT,
    image_url   TEXT,
    product_url TEXT,
    notes       TEXT,
    added_at    INTEGER
);

-- Migrate existing data (only sku, added_at, notes survive; new cols get NULL)
INSERT INTO wishlist_v2 (sku, added_at, notes)
    SELECT sku, added_at, notes FROM wishlist;

-- Swap tables
DROP TABLE IF EXISTS wishlist;
ALTER TABLE wishlist_v2 RENAME TO wishlist;
