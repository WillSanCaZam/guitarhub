# ADR-005: User-Connected Store Accounts

## Status

Accepted

## Context

Today GuitarHub central-scrapes product catalogs from marketplace/retailer websites (Reverb, Guitar Center). All users see the same catalog. The user wants a model where:

1. Users **discover stores** inside the app
2. Users **connect their own accounts** to supported stores (e.g., Reverb OAuth)
3. The store's catalog becomes available in the app
4. GuitarHub **classifies and normalizes** products from all connected sources

For marketplaces like Reverb, this unlocks user-specific data: personal listings, purchase history, watchlist items, shop inventory. For retailers (GC, Sweetwater, Thomann) that don't expose user APIs, public scraping remains the only option.

### Store API Landscape

| Store | User API | Public Catalog API | Auth Model |
|-------|----------|-------------------|------------|
| Reverb | ✅ Listings, orders, shop | ✅ Browse/search | Personal Access Token (OAuth scopes: `public`, `read_listings`, `read_orders`) |
| Guitar Center | ❌ | ❌ No public API | N/A (scrape only) |
| Sweetwater | ❌ | ❌ No public API | N/A (scrape only) |
| Thomann | ❌ | ❌ No public API | N/A (scrape only) |

### Tauri Auth Feasibility

Store APIs use two auth models:

1. **Personal Access Tokens** (Reverb): User generates a token from the store's settings page and pastes it into the app. No redirect flow.
2. **OAuth 2.0 Authorization Code** (eBay, Etsy, future stores): Desktop apps handle this via:
   - **Localhost callback server**: App starts a tiny HTTP server on loopback, opens browser for auth, provider redirects to `http://localhost:{port}/callback`
   - **Custom URL scheme**: Register `guitarhub://` protocol handler, provider redirects there
   - **PKCE**: Recommended for public clients — no client secret needed

For **MVP**, Reverb's Personal Access Token paste flow is the target. OAuth redirect will be implemented when a store that requires it is added.

## Decision

Adopt a **Dual Pipeline Architecture** with user-connected store accounts.

### Architecture

```
┌─────────────────────────────────────────────────────────┐
│                    GuitarHub App                         │
│                                                          │
│  ┌──────────────┐    ┌─────────────────────────────┐    │
│  │ Store         │    │  Connection Manager          │    │
│  │ Registry      │    │  - OAuth flow per store      │    │
│  │ - Reverb      │───→│  - Token storage (OS keyring)│    │
│  │ - GC          │    │  - Refresh / revoke          │    │
│  │ - Sweetwater  │    └──────────┬──────────────────┘    │
│  └──────────────┘               │                        │
│                                  │                        │
│  ┌───────────────────────────────┴──────────────────┐    │
│  │              Sync Pipeline                        │    │
│  │                                                   │    │
│  │  Public Scrape (no auth)  User API (with token)  │    │
│  │  ┌──────────────────┐    ┌───────────────────┐   │    │
│  │  │ Reverb (listings)│    │ Reverb (user data) │   │    │
│  │  │ GC (scrape)      │    │ (future: eBay,     │   │    │
│  │  │ Sweetwater       │    │  Etsy, etc.)       │   │    │
│  │  └──────────────────┘    └───────────────────┘   │    │
│  │                                                   │    │
│  │  ┌────────────────────────────────────────────┐   │    │
│  │  │      Normalizer (condition, category,       │   │    │
│  │  │       price, availability)                  │   │    │
│  │  └────────────────────────────────────────────┘   │    │
│  └───────────────────────────────────────────────────┘    │
│                                                          │
│  ┌──────────────────────────────────────────────────┐    │
│  │              SQLite Catalog                       │    │
│  │  products_meta (source_id → user_id nullable)     │    │
│  └──────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────┘
```

### Key Components

**1. Store Registry**
- In-app list of supported stores with metadata (name, icon, auth type, website)
- Hardcoded initially, could become dynamic later
- Stored in `settings` or a new `stores` table

