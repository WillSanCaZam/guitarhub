-- Down migration 001: Undo initial schema
DROP TABLE IF EXISTS price_history;
DROP TABLE IF EXISTS wishlist;
DROP TABLE IF EXISTS sync_state;
DROP TABLE IF EXISTS products_fts;
DROP TABLE IF EXISTS products_meta;
DROP TABLE IF EXISTS schema_meta;
