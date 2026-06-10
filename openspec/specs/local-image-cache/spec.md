# Local Image Cache Specification

> **v3** (2026-06-10) — Added URL scheme validation from `mvp-hardening`. Only `https://` URLs accepted at the service layer.
> Content-Type must match `["image/jpeg", "image/png", "image/webp", "image/avif", "image/gif"]`.
> Byte-sniffing fallback removed.

## Purpose

Offline-first image caching that lets the app display product images without an active internet connection. Images are fetched on demand, stored in SQLite blobs, and evicted by LRU policy when the cache exceeds its size limit.

## Requirements

### Requirement: Cache images on first display

The system MUST download the image from `image_url` on first request, store it in `image_cache` (SQLite blobs), and serve subsequent requests from cache.

**MIME validation** (added v6): The system MUST validate `Content-Type` against `["image/jpeg", "image/png", "image/webp", "image/avif", "image/gif"]`. Non-matching responses MUST be rejected before cache write. The byte-sniffing fallback MUST NOT be used.

**Domain validation** (added v7): The system MUST validate the image URL domain against the configured allowlist before initiating any HTTP request. The allowlist MUST be read from `get_setting("allowed_image_domains")`. If the setting is empty, unparseable, or missing, the system MUST fall back to `["reverb.com", "mlstatic.com"]`. Domains not in the allowlist MUST be rejected before download.
(Previously: no domain validation at the cache layer; now domain allowlist is configurable via settings)

**URL scheme validation** (added v8): The system MUST reject any URL that does not start with `https://` at the service layer with `ImageCacheError::InvalidUrl`. The `tracing::warn!` path for `http://` URLs MUST be removed — non-HTTPS URLs MUST produce an error, not a warning. This validation MUST happen before any cache lookup or HTTP request, making the service-layer behavior consistent with the command-layer validation.
(Previously: service layer warned on `http://` with `tracing::warn!` but allowed it through to the HTTP client; only the command layer rejected non-HTTPS URLs)

#### Updated Scenarios

| Case | Precondition | Action | Outcome |
|------|-------------|--------|---------|
| First load, domain allowed | Image not cached, domain in settings | Request `https://img.reverb.com/pedal.jpg` | Download, store blob, display |
| Domain rejected | Domain not in settings or fallback | Request `https://evil.com/payload.jpg` | REJECTED before HTTP call |
| Setting empty uses fallback | `allowed_image_domains` empty | Request `https://img.mlstatic.com/pedal.jpg` | Passes (fallback allows mlstatic.com) |
| Offline hit | Image cached, device offline | Request same URL | Return cached blob, no network call |
| MIME rejected | URL returns `Content-Type: image/svg+xml` | Request, validate | REJECTED, not cached |
| MIME missing | No `Content-Type` header | Request, validate | REJECTED, not cached |
| HTTP URL rejected | URL is `http://img.reverb.com/pedal.jpg` | Request | REJECTED with `ImageCacheError::InvalidUrl` |
| Non-HTTPS scheme rejected | URL is `file:///tmp/image.jpg` | Request | REJECTED with `ImageCacheError::InvalidUrl` |
| IP literal URL rejected | URL is `http://192.168.1.1/image.jpg` | Request | REJECTED with `ImageCacheError::InvalidUrl` |

### Requirement: Deduplicate concurrent downloads

The system MUST coalesce simultaneous requests for the same `image_url` into a single HTTP call.

#### Scenario: Concurrent requests

- GIVEN 20 products share the same `image_url`
- WHEN the UI requests all at once
- THEN only ONE HTTP request is made
- AND all 20 resolve to the same blob

### Requirement: Enforce cache size via LRU eviction

The system MUST limit total cache size to a configured maximum (default: 50 MB). When a new image would exceed the limit, the system MUST evict least-recently-used entries until the new image fits.

#### Scenarios

| Case | Precondition | Action | Outcome |
|------|-------------|--------|---------|
| Cache full | 48 MB used, new image is 5 MB | Store | Evict oldest entries (≥3 MB), store new, total ≤50 MB |
| Oversized image | Limit 50 MB, image is 60 MB | Store | MAY skip caching, serve in-memory only |
| Single entry eviction | One 50 MB entry, new 1 MB image | Store | Evict 50 MB entry, store 1 MB |

### Requirement: Handle download failures gracefully

The system MUST NOT crash or block the UI on download failure. It MUST return a placeholder and log the failure.

#### Scenarios

| Case | Precondition | Action | Outcome |
|------|-------------|--------|---------|
| Network failure | Offline, image uncached | Request image | Placeholder returned, WARN logged |
| HTTP 404 | Image URL returns 404 | Request image | Placeholder returned, SHOULD mark URL as failed to avoid retries |

### Requirement: Settings changes take effect without restart

Changes to `allowed_image_domains` via the settings UI MUST be picked up by the cache layer on the next image request. A restart MUST NOT be required.

#### Scenario: Live domain update
- GIVEN `allowed_image_domains` = "reverb.com"
- AND a request to `https://newstore.com/pedal.jpg` is REJECTED
- WHEN the user adds "newstore.com" via settings
- THEN a subsequent request to the same URL succeeds

#### Scenario: Invalid setting falls back immediately
- GIVEN `allowed_image_domains` = "not,a,domain" (malformed)
- WHEN the next image request is validated
- THEN the system uses the fallback allowlist without crashing

### Requirement: TTL-based cache invalidation

The system SHOULD support configurable TTL. Entries older than TTL SHOULD be re-fetched on next access with graceful fallback to stale blob if re-fetch fails.

#### Scenarios

| Case | Precondition | Action | Outcome |
|------|-------------|--------|---------|
| Stale → refresh | 8 days old, TTL 7 days, online | Request | Fetch fresh copy, update cache |
| Stale → fallback | 8 days old, TTL 7 days, offline | Request | Return stale blob, log warning |

## Acceptance Criteria

| Criterion | How to verify |
|-----------|---------------|
| Cache hit on second load | Load offline after first online load — renders correctly |
| Size limit enforced | Fill cache with known sizes — verify `SUM(LENGTH(blob))` ≤ limit |
| LRU eviction works | Access 10 images, re-access first, check `last_accessed` order |
| No duplicate downloads | Same URL requested once during concurrent access |
| Network failure no crash | Kill network, request uncached — placeholder, no crash |
| MIME validation enforced | Request with SVG `Content-Type` — rejected, not stored in cache |
| No byte-sniffing fallback | Request with missing `Content-Type` — rejected, not stored | |
| Stale served when offline | Set TTL=1s, wait, go offline — stale blob returned |

## Out of Scope

- Image preprocessing (resize, format conversion)
- Video or animated image caching
- Shared cache between user profiles
- Filesystem-level image cache
- Progressive image loading
