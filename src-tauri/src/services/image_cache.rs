// SPDX-License-Identifier: GPL-3.0-or-later

use std::collections::HashSet;
use std::sync::Arc;
use std::time::Duration;

use dashmap::DashMap;
use reqwest::Client;
use sha2::{Digest, Sha256};
use sqlx::SqlitePool;
use tokio::sync::{watch, RwLock};

use crate::repository::settings::SettingsRepository;
use crate::repository::sqlite::image_cache::ImageCacheRepo;

// Defaults
const DEFAULT_MAX_BYTES: u64 = 50 * 1024 * 1024; // 50 MB
const DEFAULT_TTL: Duration = Duration::from_secs(7 * 24 * 60 * 60); // 7 days
const MAX_DOWNLOAD_SIZE: u64 = 50 * 1024 * 1024; // Reject individual files > 50 MB

/// Public API for image caching.
///
/// Provides a single `get(url)` method that:
/// - Returns cached images (SQLite BLOBs) as base64 data URIs
/// - Fetches and caches on first access
/// - Coalesces concurrent requests for the same URL into one HTTP call
/// - Enforces LRU eviction when cache exceeds size limit
/// - Falls back to stale blobs when offline
pub struct ImageCacheService {
    pool: SqlitePool,
    repo: ImageCacheRepo,
    settings_repo: Arc<dyn SettingsRepository>,
    max_bytes: u64,
    default_ttl: Duration,
    http: Client,
    in_flight: Arc<DashMap<String, InFlightTx>>,
    failed_urls: Arc<RwLock<HashSet<String>>>,
}

/// Type alias for the watch-based in-flight channel used for request coalescing.
type InFlightTx = watch::Sender<Option<Result<(Vec<u8>, String), ImageCacheError>>>;

#[derive(Debug, Clone)]
pub enum ImageCacheError {
    InvalidUrl(String),
    DownloadFailed(String),
    Oversized { size: u64, limit: u64 },
    Placeholder,
}

impl std::fmt::Display for ImageCacheError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidUrl(url) => write!(f, "invalid image URL: {url}"),
            Self::DownloadFailed(msg) => write!(f, "download failed: {msg}"),
            Self::Oversized { size, limit } => {
                write!(f, "image too large: {size} bytes exceeds limit of {limit} bytes")
            }
            Self::Placeholder => write!(f, "placeholder (no cache available)"),
        }
    }
}

impl Clone for ImageCacheService {
    fn clone(&self) -> Self {
        Self {
            pool: self.pool.clone(),
            repo: ImageCacheRepo::new(self.pool.clone()),
            settings_repo: self.settings_repo.clone(),
            max_bytes: self.max_bytes,
            default_ttl: self.default_ttl,
            http: self.http.clone(),
            in_flight: self.in_flight.clone(),
            failed_urls: self.failed_urls.clone(),
        }
    }
}

impl ImageCacheService {
    pub fn new(
        pool: SqlitePool,
        max_bytes: u64,
        default_ttl: Duration,
        settings_repo: Arc<dyn SettingsRepository>,
    ) -> Self {
        let http = Client::builder()
            .timeout(Duration::from_secs(30))
            .user_agent("GuitarHub/0.1")
            .build()
            .expect("reqwest Client should build");

        Self {
            repo: ImageCacheRepo::new(pool.clone()),
            pool,
            settings_repo,
            max_bytes,
            default_ttl,
            http,
            in_flight: Arc::new(DashMap::new()),
            failed_urls: Arc::new(RwLock::new(HashSet::new())),
        }
    }

    /// Convenience constructor with default 50 MB limit and 7-day TTL.
    pub fn new_default(pool: SqlitePool, settings_repo: Arc<dyn SettingsRepository>) -> Self {
        Self::new(pool, DEFAULT_MAX_BYTES, DEFAULT_TTL, settings_repo)
    }

