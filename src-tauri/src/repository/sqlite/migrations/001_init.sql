-- Migration 001: Initial schema
-- Creates the core tables for the GuitarHub offline catalog.

CREATE TABLE IF NOT EXISTS schema_meta (
  key   TEXT PRIMARY KEY,
  value TEXT NOT NULL
);

INSERT INTO schema_meta (key, value) VALUES ('db_version', '1')
ON CONFLICT(key) DO UPDATE SET value = excluded.value;

CREATE VIRTUAL TABLE IF NOT EXISTS products_fts USING fts5(
  sku, source_id, name, brand, model, category, subcategory, specs_json,
  tokenize = 'trigram'
);

CREATE TABLE IF NOT EXISTS products_meta (
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

CREATE TABLE IF NOT EXISTS sync_state (
  source_id        TEXT PRIMARY KEY,
  enabled          INTEGER DEFAULT 1,
  last_synced      INTEGER,
  last_run_id      TEXT,
  status           TEXT CHECK(status IN
                     ('idle','downloading','validating','sanitizing',
                      'inserting','done',
                      'failed_network','failed_schema','failed_db')),
  error_msg        TEXT
);

CREATE TABLE IF NOT EXISTS wishlist (
  sku          TEXT PRIMARY KEY,
  added_at     INTEGER NOT NULL,
  price_at_add REAL,
  notes        TEXT
);

CREATE TABLE IF NOT EXISTS price_history (
  id          INTEGER PRIMARY KEY AUTOINCREMENT,
  sku         TEXT NOT NULL,
  price       REAL NOT NULL,
  recorded_at INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_price_history_sku    ON price_history(sku);
CREATE INDEX IF NOT EXISTS idx_products_meta_source ON products_meta(source_id);
CREATE INDEX IF NOT EXISTS idx_products_meta_price  ON products_meta(price);
CREATE INDEX IF NOT EXISTS idx_products_meta_cond   ON products_meta(condition);
