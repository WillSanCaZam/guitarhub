-- Migration 001: Initial schema
-- Creates the core tables for the GuitarHub offline catalog.

CREATE TABLE IF NOT EXISTS schema_meta (
  key   TEXT PRIMARY KEY,
  value TEXT NOT NULL
);

INSERT INTO schema_meta (key, value) VALUES ('db_version', '1')
ON CONFLICT(key) DO UPDATE SET value = excluded.value;

CREATE TABLE IF NOT EXISTS products_meta (
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

CREATE VIRTUAL TABLE IF NOT EXISTS products_fts USING fts5(
  sku, source_id, name, brand, model, category, subcategory, specs_json,
  tokenize = 'trigram',
  content = 'products_meta',
  content_rowid = 'rowid'
);

-- Triggers keep the FTS index (tokenizer data) in sync with products_meta.
-- FTS5 external content mode reads the actual column values from products_meta at query time,
-- so the triggers only need to maintain the index, not duplicate the content.
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