    /// Core method: return cached image or fetch + store.
    ///
    /// Returns `(image_bytes, mime_type)`.
    pub async fn get(&self, url: &str) -> Result<(Vec<u8>, String), ImageCacheError> {
        // 1. Validate URL
        if !url.starts_with("https://") {
            // Accept http:// for local dev/testing but log a warning
            if url.starts_with("http://") {
                tracing::warn!("Fetching non-HTTPS image URL: {url}");
            } else {
                return Err(ImageCacheError::InvalidUrl(url.to_string()));
            }
        }

        // 2. Hash the URL for cache key
        let hash = hex_sha256(url);

        // 3. Check in_flight for request coalescing.
        //
        // CRITICAL: We must drop the DashMap entry guard BEFORE awaiting,
        // otherwise the resolver (first request) cannot acquire the same
        // DashMap shard to send the result.
        match self.in_flight.entry(hash.clone()) {
            dashmap::mapref::entry::Entry::Occupied(occupied) => {
                // Another request is already fetching — subscribe to its
                // watch channel and drop the DashMap guard before awaiting.
                let tx = occupied.get().clone();
                let mut rx = tx.subscribe();
                drop(occupied);

                rx.wait_for(|v| v.is_some())
                    .await
                    .expect("in_flight watch channel closed unexpectedly");
                return rx
                    .borrow()
                    .as_ref()
                    .expect("result should be Some after wait_for")
                    .clone();
            }
            dashmap::mapref::entry::Entry::Vacant(vacant) => {
                let (tx, _rx) = watch::channel(None);
                vacant.insert(tx.clone());
            }
        }

        // We are the designated fetcher. Do the work, send the result
        // through the watch channel, then remove from in_flight.
        let result = self.get_inner(url, &hash).await;
        if let Some(tx) = self.in_flight.get(&hash) {
            let _ = tx.send(Some(result.clone()));
        }
        self.in_flight.remove(&hash);

        result
    }

    /// The actual fetch/cache logic (called exactly once per unique URL).
    async fn get_inner(
        &self,
        url: &str,
        hash: &str,
    ) -> Result<(Vec<u8>, String), ImageCacheError> {
        // Check cache first
        let cached = self.repo.fetch(hash).await.map_err(|e| {
            ImageCacheError::DownloadFailed(format!("DB fetch error: {e}"))
        })?;

        let now_secs = chrono_now();

        match cached {
            // ── Cache HIT ───────────────────────────────────────────────
            Some((blob, mime_type, last_accessed)) => {
                let age_secs = now_secs - last_accessed;
                let is_stale = age_secs > 0 && {
                    // We need the TTL for this entry to determine staleness.
                    // Query it from the DB.
                    let ttl = self.get_ttl(hash).await.unwrap_or(self.default_ttl.as_secs() as i64);
                    age_secs > ttl
                };

                if is_stale {
                    // Check domain allowlist before re-fetching
                    let allowed = get_allowed_domains_from_repo(&*self.settings_repo).await;
                    if !is_domain_in_list(url, &allowed) {
                        tracing::info!(
                            "Domain no longer allowed for stale image {url}; serving stale blob."
                        );
                        let _ = self.repo.touch(hash).await;
                        return Ok((blob, mime_type));
                    }

                    // Try to re-fetch, fall back to stale on failure
                    match self.http_get(url).await {
                        Ok((fresh_bytes, fresh_mime)) => {
                            // Update cache
                            let size = fresh_bytes.len() as u64;
                            if size < self.max_bytes {
                                let _ = self.repo.update(hash, &fresh_bytes, &fresh_mime, size).await;
                                self.enforce_size_limit().await;
                            }
                            // Touch regardless
                            let _ = self.repo.touch(hash).await;
                            Ok((fresh_bytes, fresh_mime))
                        }
                        Err(e) => {
                            // Stale fallback: return old blob, touch it
                            tracing::warn!(
                                "Failed to re-fetch stale image {url}: {e}. Returning stale blob."
                            );
                            let _ = self.repo.touch(hash).await;
                            Ok((blob, mime_type))
                        }
                    }
                } else {
                    // Cache entry is fresh — touch and return
                    let _ = self.repo.touch(hash).await;
                    Ok((blob, mime_type))
                }
            }

            // ── Cache MISS ──────────────────────────────────────────────
            None => {
                // Check failed_urls to avoid re-downloading known 404s
                {
                    let failed = self.failed_urls.read().await;
                    if failed.contains(url) {
                        return Err(ImageCacheError::Placeholder);
                    }
                }

                // Check domain allowlist before HTTP fetch (defense-in-depth)
                let allowed = get_allowed_domains_from_repo(&*self.settings_repo).await;
                if !is_domain_in_list(url, &allowed) {
                    return Err(ImageCacheError::Placeholder);
                }

                // HTTP fetch
                match self.http_get(url).await {
                    Ok((bytes, mime_type)) => {
                        let size = bytes.len() as u64;

                        // Skip cache for oversized single entries
                        if size > self.max_bytes {
                            tracing::warn!(
                                "Image {url} is {size} bytes, exceeds cache limit {}. Serving in-memory only.",
                                self.max_bytes
                            );
                            return Ok((bytes, mime_type));
                        }

                        // Store in cache
                        let ttl = self.default_ttl.as_secs();
                        self.repo
                            .insert(hash, &bytes, &mime_type, size, ttl)
                            .await
                            .map_err(|e| {
                                ImageCacheError::DownloadFailed(format!("DB insert error: {e}"))
                            })?;

                        // Enforce LRU eviction
                        self.enforce_size_limit().await;

                        Ok((bytes, mime_type))
                    }
                    Err(e) => {
                        // Mark as failed URL to avoid retries
                        self.failed_urls.write().await.insert(url.to_string());
                        Err(e)
                    }
                }
            }
        }
    }

