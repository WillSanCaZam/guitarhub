# GuitarHub — Project Plan v7

> **Estado del documento:** Actualizado al 2026-06-04 para reflejar el código real.  
> Secciones marcadas con 🟢 = implementado, 🟡 = parcial, 🔴 = no implementado (aspiracional).

## What is GuitarHub?

GuitarHub is a **native cross-platform desktop app** that aggregates guitar, amp, pedal, and accessory listings from multiple online stores into a single unified catalog. It is the **Mihon of guitars**: free forever, open source, offline-first, zero server costs, community-extensible.

The user installs the app, syncs the catalog, and browses/searches without ads, registration, or trackers. When they find a product, they are redirected to the original store to buy. GuitarHub never holds inventory, never processes payments, never touches money.

**Reference projects:**
- [Mihon](https://mihon.app/) — manga aggregator, offline, extension-based, FOSS
- [Hydra Launcher](https://hydralauncher.gg/) — game library aggregator, FOSS

---

## Core Principles

- **Zero infrastructure cost** — scraper runs on GitHub Actions (free), catalog served from GitHub Pages (free), no backend server ever
- **Offline-first** — catalog downloaded and cached in SQLite on the user's device; full search/filter without internet
- **Extension-based sources** — each store is an independent plugin; the community can add/maintain stores without touching core
- **Free forever** — no ads, no tracking, no registration, no paywalls, no Pro version
- **FOSS-compatible** — GPL-3.0 app, MIT scraper; F-Droid, Flathub, AUR compatible
- **Windows-first distribution** — Windows is the dominant OS for guitarists; Linux, macOS, and Android follow
- **Explicit contracts** — every integration boundary (extension port, catalog schema, IPC command) is formally defined and versioned
- **Security by design** — community contributions run sandboxed; catalog content validated before insertion; URLs sanitized at ingestion
- **Observable by default** — scraper health, run history, and source metrics are publicly visible at zero cost
- **Governed from day one** — contribution rules, decision process, and code of conduct are defined before the first external PR

---

## Architecture Overview

```
┌──────────────────────────────────────────────────────────────────────┐
│  GITHUB ACTIONS (cron: every 6h)           🔴 NO IMPLEMENTADO        │
│                                                                      │
│  scrape.yml (matrix, fail-fast: false)                               │
│  ├── [no secrets] scrape job per source                             │
│  │     use_cases/ScrapeSourceUseCase                                 │
│  │       └── SourcePort → adapters/sources/{source}.py              │
│  │     use_cases/ValidateCatalogUseCase                              │
│  │       └── HealthPort → adapters/health/github_issues.py          │
│  │     → writes catalog-{source}.json + metrics-{source}.json       │
│  │       (artifacts, no gh-pages access)                             │
│  │                                                                   │
│  └── [GITHUB_TOKEN] publish job (after all scrapes)                 │
│        use_cases/PublishCatalogUseCase                               │
│          └── PublisherPort → adapters/publishers/gh_pages.py        │
│        → pushes to gh-pages + updates index.json + status.json      │
└──────────────────────────────────────────────────────────────────────┘
                         │ HTTPS (gzip)
                         ▼
┌──────────────────────────────────────────────────────────────────────┐
│  GITHUB PAGES  (free static CDN)              🔴 NO IMPLEMENTADO     │
│  https://user.github.io/guitarhub/                                   │
│  ├── index.json              ← source registry (versioned)           │
│  ├── status.json             ← public health dashboard data          │
│  ├── latest.json             ← app update endpoint (tauri-updater)   │
│  ├── catalog-reverb.json                                             │
│  ├── catalog-reverb-delta.json  ← incremental updates               │
│  ├── catalog-mercadolibre.json                                       │
│  ├── metrics-reverb.json     ← historical quality metrics            │
│  └── catalog-{source}.json                                           │
└──────────────────────────────────────────────────────────────────────┘
                         │ HTTPS (Accept-Encoding: gzip)
                         ▼
┌──────────────────────────────────────────────────────────────────────┐
│  USER DEVICE — GuitarHub App (Tauri 2)       🟡 PARCIAL              │
│                                                                      │
│  ┌────────────────────────────────────────────────────────────────┐  │
│  │  UI Layer (Svelte 5)          Backend (Rust)                   │  │
│  │  🟡 5 components sueltos      🟢 commands/ (Tauri IPC)         │  │
│  │  🔴 Sin SvelteKit scaffold        ├── image_command.rs 🟢      │  │
│  │  🔴 Sin stores TS                 ├── price_command.rs 🟢      │  │
│  │  🔴 Sin i18n                      ├── settings_command.rs 🟢   │  │
│  │  🔴 Sin routes                    └── export_command.rs 🟢     │  │
│  │                                                              │  │
│  │                                       services/               │  │
│  │                                       ├── image_cache.rs 🟢   │  │
│  │                                       ├── alert_service.rs 🟢 │  │
│  │                                       └── export_service.rs 🟢│  │
│  │                                               │                │  │
│  │                                       repository/              │  │
│  │                                       ├── price_history.rs 🟢  │  │
│  │                                       ├── settings.rs (trait)  │  │
│  │                                       └── sqlite/ impls 🟢     │  │
│  └────────────────────────────────────────────────────────────────┘  │
│                                                                      │
│  SQLite 🟢 (image_cache, price_history, settings,                     │
│            products_meta, products_fts, sync_state, wishlist)         │
└──────────────────────────────────────────────────────────────────────┘
```

**Data flow (aspiracional — el scraper no está implementado aún):**
1. GitHub Action runs each source adapter in an isolated job (no `GITHUB_TOKEN` in scope)
2. `ScrapeSourceUseCase` → `SourcePort.fetch_products()` → raw products
3. `ValidateCatalogUseCase` checks quality thresholds; on failure → `HealthPort.report()` creates GitHub Issue and skips publish for that source
4. Each scrape job writes `catalog-{source}.json` + `metrics-{source}.json` as workflow artifacts
5. A separate `publish` job (with `GITHUB_TOKEN`) collects artifacts and pushes to `gh-pages`
6. `index.json`, `status.json`, and `latest.json` are regenerated on every publish
7. App downloads only enabled sources; requests include `Accept-Encoding: gzip`
8. `SyncService` follows the **Sync State Machine** — previous SQLite state never destroyed on failure
9. All URLs sanitized to `https://` before insertion; invalid URLs silently dropped with a warning log
10. All search/filter runs locally — no network needed after sync

---

## Repository Structure

```
guitarhub/
├── scraper/                          🔴 NO IMPLEMENTADO — directorio completo por crear
│
├── src-tauri/                        🟢 Backend Rust (Tauri 2)
│   ├── src/
│   │   ├── main.rs                   🟢 Tauri builder con plugins, commands, state
│   │   ├── lib.rs                    🟢 initialize_database(), AppState, MigrationRunner
│   │   ├── commands/                 🟢 Tauri IPC glue
│   │   │   ├── mod.rs
│   │   │   ├── image_command.rs      🟢 get_product_image con domain allowlist + base64
│   │   │   ├── price_command.rs      🟢 get_price_history, get_price_insight
│   │   │   ├── settings_command.rs   🟢 get/save setting, test_alert_channel
│   │   │   └── export_command.rs     🟢 export_data a ZIP
│   │   ├── services/                 🟢 Lógica de negocio
│   │   │   ├── mod.rs
│   │   │   ├── image_cache.rs        🟢 LRU/TTL, DashMap coalescing, SQLite BLOBs
│   │   │   ├── alert_service.rs      🟢 AlertDispatcher: App/Ntfy/Webhook, SSRF-safe
│   │   │   └── export_service.rs     🟢 Export ZIP con wishlist + price_history
│   │   └── repository/               🟢 Acceso a datos
│   │       ├── mod.rs
│   │       ├── price_history.rs      🟢 PriceHistoryRepo con outlier filtering
│   │       ├── settings.rs           🟢 SettingsRepository trait + Mock
│   │       └── sqlite/
│   │           ├── mod.rs
│   │           ├── image_cache.rs    🟢 ImageCacheRepo (sqlx directo)
│   │           ├── settings.rs       🟢 SqliteSettingsRepository
│   │           └── migrations/       🟢 5 migraciones SQL + MigrationRunner (755 loc)
│   │               ├── mod.rs        🟢 MigrationRunner con gap detection
│   │               ├── 001_init.sql
│   │               ├── 002_add_url_validation.sql
│   │               ├── 003_add_image_cache.sql
│   │               ├── 004_add_price_source.sql
│   │               └── 005_add_settings.sql
│   ├── capabilities/
│   │   └── default.json              🟢 core + dialog + notification permissions
│   ├── Cargo.toml                    🟢 tauri 2, sqlx, reqwest, serde, zip, etc.
│   ├── tauri.conf.json               🟢 CSP, window config, build commands
│   ├── .cargo-audit.toml             🟢 threshold = "high"
│   ├── build.rs                      🟢 tauri_build::build()
│   ├── gen/                          🟢 Auto-generado por Tauri
│   └── icons/                        🟢 Iconos de la app
│
├── src/                              🟡 Frontend Svelte 5 (incompleto)
│   └── lib/
│       └── components/               🟡 5 componentes, sin SvelteKit scaffold
│           ├── ProductCard.svelte
│           ├── ProductDetail.svelte
│           ├── PriceBadge.svelte
│           ├── PriceChart.svelte
│           └── Settings.svelte
│   🔴 Faltan: package.json, svelte.config.js, vite.config.ts, tsconfig.json
│   🔴 Faltan: routes/, stores/, i18n/, app.html
│
├── openspec/                         🟢 SDD artifacts
│   ├── config.yaml                   🟢 strict_tdd: true
│   ├── specs/                        🟢 Especificaciones activas
│   └── changes/archive/              🟢 Cambios archivados
│       ├── price-intelligence-phase-3/
│       ├── 2026-06-03-mvp-foundation/
│       ├── 2026-06-03-plan-v3-revision/
│       └── 2026-06-03-fix-v4-findings/
│
├── scripts/
│   └── packaging/                    🟡 Packaging metadata
│       ├── com.guitarhub.app.desktop
│       ├── com.guitarhub.app.metainfo.xml
│       ├── fdroid-reproducible-build.md
│       └── icons/
│
├── docs/
│   └── CONTRIBUTING.md               🟡 Existe, referencia scraper/ que no existe
│   🔴 Faltan: ARCHITECTURE.md, GOVERNANCE.md, SCHEMA.md,
│   🔴         SYNC_STATE_MACHINE.md, TESTING.md, DELTA_SYNC.md
│
├── .github/
│   ├── workflows/
│   │   ├── ci.yml                    ⚠️ ROTO — referencia scraper/ que no existe
│   │   ├── scrape.yml                ⚠️ ROTO — referencia scraper/ que no existe
│   │   ├── release.yml               🟢 Builds Rust, necesita ajustes
│   │   └── e2e.yml                   ⚠️ ROTO — referencia scraper/ que no existe
│   └── dependabot.yml                ⚠️ ROTO — referencia scraper/ para pip
│   🔴 Faltan: ISSUE_TEMPLATE/, PULL_REQUEST_TEMPLATE.md
│
├── .devcontainer/                    🟢 Dev container config
├── .gitignore                        🟢 target/, .env, __pycache__/, node_modules/
├── .pre-commit-config.yaml           🟢 ruff, mypy, cargo fmt/clippy, gitleaks
├── .env.example                      🟢 Variables de entorno documentadas
├── .atl/                             🟢 Skill registry (Gentle AI)
├── Makefile                          🟡 Targets de scraper con fallback
├── requirements-dev.txt              🟢 Dev deps (ruff, mypy, pytest, etc.)
├── rust-toolchain.toml               🟢 Rust stable + rustfmt + clippy
├── SECURITY.md                       🟢 Política de vulnerabilidades
│
├── 🔴 FALTAN: README.md, LICENSE, CHANGELOG.md, CODE_OF_CONDUCT.md
│
└── guitarhub-plan-v6.md              🟢 Este documento (v7 actualizado)
```

---

## Governance

Documented in full in (🔴 **`docs/GOVERNANCE.md`** — no implementado aún).

### Decision types

| Decision | Who decides | Process |
|---|---|---|
| Bug fix | Any maintainer | PR + 1 approval |
| New source adapter | Any maintainer | PR + 1 approval + contract tests pass |
| Schema minor bump (1.x) | Any maintainer | PR + 1 approval + SCHEMA.md update |
| Schema major bump (2.0) | All maintainers | Issue discussion + 2 weeks comment period |
| Core architecture change | All maintainers | RFC in `docs/` + 2 weeks comment period |
| New maintainer | All maintainers | Nomination + consensus |

### Merge rights

- `main` branch: protected. Requires 1 approving review + all CI checks green.
- `gh-pages` branch: only the `publish` CI job can push (via `GITHUB_TOKEN`). No human push.
- Community source adapters: reviewed by at least one maintainer before merging to main and before being added to the scrape cron.

### Community source adapter security policy

Community-contributed adapters run in a **sandboxed CI job** with:
- No access to `GITHUB_TOKEN` or any repository secrets
- No write access to `gh-pages`
- Network access limited to the adapter's declared `BASE_URL`
- Output written only as workflow artifacts

The `publish` job (which has `GITHUB_TOKEN`) only picks up artifacts from adapters that passed contract tests and health validation. A malicious adapter cannot publish to `gh-pages` or exfiltrate secrets.

---

## Clean Architecture — Scraper (Python)

🔴 **NO IMPLEMENTADO.** Todo el directorio `scraper/` está por crear. Esta sección es aspiracional.

El scraper seguirá **Ports & Adapters (Hexagonal Architecture)**. Domain y use cases nunca importan de `adapters/`. La dirección de dependencia siempre apunta hacia adentro.

```
adapters/ ──► use_cases/ ──► ports/ (abstract)
                    └──► domain/
```

Ver `docs/CONTRIBUTING.md` cuando el scraper esté implementado para la guía completa de cómo agregar una fuente.

---

## Clean Architecture — App Backend (Rust) 🟢

Dependency direction: `commands → services → repository (trait) ← sqlite impl`.

```
commands/ ──► services/ ──► repository traits
                                  ▲
                             sqlite/ (impl)
```

### Domain (🟡 parcial)

No existe un módulo `domain/` separado. Los tipos de dominio están definidos inline donde se usan:

- `PricePoint`, `PriceInsightRow` → `repository/price_history.rs`
- `AlertChannel`, `AlertTestResult` → `services/alert_service.rs`
- `ExportResult`, `ExportError` → `services/export_service.rs`
- `SettingsRepository` trait → `repository/settings.rs`

Las estructuras de catálogo (`Product`, `Condition`, `Availability`) existen en el plan pero **no se han migrado al código** porque el SyncService no está implementado todavía.

### Repository traits (🟢)

```rust
// src-tauri/src/repository/settings.rs
#[async_trait]
pub trait SettingsRepository: Send + Sync {
    async fn get(&self, key: &str) -> Option<String>;
    async fn save(&self, key: &str, value: &str) -> Result<(), sqlx::Error>;
    async fn delete(&self, key: &str) -> Result<(), sqlx::Error>;
}
```

```rust
// src-tauri/src/repository/price_history.rs
// Concrete struct PriceHistoryRepo — queries price_history table with outlier filtering.
pub struct PriceHistoryRepo { pool: SqlitePool }

impl PriceHistoryRepo {
    pub async fn get_history(&self, sku: &str) -> Result<Vec<PricePoint>, sqlx::Error> { ... }
    pub async fn get_insight(&self, sku: &str) -> Result<PriceInsightRow, sqlx::Error> { ... }
}
```

### Services (🟢)

```rust
// src-tauri/src/services/alert_service.rs
pub enum AlertChannel { App, Ntfy { topic: String }, Webhook { url: String } }

pub struct AlertDispatcher { /* ntfy + webhook clients */ }

impl AlertDispatcher {
    pub async fn send_test(&self, channel: &AlertChannel) -> AlertTestResult { ... }
    pub async fn send_alert(&self, channel: &AlertChannel, ...) -> Result<(), String> { ... }
}
```

```rust
// src-tauri/src/services/export_service.rs
pub struct ExportService { pool: SqlitePool }

impl ExportService {
    pub async fn export_to(&self, path: &str) -> Result<ExportResult, ExportError> {
        // Query wishlist + price_history → ZIP con CSV files
    }
}
```

```rust
// src-tauri/src/services/image_cache.rs
// 688 líneas — LRU eviction (50MB), TTL (7 days), DashMap request coalescing,
// MIME allowlist (jpeg/png/webp/avif/gif), 21 tests.
pub struct ImageCacheService { ... }
impl ImageCacheService {
    pub async fn get(&self, url: &str) -> Result<(Vec<u8>, String)> { ... }
}
```

### Commands (Tauri IPC glue only) (🟢)

```rust
// src-tauri/src/commands/image_command.rs
#[tauri::command]
pub async fn get_product_image(image_url: String, state: ...) -> Result<String, String> {
    // 1. Validate URL: scheme=https, host in ALLOWED_DOMAINS, no IP literals
    // 2. ImageCacheService.get(url) → base64 data URI
    // 3. Return data:image/{mime};base64,...
}
```

```rust
// src-tauri/src/commands/price_command.rs
#[tauri::command]
pub async fn get_price_history(sku: String, state: ...) -> Result<Vec<PricePoint>, String> { ... }

#[tauri::command]
pub async fn get_price_insight(sku: String, state: ...) -> Result<PriceInsight, String> { ... }
```

```rust
// src-tauri/src/commands/settings_command.rs
#[tauri::command]
pub async fn get_setting(key: String, ...) -> Result<String, String> { ... }
#[tauri::command]
pub async fn save_setting(key: String, value: String, ...) -> Result<(), String> { ... }
#[tauri::command]
pub async fn test_alert_channel(channel: AlertChannel, ...) -> Result<AlertTestResult, String> { ... }
```

```rust
// src-tauri/src/commands/export_command.rs
#[tauri::command]
pub async fn export_data(path: String, ...) -> Result<ExportResult, String> { ... }
```

### Commands registrados en `main.rs`

```rust
// main.rs
.invoke_handler(tauri::generate_handler![
    guitarhub_lib::commands::image_command::get_product_image,
    guitarhub_lib::commands::price_command::get_price_history,
    guitarhub_lib::commands::price_command::get_price_insight,
    guitarhub_lib::commands::settings_command::get_setting,
    guitarhub_lib::commands::settings_command::save_setting,
    guitarhub_lib::commands::settings_command::test_alert_channel,
    guitarhub_lib::commands::export_command::export_data,
])
```

### AppState

```rust
// lib.rs
pub struct AppState {
    pub pool: sqlx::SqlitePool,
    pub image_cache_service: ImageCacheService,
    pub http_client: reqwest::Client,
}
```

---

## Sync State Machine 🟡 (parcial)

The `sync_state` table existe en SQLite (migration 001) con los estados: `idle`, `downloading`, `validating`, `sanitizing`, `inserting`, `done`, `failed_network`, `failed_schema`, `failed_db`.

🔴 **SyncService no está implementado** — no hay código Rust que recorra la state machine. La tabla existe pero no se escribe desde la app.

```
          ┌──────────────────────────────────────────────────────┐
          │                    IDLE                              │
          └──────────────────────┬───────────────────────────────┘
                                 │ user triggers / cron
                                 ▼
          ┌──────────────────────────────────────────────────────┐
          │               DOWNLOADING                            │
          │  GET catalog (retry ×3, exponential backoff 2s)      │
          │  Header: Accept-Encoding: gzip                        │
          └──────┬────────────────────────────────┬──────────────┘
     success     │                                │ network error / timeout
                 ▼                                ▼
          ┌──────────────┐                ┌────────────────────┐
          │  VALIDATING  │                │   FAILED_NETWORK   │
          │  schema major│                │   (keep old data)  │
          └──────┬───┬───┘                └────────────────────┘
    valid schema │   │ major mismatch
                 │   └────────────────────► FAILED_SCHEMA
                 ▼                          (show UpdatePrompt)
          ┌──────────────┐
          │  SANITIZING  │
          │  URL scheme  │
          │  validation  │
          └──────┬───────┘
                 │
                 ▼
          ┌──────────────┐
          │  INSERTING   │
          │  SQLite upsert│
          └──────┬───┬───┘
                 │   │ DB error
           done  │   └────────────────────► FAILED_DB
                 ▼                          (keep old data)
          ┌──────────────┐
          │     DONE     │
          │  emit event  │
          │  update      │
          │  sync_state  │
          └──────────────┘
```

**Invariants:**
- Previous SQLite state is NEVER destroyed before new data is fully inserted.
- `sync_state` table reflects the current state in real time; the UI reads from it.
- Each failure state includes an `error_msg` column for display in `SyncStatus`.
- `FAILED_SCHEMA` shows `UpdatePrompt` component (schema major version mismatch only).

---

## Local Image Cache 🟢 IMPLEMENTADO

Product images are loaded from external HTTPS URLs (Reverb, MercadoLibre, etc.). For true offline-first operation, the app caches images locally.

### Cache architecture

- **Storage**: SQLite BLOB store in `image_cache` table (`url_hash TEXT PK`, `blob BLOB`, `mime_type TEXT`, `size_bytes INTEGER`, `last_accessed INTEGER`, `created_at INTEGER`, `ttl_seconds INTEGER`)
- **Cache key**: SHA-256 hash of the image URL — deterministic, no collisions
- **Eviction**: Three-layer — LRU (`ORDER BY last_accessed ASC`), TTL (7 days default), maximum total size (50 MB default)
- **Concurrency**: `DashMap<url_hash, oneshot::Receiver>` coalesces concurrent requests — 10 products loading the same image produce 1 HTTP fetch
- **MIME allowlist**: Solo `image/jpeg`, `image/png`, `image/webp`, `image/avif`, `image/gif` — tipos desconocidos son rechazados (seguridad)
- **SSRF protection**: Domain allowlist + `url` crate parsing + IP literal rejection en `image_command.rs`
- **Frontend delivery**: `ImageCacheService` returns `data:image/<mime>;base64,...` strings via Tauri IPC; Svelte's `<img src>` receives the data URI directly (⚠️ **known issue**: data URIs en Svelte state causan memory bloat con 1000+ productos — migrar a Blob URLs como mejora futura)

### Cache flow

```
ProductCard requests image
       │
       ▼
ImageCacheService.get(url)
       │
       ├── cache HIT (TTL valid) → return base64 blob
       │
       ├── cache HIT (TTL expired, online) → re-fetch, update blob + TTL
       │
       ├── cache HIT (TTL expired, offline) → return stale blob (best-effort)
       │
       └── cache MISS → HTTP fetch → store blob → return base64
```

### Failure modes

| Scenario | Behavior |
|----------|----------|
| HTTP 404 / network error | Stale blob returned if available; empty string if no cache entry |
| Oversized image (>10 MB) | Skipped — logged as warning, not cached |
| DB write failure | Logged — image served from URL for that session |
| Non-allowlisted domain | Rejected at IPC boundary — no network call made |

---

## Security Model 🟢 (backend) / 🔴 (scraper pendiente)

### Scraper (GitHub Actions) 🔴

| Concern | Mitigation |
|---|---|
| Community adapter exfiltrates `GITHUB_TOKEN` | Scrape jobs run without secrets; only the separate `publish` job has `GITHUB_TOKEN` |
| Community adapter pushes malicious data to gh-pages | Only `publish` job can write to gh-pages; it only publishes artifacts that passed contract + health validation |
| Community adapter calls arbitrary external URLs | `BASE_URL` is declared in `SourcePort`; CI lint will verify adapter only requests its declared `BASE_URL` |
| Malicious adapter reads other secrets | No secrets in scope during scrape matrix jobs |

### App (Tauri / Rust) 🟢

| Concern | Mitigation |
|---|---|
| Catalog contains `javascript:` or `file://` URLs | `map_and_sanitize_vec()` drops any product whose `url` doesn't start with `https://` (pendiente — SyncService no implementado) |
| Malicious `image_url` schemes | Same sanitization — non-https image URLs replaced with empty string |
| MITM on catalog download | GitHub Pages uses HTTPS; `reqwest` validates TLS by default; no HTTP fallback |
| Sync command accepts arbitrary URLs | `sync_command` no implementado aún — el plan requiere validación con `url::Url::parse()` |
| Image cache downloads from arbitrary CDNs | 🟢 **IMPLEMENTADO**: `ALLOWED_DOMAINS` en `image_command.rs` — `reverb.com`, `mlstatic.com`. IP literals rechazados. |
| SQLite injection via product data | All inserts use parameterized queries via `sqlx` |
| FTS5 injection | Search terms sanitized: wrapped in double quotes, special characters escaped (pendiente — SearchService no implementado) |
| Image MIME confusion | 🟢 **IMPLEMENTADO**: solo `image/jpeg`, `image/png`, `image/webp`, `image/avif`, `image/gif` aceptados |
| Devcontainer runs as root | 🟢 **IMPLEMENTADO**: `"remoteUser": "vscode"` |
| Pre-commit secret scanning | 🟢 **IMPLEMENTADO**: `gitleaks` hook en `.pre-commit-config.yaml` |
| WAL mode | 🟢 **IMPLEMENTADO**: `PRAGMA journal_mode=WAL;` en `initialize_database()` |
| `.gitignore` | 🟢 **IMPLEMENTADO**: `target/`, `.env`, `__pycache__/`, `node_modules/`, `*.pyc`, `.DS_Store` |
| `cargo audit` thresholds | 🟢 **IMPLEMENTADO**: `.cargo-audit.toml` con `threshold = "high"` |
| Dependabot | 🟡 Parcial: solo Cargo configurado, el pip reference `scraper/` que no existe |
| Pipeline audit order | 🟡 `ci.yml` tiene `pip-audit` pero reference `scraper/requirements.txt` que no existe |

### Tauri configuration

```json
// tauri.conf.json — CSP restrictivo 🟢 IMPLEMENTADO
{
  "security": {
    "csp": "default-src 'self' customprotocol: asset:; connect-src ipc: http://ipc.localhost; img-src 'self' asset: http://asset.localhost blob: data: https:; style-src 'unsafe-inline' 'self'",
    "dangerousDisableAssetCspModification": false
  }
}
```

### Tauri 2 capabilities 🟢

**File**: `src-tauri/capabilities/default.json`
```json
{
  "identifier": "main-capability",
  "windows": ["main"],
  "permissions": [
    "core:default",
    "dialog:default",
    "notification:default",
    "notification:allow-notify",
    "notification:allow-is-permission-granted",
    "notification:allow-request-permission"
  ]
}
```

---

## Data Schema 🟢

### SQLite schema (local device) — 5 migraciones

```sql
-- Migration 001_init.sql

CREATE TABLE schema_meta (
  key   TEXT PRIMARY KEY,
  value TEXT NOT NULL
);
INSERT INTO schema_meta VALUES ('db_version', '1');

CREATE VIRTUAL TABLE products_fts USING fts5(
  sku, source_id, name, brand, model, category, subcategory, specs_json,
  tokenize = 'trigram',
  content = 'products_meta',
  content_rowid = 'rowid'
);

-- FTS triggers — IMPLEMENTADOS (antes missing, corregido en mvp-foundation)
CREATE TRIGGER IF NOT EXISTS products_fts_ai AFTER INSERT ON products_meta BEGIN
  INSERT INTO products_fts(rowid, sku, source_id, name, brand, model, category, subcategory, specs_json)
  VALUES (new.rowid, new.sku, new.source_id, new.name, new.brand, new.model, new.category, new.subcategory, new.specs_json);
END;
CREATE TRIGGER IF NOT EXISTS products_fts_ad AFTER DELETE ON products_meta BEGIN
  INSERT INTO products_fts(products_fts, rowid, sku, source_id, name, brand, model, category, subcategory, specs_json)
  VALUES ('delete', old.rowid, old.sku, old.source_id, old.name, old.brand, old.model, old.category, old.subcategory, old.specs_json);
END;
CREATE TRIGGER IF NOT EXISTS products_fts_au AFTER UPDATE ON products_meta BEGIN
  INSERT INTO products_fts(products_fts, rowid, sku, source_id, name, brand, model, category, subcategory, specs_json)
  VALUES ('delete', old.rowid, old.sku, old.source_id, old.name, old.brand, old.model, old.category, old.subcategory, old.specs_json);
  INSERT INTO products_fts(rowid, sku, source_id, name, brand, model, category, subcategory, specs_json)
  VALUES (new.rowid, new.sku, new.source_id, new.name, new.brand, new.model, new.category, new.subcategory, new.specs_json);
END;

-- Note: trigram tokenizer is slow on queries of 1-2 characters.
-- SearchService enforces a minimum of 3 characters before issuing FTS queries.
-- The UI provides feedback: "Type at least 3 characters to search."

CREATE TABLE products_meta (
  sku          TEXT PRIMARY KEY,
  source_id    TEXT NOT NULL,
  name         TEXT NOT NULL DEFAULT '',
  brand        TEXT NOT NULL DEFAULT '',
  model        TEXT NOT NULL DEFAULT '',
  category     TEXT NOT NULL DEFAULT '',
  subcategory  TEXT NOT NULL DEFAULT '',
  specs_json   TEXT NOT NULL DEFAULT '{}',
  price        REAL,
  currency     TEXT,
  condition    TEXT CHECK(condition IN ('new','used','refurbished','unknown')),
  availability TEXT CHECK(availability IN ('in_stock','out_of_stock','unknown')),
  url          TEXT NOT NULL CHECK(url LIKE 'https://%'),
  image_url    TEXT CHECK(image_url = '' OR image_url LIKE 'https://%'),
  seller       TEXT,
  location     TEXT,
  synced_at    INTEGER NOT NULL
);

CREATE TABLE sync_state (
  source_id   TEXT PRIMARY KEY,
  enabled     INTEGER DEFAULT 1,
  last_synced INTEGER,
  last_run_id TEXT,
  status      TEXT CHECK(status IN
                ('idle','downloading','validating','sanitizing',
                 'inserting','done',
                 'failed_network','failed_schema','failed_db')),
  error_msg   TEXT
);

CREATE TABLE wishlist (
  sku          TEXT PRIMARY KEY,
  added_at     INTEGER NOT NULL,
  price_at_add REAL,
  notes        TEXT
);

CREATE TABLE price_history (
  id          INTEGER PRIMARY KEY AUTOINCREMENT,
  sku         TEXT NOT NULL,
  price       REAL NOT NULL,
  recorded_at INTEGER NOT NULL
);

CREATE INDEX idx_price_history_sku    ON price_history(sku);
CREATE INDEX idx_products_meta_source ON products_meta(source_id);
CREATE INDEX idx_products_meta_price  ON products_meta(price);
CREATE INDEX idx_products_meta_cond   ON products_meta(condition);
```

```sql
-- Migration 002_add_url_validation.sql — CHECK constraints https://

CREATE TABLE IF NOT EXISTS products_meta_new (
    sku          TEXT PRIMARY KEY,
    source_id    TEXT NOT NULL,
    price        REAL,
    currency     TEXT,
    condition    TEXT CHECK(condition IN ('new','used','refurbished','unknown')),
    availability TEXT CHECK(availability IN ('in_stock','out_of_stock','unknown')),
    url          TEXT NOT NULL CHECK(url LIKE 'https://%'),
    image_url    TEXT CHECK(image_url = '' OR image_url LIKE 'https://%'),
    seller       TEXT,
    location     TEXT,
    synced_at    INTEGER NOT NULL
);
INSERT OR IGNORE INTO products_meta_new SELECT * FROM products_meta;
DROP TABLE IF EXISTS products_meta;
ALTER TABLE products_meta_new RENAME TO products_meta;
-- Recreate indexes
```

```sql
-- Migration 003_add_image_cache.sql

CREATE TABLE IF NOT EXISTS image_cache (
    url_hash      TEXT PRIMARY KEY,
    blob          BLOB NOT NULL,
    mime_type     TEXT NOT NULL DEFAULT 'image/jpeg',
    size_bytes    INTEGER NOT NULL,
    last_accessed INTEGER NOT NULL,
    created_at    INTEGER NOT NULL,
    ttl_seconds   INTEGER NOT NULL DEFAULT 604800  -- 7 days
);
CREATE INDEX IF NOT EXISTS idx_image_cache_last_accessed ON image_cache(last_accessed);
```

```sql
-- Migration 004_add_price_source.sql — add source_id to price_history

ALTER TABLE price_history ADD COLUMN source_id TEXT NOT NULL DEFAULT '';
CREATE INDEX IF NOT EXISTS idx_price_history_sku_recorded
  ON price_history(sku, recorded_at);
```

```sql
-- Migration 005_add_settings.sql

CREATE TABLE IF NOT EXISTS settings (
    key   TEXT PRIMARY KEY,
    value TEXT NOT NULL
);
```

---

## Database Migration Runner 🟢

The app uses a **custom lightweight migration runner** in Rust (not `sqlx::migrate!`) to keep schema upgrades atomic and auditable. **755 líneas, 15 tests.**

### How it works

- **Discovery**: `MigrationRunner` scans `src-tauri/src/repository/sqlite/migrations/` for `.sql` files with numeric prefix ordering (`001_`, `002_`, `003_`).
- **Version tracking**: The `schema_meta` table stores `db_version` key/value. Fresh DB → version 0. After migration `005`, db_version = 5.
- **Application**: Each pending migration runs in its own SQLite transaction. If migration `003` fails, `002` is already committed — the error is reported and the app can retry on next startup.
- **Gap detection**: If `001_`, `003_` are found but `002_` is missing, the runner errors immediately — no silent schema gaps.
- **Idempotency**: SQL files use `CREATE TABLE IF NOT EXISTS` and `INSERT OR IGNORE` so re-running a partially applied migration is safe.
- **Env override**: `GUITARHUB_MIGRATIONS_DIR` env var overrides the compile-time default path.

### Failure modes

| State | Behavior |
|-------|----------|
| Fresh DB | All migrations applied in order |
| Up-to-date DB | No-op (version matches latest migration) |
| Gap in sequence | Hard error — prevents startup |
| Corrupt `db_version` | Hard error — prevents silent misalignment |
| SQL failure in migration | Transaction rolled back; version NOT updated; error logged |

This runner is initialized on app startup: `initialize_database()` connects the pool, runs pending migrations, and panics only if the schema is unrecoverable (gap or corrupt version).

---

## Testing Strategy 🟢 (Rust) / 🔴 (scraper)

### App Backend (Rust) 🟢

| Layer | Tool | What's tested | Tests |
|---|---|---|---|
| Unit | `cargo test` | Services, commands, repository logic | ~123 tests across all modules |
| Integration | `cargo test` (inline) | SQLite repos with in-memory DB | Included in test suite |
| Security | `cargo test` | Domain allowlist, URL validation, SSRF prevention, MIME filtering | Included |

Rust tests están inline en cada archivo (`#[cfg(test)] mod tests`). Archivos con tests:
- `commands/image_command.rs`
- `commands/settings_command.rs`
- `services/alert_service.rs`
- `services/export_service.rs`
- `services/image_cache.rs`
- `repository/settings.rs`
- `repository/price_history.rs`
- `repository/sqlite/image_cache.rs`
- `repository/sqlite/settings.rs`
- `repository/sqlite/migrations/mod.rs`

### Scraper (Python) 🔴

No implementado. Se usará `pytest` con fixtures JSON, capas unit/contract/integration, y `jsonschema` para validación de schema.

---

## CI/CD Workflows ⚠️

### `ci.yml` — every PR (⚠️ ROTO — referencia `scraper/` que no existe)

```yaml
name: CI
on: [pull_request]

concurrency:
  group: ci-${{ github.ref }}
  cancel-in-progress: true

jobs:
  python:         # ⚠️ ROTO: ruff, mypy, pytest target scraper/ que no existe
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - run: pip install -r requirements-dev.txt --break-system-packages
      - run: ruff check scraper/          # FAIL — directorio no existe
      - run: mypy scraper/ --strict       # FAIL — directorio no existe
      - run: pytest scraper/tests/...     # FAIL — directorio no existe

  rust:             # 🟢 Funciona
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo clippy --all-targets -- -D warnings
        working-directory: src-tauri
      - run: cargo test
        working-directory: src-tauri
      - run: cargo audit
        working-directory: src-tauri
```

### `scrape.yml` — cron every 6h (⚠️ ROTO — scraper no implementado)

Workflow completo depende de `scraper/` que no existe. **Debe ser corregido o eliminado hasta que el scraper esté implementado.**

### `release.yml` — on tag `v*` (🟡 Parcial)

Builds Rust para windows/linux/macos. No tiene `tauri-action` configurado aún — solo compila con `cargo test` y `cargo build`.

### `e2e.yml` — weekly (⚠️ ROTO — referencia scraper que no existe)

---

## Implementation Status (2026-06-04) — v7

### 🟢 Implementado y funcionando

| Componente | Detalle | Tests |
|---|---|---|
| **MigrationRunner** | 755 líneas, gap detection, transaction-per-migration, env override | 15 tests |
| **ImageCacheService** | 688 líneas, LRU (50MB), TTL (7d), DashMap coalescing, MIME allowlist | 21 tests |
| **SSRF protection** | Domain allowlist + url crate + IP literal rejection | Tests en image_command |
| **AlertService** | 545 líneas, App/Ntfy/Webhook dispatchers, URL validation | Tests inline |
| **ExportService** | 425 líneas, ZIP con wishlist + price_history | Tests inline |
| **PriceHistoryRepo** | Stats con outlier filtering, multi-source | Tests inline |
| **Settings** | Trait + Sqlite impl + Mock, key-value store | Tests inline |
| **tauri.conf.json** | CSP, window config, build commands | — |
| **Capabilities** | core + dialog + notification | — |
| **FTS5 triggers** | Corregidos en 001_init.sql (antes missing) | — |
| **WAL mode** | `PRAGMA journal_mode=WAL` en initialize_database() | — |
| **Security hardening** | `.cargo-audit.toml`, `gitleaks`, `.gitignore`, `rust-toolchain.toml` | — |

### 🟡 Parcialmente implementado

| Componente | Estado |
|---|---|
| **Frontend Svelte** | 5 componentes sueltos (`src/lib/components/`). Sin SvelteKit, sin `package.json`, sin routes, sin stores, sin i18n |
| **CI workflows** | Rust job funciona. Python/scraper jobs rotos (referencian `scraper/` que no existe) |
| **Dependabot** | Cargo funciona. Pip reference `scraper/` que no existe |
| **Packaging** | Metadatos en `scripts/packaging/`. Builds reales no configurados |
| **CONTRIBUTING.md** | Existe pero referencia `scraper/` que no existe |
| **Makefile** | Targets de Rust funcionan. Targets de scraper con fallback |

### 🔴 No implementado

| Componente | Prioridad |
|---|---|
| **Scraper Python** (`scraper/` completo) | Alta — bloquea MVP |
| **SvelteKit scaffold** (`package.json`, routes, stores, i18n) | Alta — bloquea MVP |
| **SyncService** (state machine + schema validation + delta sync) | Alta |
| **SearchService** (FTS5 con filtros y sanitize) | Alta |
| **Domain models** (`Product`, `Condition`, `Availability`) en Rust | Alta |
| **README.md** | Media |
| **LICENSE** (GPL-3.0) | Media |
| **CHANGELOG.md** | Media |
| **CODE_OF_CONDUCT.md** | Media |
| **Documentación** (`ARCHITECTURE.md`, `GOVERNANCE.md`, `SCHEMA.md`, etc.) | Media |
| **Landing page** (Astro) | Baja |
| **Auto-update** (tauri-plugin-updater + latest.json) | Baja |
| **GitHub templates** (ISSUE_TEMPLATE, PR_TEMPLATE) | Baja |
| **iOS support** | Futuro |

---

## Roadmap Actualizado

### ✅ Completado (cambios SDD archivados)

- **Fix v4 findings** — Correcciones de la revisión adversarial v4
- **Plan v3 revision** — db-migration-runner + local-image-cache specs y diseño
- **MVP Foundation** — Tauri wiring, security hardening, CI/CD hardening, repo hygiene (14 tasks)
- **Price Intelligence Phase 3** — Price history, price insight, alert delivery (3 canales), data export ZIP, settings, image cache (25 tasks, 3 PRs)

### 🎯 Próximos pasos recomendados

1. **Corregir CI roto**: Desactivar jobs de Python en workflows hasta que `scraper/` exista, o crear stub
2. **Frontend scaffold**: `package.json`, SvelteKit, Vite, TypeScript strict
3. **Domain models + SyncService**: `Product`, `Condition`, `Availability`, state machine, delta sync
4. **SearchService**: FTS5 queries con sanitize + min 3 chars + filters
5. **README.md + LICENSE**: Que el repo sea presentable
6. **Scraper MVP**: Reverb adapter + MercadoLibre adapter (Phase 0 del plan original)

---

## Tech Stack (actualizado)

| Layer | Technology | Estado |
|---|---|---|
| App framework | Tauri 2.x (stable) | 🟢 |
| UI | Svelte 5 | 🟡 (componentes sueltos) |
| i18n | `svelte-i18n` | 🔴 |
| App backend | Rust | 🟢 |
| App logging | `tracing` + `tracing-subscriber` | 🟢 |
| Frontend scaffold | Svelte 5 + Vite + TypeScript strict | 🔴 |
| Local DB | SQLite + FTS5 | 🟢 |
| DB migrations | Custom `MigrationRunner` | 🟢 |
| URL parsing | `url` crate v2 | 🟢 |
| Image cache | SQLite BLOB + SHA-256 + LRU/TTL | 🟢 |
| Auto-update | `tauri-plugin-updater` | 🔴 |
| Scraper architecture | Ports & Adapters (Python 3.12) | 🔴 |
| HTTP (Rust) | reqwest | 🟢 |
| CI/CD | GitHub Actions | ⚠️ (rotos los jobs de Python) |
| Security scanning | `cargo audit`, `pip-audit`, Dependabot | 🟡 |
| Catalog hosting | GitHub Pages | 🔴 |
| Landing page | Astro | 🔴 |
| Packaging | AppImage, .deb, .msi, .dmg, APK | 🟡 |
| License | GPL-3.0 (app) + MIT (scraper) | 🔴 (archivos faltantes) |
