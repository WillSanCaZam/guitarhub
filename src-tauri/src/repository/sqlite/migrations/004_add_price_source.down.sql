-- Down migration 004: Remove source_id from price_history
-- SQLite does not support DROP COLUMN, so we recreate the table.
CREATE TABLE IF NOT EXISTS price_history_v3 (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    sku         TEXT NOT NULL,
    price       REAL NOT NULL,
    recorded_at INTEGER NOT NULL
);

INSERT OR IGNORE INTO price_history_v3 (id, sku, price, recorded_at)
SELECT id, sku, price, recorded_at FROM price_history;

DROP TABLE IF EXISTS price_history;
ALTER TABLE price_history_v3 RENAME TO price_history;

CREATE INDEX IF NOT EXISTS idx_price_history_sku ON price_history(sku);