    /// HTTP GET and validate the response.
    async fn http_get(&self, url: &str) -> Result<(Vec<u8>, String), ImageCacheError> {
        let response = self
            .http
            .get(url)
            .send()
            .await
            .map_err(|e| ImageCacheError::DownloadFailed(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            return Err(ImageCacheError::DownloadFailed(format!(
                "HTTP {status} for {url}"
            )));
        }

        // Check Content-Length to reject oversized files early
        if let Some(content_length) = response.content_length() {
            if content_length > MAX_DOWNLOAD_SIZE {
                return Err(ImageCacheError::Oversized {
                    size: content_length,
                    limit: MAX_DOWNLOAD_SIZE,
                });
            }
        }

        // Extract Content-Type BEFORE consuming the response body
        let content_type = response
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .map(|s| s.split(';').next().unwrap_or(s).trim().to_lowercase());

        let bytes = response
            .bytes()
            .await
            .map_err(|e| ImageCacheError::DownloadFailed(e.to_string()))?
            .to_vec();

        if bytes.len() as u64 > MAX_DOWNLOAD_SIZE {
            return Err(ImageCacheError::Oversized {
                size: bytes.len() as u64,
                limit: MAX_DOWNLOAD_SIZE,
            });
        }

        // Strict MIME allowlist — only known image types
        let mime_type = content_type.ok_or_else(|| {
            ImageCacheError::DownloadFailed("Missing Content-Type header".to_string())
        })?;

        match mime_type.as_str() {
            "image/jpeg" | "image/png" | "image/webp" | "image/avif" | "image/gif" => {}
            _ => {
                return Err(ImageCacheError::DownloadFailed(format!(
                    "Rejected Content-Type '{mime_type}': not in image allowlist"
                )));
            }
        }

        Ok((bytes, mime_type))
    }

    /// Enforce the cache size limit by evicting LRU entries.
    async fn enforce_size_limit(&self) {
        if let Err(e) = self.repo.evict_lru(self.max_bytes).await {
            tracing::error!("Failed to evict LRU cache entries: {e}");
        }
    }

    /// Read the TTL for a specific cache entry.
    async fn get_ttl(&self, hash: &str) -> Option<i64> {
        sqlx::query_scalar::<_, i64>(
            "SELECT ttl_seconds FROM image_cache WHERE url_hash = ?1",
        )
        .bind(hash)
        .fetch_optional(&self.pool)
        .await
        .ok()
        .flatten()
    }

    /// Evict all expired entries (those past their TTL). For manual/background use.
    pub async fn evict_expired(&self) -> usize {
        self.repo.evict_expired().await.unwrap_or(0)
    }

    /// Returns the current total cache size in bytes.
    pub async fn total_cache_size(&self) -> u64 {
        self.repo.total_size().await.unwrap_or(0)
    }
}

/// Domain check helper: read the allowed image domains from settings.
async fn get_allowed_domains_from_repo(repo: &dyn SettingsRepository) -> Vec<String> {
    let raw = repo.get("allowed_image_domains").await;
    match raw {
        Some(val) if !val.trim().is_empty() => {
            let parsed = parse_domain_list(&val);
            if parsed.is_empty() {
                vec!["reverb.com".to_string(), "mlstatic.com".to_string()]
            } else {
                parsed
            }
        }
        _ => vec!["reverb.com".to_string(), "mlstatic.com".to_string()],
    }
}

/// Parse a comma-separated domain list, trimming whitespace.
fn parse_domain_list(input: &str) -> Vec<String> {
    input
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

/// Check if a URL's domain is in the allowed list.
/// IP literal hosts are allowed through (IPC layer handles those).
fn is_domain_in_list(url: &str, allowed: &[String]) -> bool {
    if let Ok(parsed) = url::Url::parse(url) {
        if let Some(host) = parsed.host() {
            // Allow IP literals (IPC layer handles those)
            if matches!(host, url::Host::Ipv4(_) | url::Host::Ipv6(_)) {
                return true;
            }
            let host_str = host.to_string();
            return allowed.iter().any(|domain| {
                host_str == domain.as_str() || host_str.ends_with(&format!(".{domain}"))
            });
        }
    }
    false
}

/// SHA-256 hex digest of a string.
fn hex_sha256(input: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    let result = hasher.finalize();
    result.iter().map(|b| format!("{:02x}", b)).collect()
}

/// Current Unix epoch seconds.
fn chrono_now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

// ── Tests ─────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repository::settings::MockSettingsRepository;
    use httpmock::prelude::*;
    use std::sync::Arc;

    /// Helper: create a MockSettingsRepository with default image domains.
    fn mock_settings() -> Arc<dyn SettingsRepository> {
        Arc::new(MockSettingsRepository::default())
    }

    /// Helper: create an in-memory SqlitePool with the image_cache table.
    async fn test_pool() -> SqlitePool {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
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
        .execute(&pool)
        .await
        .unwrap();
        sqlx::query(
            "CREATE INDEX IF NOT EXISTS idx_image_cache_last_accessed ON image_cache(last_accessed)",
        )
        .execute(&pool)
        .await
        .unwrap();
        pool
    }

    // ── Cache hit returns bytes and correct mime_type ───────────────────

    #[tokio::test]
    async fn cache_hit_returns_stored_blob() {
        let pool = test_pool().await;
        let svc = ImageCacheService::new_default(pool, mock_settings());

        // Pre-populate cache
        let repo = ImageCacheRepo::new(svc.pool.clone());
        let hash = hex_sha256("https://example.com/img.jpg");
        repo.insert(&hash, b"cached-image-data", "image/webp", 16, 3600)
            .await
            .unwrap();

        let (bytes, mime) = svc.get("https://example.com/img.jpg").await.unwrap();
        assert_eq!(bytes, b"cached-image-data");
        assert_eq!(mime, "image/webp");
    }

    // ── Cache miss fetches and stores ───────────────────────────────────

    #[tokio::test]
    async fn cache_miss_fetches_and_stores() {
        let server = MockServer::start();
        let mock = server.mock(|when, then| {
            when.method(GET).path("/test.png");
            then.status(200)
                .header("Content-Type", "image/png")
                .body("fake-png-bytes");
        });

        let pool = test_pool().await;
        let svc = ImageCacheService::new(pool, 10 * 1024 * 1024, Duration::from_secs(3600), mock_settings());

        let url = format!("{}/test.png", server.base_url());
        let (bytes, mime) = svc.get(&url).await.unwrap();

        assert_eq!(bytes, b"fake-png-bytes");
        assert_eq!(mime, "image/png");
        mock.assert_calls(1);

        // Second call should hit cache (no HTTP call)
        let (bytes2, _) = svc.get(&url).await.unwrap();
        assert_eq!(bytes2, b"fake-png-bytes");
        mock.assert_calls(1); // Still 1 — cached
    }

    // ── LRU evicts oldest entries ───────────────────────────────────────

    #[tokio::test]
    async fn lru_evicts_oldest_entries() {
        let pool = test_pool().await;
        // Set a small max_bytes so we can test eviction
        let svc = ImageCacheService::new(pool, 30, Duration::from_secs(3600), mock_settings());

        let hash_a = hex_sha256("https://example.com/a.jpg");
        let hash_b = hex_sha256("https://example.com/b.jpg");
        let hash_c = hex_sha256("https://example.com/c.jpg");

        // Manually insert 3 entries with carefully set timestamps
        let repo = ImageCacheRepo::new(svc.pool.clone());
        let now = chrono_now();

        // Insert oldest first (a), then b, then c
        sqlx::query(
            "INSERT INTO image_cache (url_hash, blob, mime_type, size_bytes, last_accessed, created_at, ttl_seconds)
             VALUES (?1, ?2, ?3, ?4, ?5, ?5, ?6)",
        )
        .bind(&hash_a)
        .bind(vec![0u8; 15])
        .bind("image/jpeg")
        .bind(15i64)
        .bind(now - 200)
        .bind(3600i64)
        .execute(&repo.pool)
        .await
        .unwrap();

        sqlx::query(
            "INSERT INTO image_cache (url_hash, blob, mime_type, size_bytes, last_accessed, created_at, ttl_seconds)
             VALUES (?1, ?2, ?3, ?4, ?5, ?5, ?6)",
        )
        .bind(&hash_b)
        .bind(vec![0u8; 10])
        .bind("image/jpeg")
        .bind(10i64)
        .bind(now - 100)
        .bind(3600i64)
        .execute(&repo.pool)
        .await
        .unwrap();

        sqlx::query(
            "INSERT INTO image_cache (url_hash, blob, mime_type, size_bytes, last_accessed, created_at, ttl_seconds)
             VALUES (?1, ?2, ?3, ?4, ?5, ?5, ?6)",
        )
        .bind(&hash_c)
        .bind(vec![0u8; 10])
        .bind("image/jpeg")
        .bind(10i64)
        .bind(now)
        .bind(3600i64)
        .execute(&repo.pool)
        .await
        .unwrap();

        // Total = 35, limit = 30. Enforce eviction.
        svc.enforce_size_limit().await;

        let total = repo.total_size().await.unwrap();
        assert!(total <= 30, "total cache size {} should be <= 30", total);

        // The oldest entry (a, 15 bytes) should be gone
        let fetched_a = repo.fetch(&hash_a).await.unwrap();
        assert!(fetched_a.is_none(), "oldest entry 'a' should be evicted");
    }

    // ── Concurrent coalesce = 1 HTTP call ────────────────────────────────

    #[tokio::test]
    async fn concurrent_requests_coalesce_into_one_http_call() {
        let server = MockServer::start();
        let mock = server.mock(|when, then| {
            when.method(GET).path("/shared.jpg");
            then.status(200)
                .header("Content-Type", "image/jpeg")
                .body("shared-image-data")
                .delay(std::time::Duration::from_millis(200)); // Ensure overlap
        });

        let pool = test_pool().await;
        let svc = Arc::new(ImageCacheService::new(
            pool,
            10 * 1024 * 1024,
            Duration::from_secs(3600),
            mock_settings(),
        ));

        let url = format!("{}/shared.jpg", server.base_url());

        // Spawn 10 concurrent requests
        let mut handles = Vec::new();
        for _ in 0..10 {
            let svc = svc.clone();
            let url = url.clone();
            handles.push(tokio::spawn(async move { svc.get(&url).await }));
        }

        // Await all
        for handle in handles {
            let result = handle.await.unwrap();
            assert!(result.is_ok());
            let (bytes, _) = result.unwrap();
            assert_eq!(bytes, b"shared-image-data");
        }

        // Only 1 HTTP call should have been made
        mock.assert_calls(1);
    }

    // ── Stale + offline returns stale blob ───────────────────────────────

    #[tokio::test]
    async fn stale_offline_returns_stale_blob() {
        let pool = test_pool().await;
        // TTL = 1 second, so the entry will be stale almost immediately
        let svc = ImageCacheService::new(pool.clone(), 10 * 1024 * 1024, Duration::from_secs(1), mock_settings());

        let hash = hex_sha256("https://example.com/stale.jpg");

        // Insert an entry with last_accessed far in the past (so TTL is expired)
        let far_past = chrono_now() - 100;
        sqlx::query(
            "INSERT INTO image_cache (url_hash, blob, mime_type, size_bytes, last_accessed, created_at, ttl_seconds)
             VALUES (?1, ?2, ?3, ?4, ?5, ?5, ?6)",
        )
        .bind(&hash)
        .bind(&b"stale-data"[..])
        .bind("image/jpeg")
        .bind(10i64)
        .bind(far_past)
        .bind(1i64) // TTL = 1 second
        .execute(&pool)
        .await
        .unwrap();

        // Now request the URL — the server is NOT running, so the re-fetch will fail
        // and we should get the stale blob back
        let expected_stale: &[u8] = b"stale-data";
        let (bytes, mime) = svc.get("https://example.com/stale.jpg").await.unwrap();
        assert_eq!(bytes, expected_stale, "should return stale blob on re-fetch failure");
        assert_eq!(mime, "image/jpeg");
    }

    // ── Oversized entry (>max_bytes) skips cache ─────────────────────────

    #[tokio::test]
    async fn oversized_entry_skips_cache() {
        let server = MockServer::start();
        let mock = server.mock(|when, then| {
            when.method(GET).path("/huge.jpg");
            then.status(200)
                .header("Content-Type", "image/jpeg")
                .body(vec![0u8; 100]); // 100 bytes
        });

        let pool = test_pool().await;
        // Set max_bytes to 50 — the 100-byte image exceeds this
        let svc = ImageCacheService::new(pool.clone(), 50, Duration::from_secs(3600), mock_settings());

        let url = format!("{}/huge.jpg", server.base_url());
        let (bytes, mime) = svc.get(&url).await.unwrap();
        assert_eq!(mime, "image/jpeg");
        assert_eq!(bytes.len(), 100);

        // The entry should NOT be in cache
        let hash = hex_sha256(&url);
        let repo = ImageCacheRepo::new(pool);
        let cached = repo.fetch(&hash).await.unwrap();
        assert!(cached.is_none(), "oversized entry should not be cached");

        mock.assert_calls(1);
    }

    // ── Stale + online re-fetches ────────────────────────────────────────

    #[tokio::test]
    async fn stale_online_refetches() {
        let server = MockServer::start();
        let mock = server.mock(|when, then| {
            when.method(GET).path("/refresh.jpg");
            then.status(200)
                .header("Content-Type", "image/jpeg")
                .body("fresh-data");
        });

        let pool = test_pool().await;
        let svc = ImageCacheService::new(pool.clone(), 10 * 1024 * 1024, Duration::from_secs(1), mock_settings());

        // Insert stale entry
        let hash = hex_sha256(&format!("{}/refresh.jpg", server.base_url()));
        let far_past = chrono_now() - 100;
        sqlx::query(
            "INSERT INTO image_cache (url_hash, blob, mime_type, size_bytes, last_accessed, created_at, ttl_seconds)
             VALUES (?1, ?2, ?3, ?4, ?5, ?5, ?6)",
        )
        .bind(&hash)
        .bind(&b"stale-data"[..])
        .bind("image/jpeg")
        .bind(10i64)
        .bind(far_past)
        .bind(1i64)
        .execute(&pool)
        .await
        .unwrap();

        let url = format!("{}/refresh.jpg", server.base_url());
        let (bytes, _) = svc.get(&url).await.unwrap();
        assert_eq!(bytes, b"fresh-data", "should re-fetch stale entry when online");

        mock.assert_calls(1);
    }

    // ── URL validation ──────────────────────────────────────────────────

    #[tokio::test]
    async fn invalid_url_returns_error() {
        let pool = test_pool().await;
        let svc = ImageCacheService::new_default(pool, mock_settings());

        let result = svc.get("not-a-url").await;
        assert!(result.is_err());
        assert!(
            matches!(&result, Err(ImageCacheError::InvalidUrl(_))),
            "expected InvalidUrl error"
        );
    }

    // ── Placeholder on network failure (uncached) ────────────────────────

    #[tokio::test]
    async fn network_failure_returns_placeholder() {
        let pool = test_pool().await;
        let svc = ImageCacheService::new_default(pool, mock_settings());

        // URL that doesn't resolve
        let result = svc
            .get("https://nonexistent-domain-abc12345.com/img.jpg")
            .await;

        assert!(result.is_err());
    }

    // ── Domain rejection via SettingsRepository ──────────────────────────

    #[tokio::test]
    async fn blocked_domain_returns_placeholder() {
        let restricted = MockSettingsRepository::default();
        restricted.save("allowed_image_domains", "reverb.com").await.unwrap();

        let pool = test_pool().await;
        let svc = ImageCacheService::new_default(
            pool,
            Arc::new(restricted) as Arc<dyn SettingsRepository>,
        );

        let result = svc.get("https://evil.com/payload.jpg").await;
        assert!(
            matches!(result, Err(ImageCacheError::Placeholder)),
            "blocked domain should return Placeholder, got {:?}",
            result,
        );
    }

    #[tokio::test]
    async fn allowed_domain_passes_check() {
        let permissive = MockSettingsRepository::default();
        permissive
            .save("allowed_image_domains", "reverb.com,mlstatic.com")
            .await
            .unwrap();

        let pool = test_pool().await;
        let svc = ImageCacheService::new_default(
            pool,
            Arc::new(permissive) as Arc<dyn SettingsRepository>,
        );

        // This should pass domain check but fail with DownloadFailed (no server running)
        let result = svc.get("https://reverb.com/pedal.jpg").await;
        assert!(
            !matches!(result, Err(ImageCacheError::Placeholder)),
            "allowed domain should not be Placeholder, got {:?}",
            result,
        );
    }
}
