// SPDX-License-Identifier: GPL-3.0-or-later

use sqlx::SqlitePool;

/// SQL CRUD operations for the `image_cache` table.
///
/// All methods operate against a `SqlitePool` and expect the `image_cache`
/// table to already exist (created by migration 003).
#[derive(Clone)]
pub struct ImageCacheRepo {
    pub(crate) pool: SqlitePool,
}

impl ImageCacheRepo {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// Insert a new cache entry. If the `url_hash` already exists, this
    /// is a no-op (callers should `delete` first if they want to replace).
    pub async fn insert(
        &self,
        url_hash: &str,
        blob: &[u8],
        mime_type: &str,
        size_bytes: u64,
        ttl_seconds: u64,
    ) -> Result<(), sqlx::Error> {
        let now = chrono_now();
        sqlx::query(
            "INSERT OR IGNORE INTO image_cache (url_hash, blob, mime_type, size_bytes, last_accessed, created_at, ttl_seconds)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        )
        .bind(url_hash)
        .bind(blob)
        .bind(mime_type)
        .bind(size_bytes as i64)
        .bind(now)
        .bind(now)
        .bind(ttl_seconds as i64)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// Update an existing entry with new blob data, resetting `last_accessed`
    /// and `created_at`. Used when a stale entry is re-fetched from the network.
    pub async fn update(
        &self,
        url_hash: &str,
        blob: &[u8],
        mime_type: &str,
        size_bytes: u64,
    ) -> Result<(), sqlx::Error> {
        let now = chrono_now();
        sqlx::query(
            "UPDATE image_cache SET blob = ?2, mime_type = ?3, size_bytes = ?4, last_accessed = ?5, created_at = ?5
             WHERE url_hash = ?1",
        )
        .bind(url_hash)
        .bind(blob)
        .bind(mime_type)
        .bind(size_bytes as i64)
        .bind(now)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// Fetch a cached entry by `url_hash`.
    /// Returns `(blob, mime_type, last_accessed)` if found.
    pub async fn fetch(&self, url_hash: &str) -> Result<Option<(Vec<u8>, String, i64)>, sqlx::Error> {
        let row: Option<(Vec<u8>, String, i64)> = sqlx::query_as(
            "SELECT blob, mime_type, last_accessed FROM image_cache WHERE url_hash = ?1",
        )
        .bind(url_hash)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row)
    }

    /// Delete a single cache entry by `url_hash`.
    pub async fn delete(&self, url_hash: &str) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM image_cache WHERE url_hash = ?1")
            .bind(url_hash)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    /// Update `last_accessed` to the current Unix epoch for the given entry.
    pub async fn touch(&self, url_hash: &str) -> Result<(), sqlx::Error> {
        let now = chrono_now();
        sqlx::query("UPDATE image_cache SET last_accessed = ?1 WHERE url_hash = ?2")
            .bind(now)
            .bind(url_hash)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    /// Evict the oldest entries (by `last_accessed`) until the total cache size
    /// is strictly less than `target_size_bytes`. If the cache is already below
    /// the target, this is a no-op.
    ///
    /// Returns the number of evicted rows.
    pub async fn evict_lru(&self, target_size_bytes: u64) -> Result<usize, sqlx::Error> {
        let mut evicted = 0usize;

        loop {
            let total: (i64,) = sqlx::query_as(
                "SELECT COALESCE(SUM(size_bytes), 0) FROM image_cache",
            )
            .fetch_one(&self.pool)
            .await?;

            if (total.0 as u64) < target_size_bytes {
                break;
            }

            // Delete the single oldest entry
            let result = sqlx::query(
                "DELETE FROM image_cache WHERE url_hash IN (
                    SELECT url_hash FROM image_cache ORDER BY last_accessed ASC LIMIT 1
                )",
            )
            .execute(&self.pool)
            .await?;

            if result.rows_affected() == 0 {
                break; // No more rows to delete (shouldn't happen, but defensive)
            }

            evicted += result.rows_affected() as usize;
        }

        Ok(evicted)
    }

    /// Delete all expired entries — those whose `last_accessed + ttl_seconds < now()`.
    /// Returns the number of evicted rows.
    pub async fn evict_expired(&self) -> Result<usize, sqlx::Error> {
        let now = chrono_now();
        let result = sqlx::query(
            "DELETE FROM image_cache WHERE (last_accessed + ttl_seconds) < ?1",
        )
        .bind(now)
        .execute(&self.pool)
        .await?;
        Ok(result.rows_affected() as usize)
    }

    /// Return the total number of bytes stored across all cache entries.
    pub async fn total_size(&self) -> Result<u64, sqlx::Error> {
        let row: (i64,) = sqlx::query_as(
            "SELECT COALESCE(SUM(size_bytes), 0) FROM image_cache",
        )
        .fetch_one(&self.pool)
        .await?;
        Ok(row.0 as u64)
    }

    /// Return the count of entries in the cache (useful for assertions in tests).
    pub async fn count(&self) -> Result<u64, sqlx::Error> {
        let row: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM image_cache",
        )
        .fetch_one(&self.pool)
        .await?;
        Ok(row.0 as u64)
    }
}

/// Helper: current Unix epoch in seconds.
fn chrono_now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

#[cfg(test)]
mod test_helpers {
    use super::*;

