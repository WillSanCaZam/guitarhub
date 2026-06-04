-- Migration 002: Add URL validation CHECK constraints
--
-- Enforces that all stored URLs and image URLs use HTTPS.
-- This is a security hardening step: even if the ingestion pipeline
-- misses a non-https URL, the database will reject it at insert time.

-- Add CHECK constraint to products_meta.url
-- SQLite before 3.25.0 doesn't support ALTER TABLE ADD CONSTRAINT,
-- so we recreate the table with the constraint.

CREATE TABLE IF NOT EXISTS products_meta_new (
    sku          TEXT PRIMARY KEY,
    source_id    TEXT NOT NULL,
    price        REAL,
    currency     TEXT,
    condition    TEXT CHECK(condition IN ('new','used','refurbished','unknown')),
    availability TEXT CHECK(availability IN ('in_stock','out_of_stock','unknown')),
    url          TEXT NOT NULL CHECK(url LIKE 'https://%'),
    image_url    TEXT CHECK(image_url = '' OR image_url LIKE 'https://%'),
    seller       TEXT,
    location     TEXT,
    synced_at    INTEGER NOT NULL
);

-- Copy data from old table if it exists (it may not on fresh DBs)
INSERT OR IGNORE INTO products_meta_new
    SELECT * FROM products_meta;

-- Swap tables
DROP TABLE IF EXISTS products_meta;
ALTER TABLE products_meta_new RENAME TO products_meta;

-- Recreate indexes
CREATE INDEX IF NOT EXISTS idx_products_meta_source ON products_meta(source_id);
CREATE INDEX IF NOT EXISTS idx_products_meta_price  ON products_meta(price);
CREATE INDEX IF NOT EXISTS idx_products_meta_cond   ON products_meta(condition);
