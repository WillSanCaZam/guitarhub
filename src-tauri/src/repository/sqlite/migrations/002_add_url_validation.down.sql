-- Down migration 002: Remove URL CHECK constraints, restore original schema
-- Recreate products_meta without CHECK constraints (matching 001_init output).
CREATE TABLE IF NOT EXISTS products_meta_v1 (
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
    condition    TEXT,
    availability TEXT,
    url          TEXT NOT NULL,
    image_url    TEXT,
    seller       TEXT,
    location     TEXT,
    synced_at    INTEGER NOT NULL
);

INSERT OR IGNORE INTO products_meta_v1
    (sku, source_id, name, brand, model, category, subcategory, specs_json,
     price, currency, condition, availability, url, image_url, seller, location, synced_at)
SELECT sku, source_id, name, brand, model, category, subcategory, specs_json,
       price, currency, condition, availability, url, image_url, seller, location, synced_at
FROM products_meta;

DROP TABLE IF EXISTS products_meta;
ALTER TABLE products_meta_v1 RENAME TO products_meta;

-- Recreate FTS5 without the CHECK-dependent references (same schema as 001).
DROP TABLE IF EXISTS products_fts;
CREATE VIRTUAL TABLE IF NOT EXISTS products_fts USING fts5(
    sku, source_id, name, brand, model, category, subcategory, specs_json,
    tokenize = 'trigram',
    content = 'products_meta',
    content_rowid = 'rowid'
);

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

-- Recreate indexes (matching 001).
CREATE INDEX IF NOT EXISTS idx_price_history_sku    ON price_history(sku);
CREATE INDEX IF NOT EXISTS idx_products_meta_source ON products_meta(source_id);
CREATE INDEX IF NOT EXISTS idx_products_meta_price  ON products_meta(price);
CREATE INDEX IF NOT EXISTS idx_products_meta_cond   ON products_meta(condition);