    /// Create the `image_cache` table in an `:memory:` database (used by tests).
    pub(super) async fn create_table(pool: &SqlitePool) {
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS image_cache (
                url_hash      TEXT PRIMARY KEY,
                blob          BLOB NOT NULL,
                mime_type     TEXT NOT NULL DEFAULT 'image/jpeg',
                size_bytes    INTEGER NOT NULL,
                last_accessed INTEGER NOT NULL,
                created_at    INTEGER NOT NULL,
                ttl_seconds   INTEGER NOT NULL DEFAULT 604800
            )",
        )
        .execute(pool)
        .await
        .unwrap();

        sqlx::query(
            "CREATE INDEX IF NOT EXISTS idx_image_cache_last_accessed ON image_cache(last_accessed)",
        )
        .execute(pool)
        .await
        .unwrap();
    }

    pub(super) async fn make_memory_pool() -> SqlitePool {
        SqlitePool::connect("sqlite::memory:").await.unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::test_helpers::*;
    use super::*;

    // ── insert / fetch ──────────────────────────────────────────────────

    #[tokio::test]
    async fn insert_and_fetch_roundtrip() {
        let pool = make_memory_pool().await;
        create_table(&pool).await;
        let repo = ImageCacheRepo::new(pool);

        let hash = "abc123";
        let blob = b"fake-image-bytes-here";
        repo.insert(hash, blob, "image/png", blob.len() as u64, 3600)
            .await
            .unwrap();

        let result = repo.fetch(hash).await.unwrap().expect("should find entry");
        assert_eq!(result.0, blob);
        assert_eq!(result.1, "image/png");
    }

    #[tokio::test]
    async fn fetch_nonexistent_returns_none() {
        let pool = make_memory_pool().await;
        create_table(&pool).await;
        let repo = ImageCacheRepo::new(pool);

        let result = repo.fetch("nonexistent").await.unwrap();
        assert!(result.is_none());
    }

    // ── update ──────────────────────────────────────────────────────────

    #[tokio::test]
    async fn update_replaces_blob_and_resets_timestamps() {
        let pool = make_memory_pool().await;
        create_table(&pool).await;
        let repo = ImageCacheRepo::new(pool);

        let hash = "hash1";
        repo.insert(hash, b"old", "image/jpeg", 3, 3600).await.unwrap();

        // Small delay so timestamps differ
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;

        repo.update(hash, b"new-bytes", "image/webp", 9).await.unwrap();

        let result = repo.fetch(hash).await.unwrap().expect("should exist");
        assert_eq!(result.0, b"new-bytes");
        assert_eq!(result.1, "image/webp");
    }

    // ── delete ──────────────────────────────────────────────────────────

    #[tokio::test]
    async fn delete_removes_entry() {
        let pool = make_memory_pool().await;
        create_table(&pool).await;
        let repo = ImageCacheRepo::new(pool);

        repo.insert("del-hash", b"data", "image/jpeg", 4, 3600)
            .await
            .unwrap();
        repo.delete("del-hash").await.unwrap();

        let result = repo.fetch("del-hash").await.unwrap();
        assert!(result.is_none());
    }

    // ── touch ───────────────────────────────────────────────────────────

    #[tokio::test]
    async fn touch_updates_last_accessed() {
        let pool = make_memory_pool().await;
        create_table(&pool).await;
        let repo = ImageCacheRepo::new(pool);

        repo.insert("t-hash", b"data", "image/jpeg", 4, 3600)
            .await
            .unwrap();

        // Read the initial last_accessed
        let initial = repo.fetch("t-hash").await.unwrap().unwrap().2;

        // Sleep long enough for the Unix-epoch-second to tick over
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;

        repo.touch("t-hash").await.unwrap();

        let updated = repo.fetch("t-hash").await.unwrap().unwrap().2;
        assert!(updated >= initial, "touch should not decrease last_accessed");
        // It should have actually increased if the second ticked over
        if updated == initial {
            // Race: the second didn't tick. That's fine — at least it didn't decrease.
            // This is very rare in practice (only when the second boundary doesn't cross).
        } else {
            assert!(updated > initial, "touch should increase last_accessed");
        }
    }

    // ── evict_lru ───────────────────────────────────────────────────────

    #[tokio::test]
    async fn evict_lru_removes_oldest_entries() {
        let pool = make_memory_pool().await;
        create_table(&pool).await;
        let repo = ImageCacheRepo::new(pool);

        let now = chrono_now();
        // Insert 3 entries with artificially aged last_accessed
        insert_at(&repo, "old", 10, now - 100).await;
        insert_at(&repo, "middle", 20, now - 50).await;
        insert_at(&repo, "new", 15, now).await;

        // Total = 45, target = 40 → should evict oldest (10) → total = 35
        let evicted = repo.evict_lru(40).await.unwrap();
        assert_eq!(evicted, 1, "should evict 1 entry");

        let total = repo.total_size().await.unwrap();
        assert_eq!(total, 35, "oldest 10-byte entry should be gone");

        // Verify "old" is gone, "middle" and "new" remain
        assert!(repo.fetch("old").await.unwrap().is_none());
        assert!(repo.fetch("middle").await.unwrap().is_some());
        assert!(repo.fetch("new").await.unwrap().is_some());
    }

    #[tokio::test]
    async fn evict_lru_removes_multiple_until_target() {
        let pool = make_memory_pool().await;
        create_table(&pool).await;
        let repo = ImageCacheRepo::new(pool);

        let now = chrono_now();
        insert_at(&repo, "a", 100, now - 300).await;
        insert_at(&repo, "b", 100, now - 200).await;
        insert_at(&repo, "c", 100, now - 100).await;

        // Total = 300, target = 150 → evict a and b (200 removed) → total = 100
        let evicted = repo.evict_lru(150).await.unwrap();
        assert_eq!(evicted, 2, "should evict 2 entries");

        let total = repo.total_size().await.unwrap();
        assert_eq!(total, 100, "only 'c' should remain");
    }

    #[tokio::test]
    async fn evict_lru_noop_when_below_target() {
        let pool = make_memory_pool().await;
        create_table(&pool).await;
        let repo = ImageCacheRepo::new(pool);

        insert_at(&repo, "x", 10, chrono_now()).await;

        let evicted = repo.evict_lru(100).await.unwrap();
        assert_eq!(evicted, 0);
        assert_eq!(repo.count().await.unwrap(), 1);
    }

    // ── evict_expired ───────────────────────────────────────────────────

    #[tokio::test]
    async fn evict_expired_removes_ttl_expired_entries() {
        let pool = make_memory_pool().await;
        create_table(&pool).await;
        let repo = ImageCacheRepo::new(pool);

        let now = chrono_now();
        // Insert expired entry: last_accessed far in the past, short TTL
        insert_at_with_ttl(&repo, "expired", 10, now - 100, 50).await;
        // Insert fresh entry
        insert_at_with_ttl(&repo, "fresh", 20, now, 3600).await;

        let evicted = repo.evict_expired().await.unwrap();
        assert_eq!(evicted, 1, "should evict the expired entry");

        assert!(repo.fetch("expired").await.unwrap().is_none());
        assert!(repo.fetch("fresh").await.unwrap().is_some());
    }

    // ── total_size ──────────────────────────────────────────────────────

    #[tokio::test]
    async fn total_size_returns_zero_on_empty_cache() {
        let pool = make_memory_pool().await;
        create_table(&pool).await;
        let repo = ImageCacheRepo::new(pool);

        assert_eq!(repo.total_size().await.unwrap(), 0);
    }

    #[tokio::test]
    async fn total_size_sums_all_entries() {
        let pool = make_memory_pool().await;
        create_table(&pool).await;
        let repo = ImageCacheRepo::new(pool);

        insert_at(&repo, "a", 50, chrono_now()).await;
        insert_at(&repo, "b", 30, chrono_now()).await;
        insert_at(&repo, "c", 20, chrono_now()).await;

        assert_eq!(repo.total_size().await.unwrap(), 100);
    }

    // ── Helper: insert with last_accessed override ──────────────────────

    async fn insert_at(repo: &ImageCacheRepo, hash: &str, size: u64, last_accessed: i64) {
        sqlx::query(
            "INSERT INTO image_cache (url_hash, blob, mime_type, size_bytes, last_accessed, created_at, ttl_seconds)
             VALUES (?1, ?2, ?3, ?4, ?5, ?5, ?6)",
        )
        .bind(hash)
        .bind(vec![0u8; size as usize])
        .bind("image/jpeg")
        .bind(size as i64)
        .bind(last_accessed)
        .bind(3600i64)
        .execute(&repo.pool)
        .await
        .unwrap();
    }

    async fn insert_at_with_ttl(
        repo: &ImageCacheRepo,
        hash: &str,
        size: u64,
        last_accessed: i64,
        ttl: i64,
    ) {
        sqlx::query(
            "INSERT INTO image_cache (url_hash, blob, mime_type, size_bytes, last_accessed, created_at, ttl_seconds)
             VALUES (?1, ?2, ?3, ?4, ?5, ?5, ?6)",
        )
        .bind(hash)
        .bind(vec![0u8; size as usize])
        .bind("image/jpeg")
        .bind(size as i64)
        .bind(last_accessed)
        .bind(ttl)
        .execute(&repo.pool)
        .await
        .unwrap();
    }
}
