-- Down migration 006: Restore original wishlist schema (v1 with sku PK)
CREATE TABLE IF NOT EXISTS wishlist_v1 (
    sku          TEXT PRIMARY KEY,
    added_at     INTEGER NOT NULL,
    price_at_add REAL,
    notes        TEXT
);

INSERT OR IGNORE INTO wishlist_v1 (sku, added_at, notes)
SELECT sku, added_at, notes FROM wishlist;

DROP TABLE IF EXISTS wishlist;
ALTER TABLE wishlist_v1 RENAME TO wishlist;
