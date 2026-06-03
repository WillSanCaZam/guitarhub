-- Migration 003: Add image_cache table
-- Stores downloaded product images as SQLite BLOBs with LRU eviction and TTL.

CREATE TABLE IF NOT EXISTS image_cache (
    url_hash      TEXT PRIMARY KEY,
    blob          BLOB NOT NULL,
    mime_type     TEXT NOT NULL DEFAULT 'image/jpeg',
    size_bytes    INTEGER NOT NULL,
    last_accessed INTEGER NOT NULL,  -- Unix epoch
    created_at    INTEGER NOT NULL,  -- Unix epoch
    ttl_seconds   INTEGER NOT NULL DEFAULT 604800  -- 7 days
);

CREATE INDEX IF NOT EXISTS idx_image_cache_last_accessed ON image_cache(last_accessed);
