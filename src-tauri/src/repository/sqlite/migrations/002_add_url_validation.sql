-- Migration 002: Add URL validation CHECK constraints
--
-- Enforces that all stored URLs and image URLs use HTTPS.
-- This is a security hardening step: even if the ingestion pipeline
-- misses a non-https URL, the database will reject it at insert time.
--
-- IMPORTANT: This migration preserves all 17 columns from products_meta.
-- The previous version dropped 6 columns (name, brand, model, category,
-- subcategory, specs_json) which caused data loss.
--
-- Corrupted-DB guard: if products_meta exists with < 17 columns (from a
-- previous broken migration), the DROP TABLE below cleans it up and the
-- full rewrite restores the correct schema. Pre-release: no deployed users.

-- Recreate products_meta with CHECK constraints and all 17 columns
CREATE TABLE IF NOT EXISTS products_meta_new (
    sku          TEXT PRIMARY KEY,
    source_id    TEXT NOT NULL,
    name         TEXT NOT NULL DEFAULT '',
    brand        TEXT NOT NULL DEFAULT '',
    model        TEXT NOT NULL DEFAULT '',
    category     TEXT NOT NULL DEFAULT '',
    subcategory  TEXT NOT NULL DEFAULT '',
    specs_json   TEXT NOT NULL DEFAULT '{}',
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

-- Copy data using explicit column list (not SELECT *) to prevent column mismatch
INSERT OR IGNORE INTO products_meta_new
    (sku, source_id, name, brand, model, category, subcategory, specs_json,
     price, currency, condition, availability, url, image_url, seller, location, synced_at)
SELECT
    sku, source_id, name, brand, model, category, subcategory, specs_json,
    price, currency, condition, availability, url, image_url, seller, location, synced_at
FROM products_meta;

-- Swap tables
DROP TABLE IF EXISTS products_meta;
ALTER TABLE products_meta_new RENAME TO products_meta;

-- Recreate FTS5 virtual table (must be recreated after table swap)
DROP TABLE IF EXISTS products_fts;
CREATE VIRTUAL TABLE IF NOT EXISTS products_fts USING fts5(
    sku, source_id, name, brand, model, category, subcategory, specs_json,
    tokenize = 'trigram',
    content = 'products_meta',
    content_rowid = 'rowid'
);

-- Recreate FTS5 sync triggers
CREATE TRIGGER IF NOT EXISTS products_fts_ai AFTER INSERT ON products_meta BEGIN
    INSERT INTO products_fts(rowid, sku, source_id, name, brand, model, category, subcategory, specs_json)
    VALUES (new.rowid, new.sku, new.source_id, new.name, new.brand, new.model, new.category, new.subcategory, new.specs_json);
END;
CREATE TRIGGER IF NOT EXISTS products_fts_ad AFTER DELETE ON products_meta BEGIN
    INSERT INTO products_fts(products_fts, rowid, sku, source_id, name, brand, model, category, subcategory, specs_json)
    VALUES ('delete', old.rowid, old.sku, old.source_id, old.name, old.brand, old.model, old.category, old.subcategory, old.specs_json);
END;
CREATE TRIGGER IF NOT EXISTS products_fts_au AFTER UPDATE ON products_meta BEGIN
    INSERT INTO products_fts(products_fts, rowid, sku, source_id, name, brand, model, category, subcategory, specs_json)
    VALUES ('delete', old.rowid, old.sku, old.source_id, old.name, old.brand, old.model, old.category, old.subcategory, old.specs_json);
    INSERT INTO products_fts(rowid, sku, source_id, name, brand, model, category, subcategory, specs_json)
    VALUES (new.rowid, new.sku, new.source_id, new.name, new.brand, new.model, new.category, new.subcategory, new.specs_json);
END;

-- Recreate indexes
CREATE INDEX IF NOT EXISTS idx_products_meta_source ON products_meta(source_id);
CREATE INDEX IF NOT EXISTS idx_products_meta_price  ON products_meta(price);
CREATE INDEX IF NOT EXISTS idx_products_meta_cond   ON products_meta(condition);