**2. Connection Manager**
- For stores with Personal Access Tokens (Reverb): 
  - Open browser to store's token generation page
  - Inline step-by-step guide for generating the token
  - Token input field with real-time validation (test against the store's API)
  - Token stored encrypted via `tauri-plugin-store` (AES-256-GCM, key derived from OS machine key)
- For stores with OAuth (future: eBay, Etsy): full redirect flow
- For non-auth stores (GC, Sweetwater): no connection needed, public scrape only
- Track per-install which stores are connected (stored in `settings` table)

**3. Dual Sync Pipeline**
- **Public scrape**: unchanged — Python scraper fetches marketplaces, produces JSON, Tauri syncs to DB (as today)
- **Authenticated API**: Rust calls Reverb API with user's token, fetches user-specific data (listings, orders), normalizes, inserts into same `products_meta` with `user_id` set

**4. Normalizer**
- Existing `normalize_condition()` in sanitize handles all sources
- Category mapping per store → common taxonomy
- No change needed — our system already normalizes

**5. Data Model Change**
- `products_meta`: add nullable `user_id` column (TEXT FK to users table)
- Products from public scrape: `user_id = NULL`
- Products from user-connected stores: `user_id = <user_uid>`
- This keeps the unified catalog but allows filtering by user

### Reverb Connection Flow (Personal Access Token)

Reverb does not support OAuth redirect for third-party apps. Users connect via Personal Access Token:

1. User clicks "Connect Reverb" in the app
2. App opens browser to `https://reverb.com/settings/api` (direct link to token generation page)
3. App shows inline guide with steps: name the token, select scopes (`public`, `read_listings`), copy
4. User pastes the token into GuitarHub's input field
5. App validates the token immediately: calls `GET /api/my/account` with it
6. On success → "✅ Connected as @username". Token stored encrypted in OS keyring
7. On failure → clear error message: "Token invalid. Make sure you copied the full token."
8. App fetches user's listings via `GET /api/my/listings` with `Authorization: Bearer <token>`

**Key UX decisions for average users:**
- One-click navigation to the token page (opens in default browser)
- Inline instructions with visual cues, not a separate help page
- Immediate validation with friendly feedback
- Status indicator: connected/disconnected per store

### MVP Scope

**Phase 1 (now):** Store Registry UI + Connection Manager + Reverb token flow
- User sees "Available Stores" in the app
- User connects Reverb account (paste token or OAuth)
- User's Reverb listings appear in the catalog
- Public scraping continues unchanged

**Phase 2:** User-specific data sync (watchlist, purchases)
**Phase 3:** Additional OAuth stores (eBay, Etsy, etc.)
**Phase 4:** Store discovery recommendations

## Consequences

### Positive
- Users see **their own gear** from marketplaces in one place
- Enables the "Mihon of guitars" vision — users bring their own sources
- Public scraping and user-API can coexist without conflict
- Normalization layer stays unchanged — one taxonomy for all sources
- OS keyring token storage is more secure than config files

### Negative
- Adds user authentication to the app (GuitarHub user accounts)
- OAuth flow on desktop is more complex than web (localhost server, URL scheme handling)
- Token management adds surface area for bugs and UX friction
- Reverb Personal Access Tokens don't expire — revoke-only
- Not all stores have user APIs — GC/Sweetwater remain scrape-only
- `products_meta` schema migration needed for `user_id`

### Risks
- **Token security**: Store encrypted, never log, never send over network
- **OAuth callback server**: Port conflicts, firewall blocking, anti-virus interference
- **API rate limits**: Reverb API rate limits per token — multiple users = distributed load
- **User accounts**: GuitarHub needs user auth (ADR needed) or anonymous device IDs

## Alternatives Considered

### 1. Proxy Scraper Service (rejected)

Run a central server that handles all OAuth flows and fetches data on behalf of users.

- **Pros**: No OAuth in desktop app, single token management
- **Cons**: $$ Server costs, contradicts offline-first / zero-server-cost constraint, introduces a central point of failure, privacy concerns (server sees all user tokens)

### 2. Iframe/Mashery (rejected)

Embed each store's website in an iframe or webview and let users interact natively.

- **Pros**: Zero API integration needed
- **Cons**: UX is terrible (unified catalog impossible), stores block iframes, no product normalization, security concerns with webview auth

### 3. Self-Hosted Scraper (rejected)

Users run their own scraper instance (Docker) that fetches all catalogs.

- **Pros**: Fully offline, no server costs
- **Cons**: Requires Docker + technical skills, terrible UX for average users, no mobile, too complex for the target audience

## Migration

1. Migration 012: Add `users` table + `user_id` column to `products_meta`
2. Add Store Registry (hardcoded JSON in Rust or settings table)
3. Add Connection Manager UI + Reverb OAuth flow
4. Add user-authenticated sync for Reverb listings
5. Existing scrapers: set `user_id = NULL` (unchanged behavior)
