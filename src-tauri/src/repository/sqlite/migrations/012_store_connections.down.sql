-- Rollback migration 012: drop user_id column and store_connections table.
-- Note: ALTER TABLE DROP COLUMN is supported in SQLite 3.35.0+.
-- If running an older SQLite, this will fail — manual migration required.

DROP INDEX IF EXISTS idx_products_meta_user_id;

ALTER TABLE products_meta DROP COLUMN user_id;

DROP TABLE IF EXISTS store_connections;
