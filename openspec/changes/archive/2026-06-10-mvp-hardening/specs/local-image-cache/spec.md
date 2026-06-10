# Delta for local-image-cache

## MODIFIED Requirements

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
| **HTTP URL rejected** | URL is `http://img.reverb.com/pedal.jpg` | Request | REJECTED with `ImageCacheError::InvalidUrl` |
| **Non-HTTPS scheme rejected** | URL is `file:///tmp/image.jpg` | Request | REJECTED with `ImageCacheError::InvalidUrl` |
| **IP literal URL rejected** | URL is `http://192.168.1.1/image.jpg` | Request | REJECTED with `ImageCacheError::InvalidUrl` |
