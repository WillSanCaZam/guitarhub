# Delta for local-image-cache

## MODIFIED Requirements

### Requirement: Cache images on first display

The system MUST download the image from `image_url` on first request, store it in `image_cache` (SQLite blobs), and serve subsequent requests from cache.

**MIME validation** (added v6): The system MUST validate `Content-Type` against `["image/jpeg", "image/png", "image/webp", "image/avif", "image/gif"]`. Non-matching responses MUST be rejected before cache write. The byte-sniffing fallback MUST NOT be used.

**Domain validation** (added v7): The system MUST validate the image URL domain against the configured allowlist before initiating any HTTP request. The allowlist MUST be read from `get_setting("allowed_image_domains")`. If the setting is empty, unparseable, or missing, the system MUST fall back to `["reverb.com", "mlstatic.com"]`. Domains not in the allowlist MUST be rejected before download.
(Previously: no domain validation at the cache layer; now domain allowlist is configurable via settings)

#### Updated Scenarios

| Case | Precondition | Action | Outcome |
|------|-------------|--------|---------|
| First load, domain allowed | Image not cached, domain in settings | Request `https://img.reverb.com/pedal.jpg` | Download, store blob, display |
| Domain rejected | Domain not in settings or fallback | Request `https://evil.com/payload.jpg` | REJECTED before HTTP call |
| Setting empty uses fallback | `allowed_image_domains` empty | Request `https://img.mlstatic.com/pedal.jpg` | Passes (fallback allows mlstatic.com) |
| Offline hit | Image cached, device offline | Request same URL | Return cached blob, no network call |
| MIME rejected | URL returns `Content-Type: image/svg+xml` | Request, validate | REJECTED, not cached |
| MIME missing | No `Content-Type` header | Request, validate | REJECTED, not cached |

## ADDED Requirements

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
