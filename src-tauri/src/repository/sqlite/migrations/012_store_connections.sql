-- Add store_connections table for user-connected store accounts.
-- Each connection stores an encrypted token (AES-256-GCM) linked to a store.
-- UNIQUE(store_id) ensures at most one connection per store.

CREATE TABLE IF NOT EXISTS store_connections (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    store_id        TEXT NOT NULL,
    label           TEXT NOT NULL DEFAULT '',
    token_encrypted TEXT NOT NULL,
    username        TEXT,
    connected_at    INTEGER NOT NULL,
    synced_at       INTEGER,
    is_active       INTEGER DEFAULT 1,
    UNIQUE(store_id)
);

-- Add user_id column to products_meta so user-synced listings can be
-- distinguished from publicly-scraped products (user_id IS NULL = public).
ALTER TABLE products_meta ADD COLUMN user_id TEXT;

-- Index for fast filtering by user_id on catalog queries.
CREATE INDEX IF NOT EXISTS idx_products_meta_user_id ON products_meta(user_id);
