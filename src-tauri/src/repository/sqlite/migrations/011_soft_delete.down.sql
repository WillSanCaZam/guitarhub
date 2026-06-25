-- Rollback soft-delete columns from products_meta.
-- Drops is_active and delisted_at columns (SQLite ignores IF EXISTS for ALTER TABLE DROP COLUMN).

ALTER TABLE products_meta DROP COLUMN is_active;
ALTER TABLE products_meta DROP COLUMN delisted_at;
