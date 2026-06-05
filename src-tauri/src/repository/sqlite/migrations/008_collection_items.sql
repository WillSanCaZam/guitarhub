-- Migration 008: Collection items table
--
-- Tracks user-owned guitar gear with purchase details and condition.
-- Estimated values are computed at query time from price_history.
--
-- Columns:
--   id                INTEGER PRIMARY KEY AUTOINCREMENT
--   sku               TEXT              — optional link to catalog SKU
--   name              TEXT NOT NULL     — user-facing item name
--   brand             TEXT              — manufacturer
--   purchase_price    REAL              — what the user paid
--   purchase_currency TEXT DEFAULT 'USD'
--   purchase_date     INTEGER           — unix epoch seconds
--   condition         TEXT CHECK(condition IN ('mint','excellent','good','fair','poor'))
--   serial_number     TEXT
--   notes             TEXT
--   image_url         TEXT
--   added_at          INTEGER NOT NULL  — when item entered collection

CREATE TABLE IF NOT EXISTS collection_items (
    id                INTEGER PRIMARY KEY AUTOINCREMENT,
    sku               TEXT,
    name              TEXT NOT NULL,
    brand             TEXT,
    purchase_price    REAL,
    purchase_currency TEXT DEFAULT 'USD',
    purchase_date     INTEGER,
    condition         TEXT CHECK(condition IN ('mint','excellent','good','fair','poor')),
    serial_number     TEXT,
    notes             TEXT,
    image_url         TEXT,
    added_at          INTEGER NOT NULL
);

-- Index for SKU lookups (used by estimated_value and export)
CREATE INDEX IF NOT EXISTS idx_collection_items_sku ON collection_items(sku);
