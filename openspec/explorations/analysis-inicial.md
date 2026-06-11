# Exploration: GuitarHub — Análisis inicial completo

## Resumen ejecutivo

GuitarHub es una app de escritorio nativa, offline-first, que agrega listados de guitarras, amplificadores, pedales y accesorios de múltiples tiendas online en un catálogo unificado. Sigue el modelo de **Mihon para guitarras**: gratis, open source, sin servidor backend, sin anuncios, sin registro. El usuario instala la app, sincroniza el catálogo y busca sin conexión.

## Stack detallado

| Capa | Tecnología | Versión |
|------|-----------|---------|
| App framework | **Tauri 2** | ^2.0 (wry) |
| Frontend | **Svelte 5** + SvelteKit 2 | ^5.0 / ^2.0 |
| Backend | **Rust** (edition 2021) | stable |
| DB local | **SQLite** + FTS5 (trigram tokenizer) | via sqlx 0.9 |
| Scraper | **Python 3.12** | >=3.12 |
| Frontend tooling | **Vite 6** + TypeScript 5 | ^6.0 / ^5.0 |
| Testing (frontend) | **Vitest 3** + jsdom + Testing Library | ^3.2.6 |
| Testing (e2e) | **WebdriverIO 9** + tauri-driver | ^9.27 |
| Testing (Rust) | **cargo test** + httpmock 0.8 + tempfile | — |
| Testing (Python) | **pytest** + ruff + mypy --strict | — |
| CI/CD | **GitHub Actions** | — |
| Linting | **pre-commit** (ruff, mypy, clippy, rustfmt, gitleaks) | — |
| Packaging | AppImage, .deb (Linux), .dmg (macOS deferred), .msi (deferred) | — |

### Dependencias clave (Rust)

```toml
sqlx = { features: ["runtime-tokio", "sqlite", "derive"] }
tauri = { features: ["wry"] }
reqwest = { features: ["rustls-tls", "json"] }
tauri-plugin-dialog, tauri-plugin-notification, tauri-plugin-updater
dashmap, sha2, base64, url, zip, async-trait, tokio, thiserror, tracing
```

### Dependencias clave (Python scraper)

```
curl_cffi>=0.15.0, requests, beautifulsoup4, pydantic>=2.5
```

## Estructura del proyecto

```
guitarhub/
├── src/                          # Svelte 5 frontend (SvelteKit static adapter)
│   ├── app.html                  # HTML shell
│   ├── setupTests.ts             # Vitest setup (mocks Tauri invoke global)
│   ├── routes/
│   │   ├── +layout.svelte        # Nav + sync + layout global
│   │   ├── +page.svelte          # Dashboard (bento grid de 9 celdas)
│   │   ├── wishlist/+page.svelte # Wishlist page
│   │   ├── collection/+page.svelte
│   │   ├── settings/+page.svelte
│   │   └── __tests__/page.test.ts
│   └── lib/
│       ├── components/           # 8 componentes Svelte
│       │   ├── DashboardCell.svelte, FilterBar.svelte,
│       │   │   ProductCard.svelte, PriceBadge.svelte,
│       │   │   PriceChart.svelte, ProductDetail.svelte,
│       │   │   Settings.svelte, CollectionView.svelte
│       │   └── __tests__/        # 7 test files (uno por componente)
│       ├── stores/               # Svelte stores (writable)
│       │   ├── sync.ts, wishlist.ts, filter.ts, dashboard.ts, collection.ts
│       │   └── __tests__/        # filter.test.ts, collection.test.ts
│       ├── types/                # TypeScript mirrors de tipos Rust
│       │   ├── search.ts, wishlist.ts, price.ts, collection.ts
│       └── utils/
│           ├── collectionValue.ts
│           └── __tests__/collectionValue.test.ts
│
├── src-tauri/                    # Rust backend
│   ├── Cargo.toml / Cargo.lock
│   ├── tauri.conf.json           # CSP, updater, ventana 1200x800
│   └── src/
│       ├── main.rs               # Entrypoint Tauri + registro de commands
│       ├── lib.rs                # AppState, initialize_database, AppError
│       ├── domain/
│       │   └── product.rs        # SyncState, SearchFilters, SortOrder, etc.
│       ├── commands/             # 10 command files (IPC glue)
│       │   ├── search_command, price_command, sync_command,
│       │   │   settings_command, image_command, export_command,
│       │   │   dashboard_command, collection_command, wishlist_command
│       │   └── mod.rs
│       ├── services/             # 6 archivos (business logic)
│       │   ├── search.rs         # FTS5 sanitization + pagination
│       │   ├── sync.rs           # CatalogSyncService + state machine
│       │   ├── image_cache.rs    # LRU cache + request coalescing
│       │   ├── alert_service.rs  # Ntfy, Webhook, App notification
│       │   ├── price_drop.rs     # Pure drop detector
│       │   └── export_service.rs # ZIP export
│       └── repository/           # Data access layer
│           ├── settings.rs       # SettingsRepository trait + Mock
│           ├── wishlist.rs, dashboard.rs, price_history.rs
│           ├── collection.rs, price_drop_notifications.rs
│           └── sqlite/
│               ├── mod.rs, settings.rs, image_cache.rs
│               └── migrations/   # 009 migraciones SQL
│                   ├── 001_init.sql a 009_add_recent_searches.sql
│                   └── mod.rs    # MigrationRunner custom
│
├── scraper/                      # Python scraper (Ports & Adapters)
│   ├── __init__.py, __main__.py, cli.py
│   ├── domain.py                 # CatalogProduct, CatalogFile (Pydantic)
│   ├── ports.py                  # ScraperPort protocol
│   ├── adapters/
│   │   └── reverb.py             # ReverbAdapter (único adapter)
│   ├── requirements.txt
│   ├── pyproject.toml            # mypy --strict config
│   └── tests/
│       ├── unit/test_domain.py
│       ├── unit/test_reverb.py
│       └── contract/test_protocol.py
│
├── .github/workflows/            # 4 workflows CI/CD
│   ├── ci.yml                    # PR: lints + tests (Python, frontend, Rust)
│   ├── scrape.yml                # Scraper cron cada 6h → gh-pages
│   ├── release.yml               # Tag v* → build + sign + release
│   └── e2e.yml                   # E2E weekly (tauri-driver + wdio)
│
├── openspec/
│   ├── config.yaml               # SDD config (strict TDD)
│   ├── specs/                    # 33 spec files
│   └── changes/                  # 3 cambios activos (archive, docs-audit, mvp-ui)
│
├── scripts/
│   ├── generate_latest_json.py   # Genera latest.json para updater
│   └── packaging/                # Scripts de empaquetado
│
├── e2e-tests/                    # WebdriverIO E2E tests
├── docs/                         # ARCHITECTURE.md, CONTRIBUTING.md, RELEASE.md
└── catalog-reverb.json           # Catálogo de ejemplo (4088 productos)
```

## Estado del código

### ✅ Completado y funcional

**Rust backend (altamente completo)**
- `lib.rs`: AppState, `initialize_database()`, `AppError` unificado con 6 variantes, conversiones `From<sqlx::Error>` y `From<anyhow::Error>`
- **commands/**: 10 módulos de comandos Tauri registrados en `main.rs`:
  - `search_command`: FTS5 search, filters, pagination
  - `sync_command`: Catalog sync trigger
  - `price_command`: Price history + insights
  - `settings_command`: CRUD settings
  - `image_command`: Product image get
  - `export_command`: ZIP export
  - `dashboard_command`: Total products, wishlist count, recent searches, categories
  - `collection_command`: CRUD collection items + stats
  - `wishlist_command`: CRUD wishlist
- **services/**: Lógica de negocio completa:
  - `FtsSearchService`: FTS5 con sanitización de input, filtros dinámicos, paginación, 5 modos de ordenamiento
  - `CatalogSyncService`: State machine (idle→downloading→validating→sanitizing→inserting→done), upsert de productos, detección de price drops con cooldown
  - `ImageCacheService`: LRU cache con SQLite BLOBs, request coalescing (DashMap + watch channels), stale fallback, domain allowlist, 7-day TTL
  - `AlertDispatcher` trait con 3 implementaciones: `AppNotificationAlert`, `NtfyAlert`, `WebhookAlert`
  - `is_price_drop()`: Pure function con thresholds configurables (10% relativo / $50 absoluto)
  - `ExportService`: ZIP con wishlist, price_history, settings, collection_items como JSON
- **repository/**: Traits + implementaciones SQLite + Mock para testing
  - `SettingsRepository` trait + `SqliteSettingsRepository` + `MockSettingsRepository`
  - `DashboardRepo`, `WishlistRepo`, `PriceHistoryRepo` (5σ outlier filter), `CollectionRepo`, `PriceDropNotificationsRepo`
- **migrations/**: 009 migraciones con `MigrationRunner` custom
  - WAL mode, FTS5 con trigram tokenizer, triggers de sincronización
  - CHECK constraints para URLs HTTPS, condition, availability
- **Tests Rust**: 150+ tests en services, repository, migrations, domain

**Python scraper**
- Arquitectura Ports & Adapters: `ScraperPort` protocol + `ReverbAdapter`
- Domain models con Pydantic: `CatalogProduct`, `CatalogFile`
- CLI completo: scrape, validate, validate-input
- Tests: unit test del domain, contract test del protocol
- mypy --strict habilitado con overrides para adapters

**Frontend Svelte 5**
- Layout global con nav, sync button, wishlist badge
- Dashboard con bento grid de 9 celdas (search + product grid + stats)
- FilterBar con 6 filtros (category, price range x2, condition, currency, sort)
- Filter state con URL sync (debounced 300ms)
- 8 componentes UI: DashboardCell, FilterBar, ProductCard, PriceBadge, PriceChart, ProductDetail, Settings, CollectionView
- Stores: sync, wishlist, filter, dashboard, collection
- Types: search, wishlist, price, collection (mirrors de Rust)
- Tests: 7 component tests + 1 page test + 2 store tests + 1 util test = ~11 test files
- Tauri invoke mockeado globalmente en setupTests.ts

**CI/CD**
- CI: 3 jobs paralelos (python, frontend, rust) en PR
- Scrape: cron cada 6h, deploy a GitHub Pages
- Release: tag v* → build + sign + GitHub Release + latest.json en gh-pages
- E2E: weekly, xvfb + tauri-driver
- Dependabot: cargo weekly
- pre-commit: trailing-whitespace, ruff, mypy, clippy, rustfmt, gitleaks

### 🔄 Parcial/Plan-only

- **MVP UI**: 3 cambios en openspec/changes/ (mvp-ui, archive, docs-audit) — no implementados
- **E2E tests**: Config de WebdriverIO existe pero sin tests escritos
- **SvelteKit coverage**: Configurado pero sin threshold
- **macOS/Windows builds**: Deferidos en release matrix

### ❌ No iniciado / Pendiente

- Landing page con Astro (mencionado en openspec config pero sin implementar)
- Packaging APK, AUR, Flathub, F-Droid (planeado pero no iniciado)
- Temas oscuro/claro completos (solo media queries parciales en +page.svelte)
- Ventana de settings completa (Settings.svelte existe pero sin lógica completa)

## Arquitectura actual

```
┌─────────────────────────────────────────────────────┐
│                   Svelte 5 Frontend                  │
│  +page.svelte → invoke('command', {args})            │
│  stores (writable) ←→ componentes                    │
└──────────────────────┬──────────────────────────────┘
                       │ IPC (Tauri invoke/serde_json)
┌──────────────────────▼──────────────────────────────┐
│              Tauri Commands (thin glue)               │
│  commands/*_command.rs → validación + delegación     │
└──────────────────────┬──────────────────────────────┘
                       │
┌──────────────────────▼──────────────────────────────┐
│              Services (business logic)                │
│  search, sync, image_cache, alerts, price_drop,      │
│  export                                              │
└──────────────────────┬──────────────────────────────┘
                       │
┌──────────────────────▼──────────────────────────────┐
│            Repository (data access layer)             │
│  Traits + SQLite impls + Mock impls                  │
│  settings::SettingsRepository trait                  │
│  sqlite::* — impls concretas                         │
└──────────────────────┬──────────────────────────────┘
                       │
┌──────────────────────▼──────────────────────────────┐
│                SQLite (local file)                    │
│  WAL mode, FTS5 trigram, 9 migrations                │
│  Tablas: products_meta, products_fts, sync_state,    │
│  wishlist, price_history, settings, image_cache,     │
│  price_drop_notifications, collection_items,         │
│  recent_searches, schema_meta                        │
└─────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────┐
│         Python Scraper (GitHub Actions cron)          │
│  ReverbAdapter → ScraperPort → CatalogFile           │
│  Salida: catalog-reverb.json → gh-pages branch        │
└─────────────────────────────────────────────────────┘
```

### Flujo de sync de catálogo

```
Scraper cron (GitHub Actions cada 6h)
  → Python ReverbAdapter.scrape()
  → CatalogFile JSON
  → git push a gh-pages (CDN free)
  → Usuario clickea "Sync Catalog"
  → CatalogSyncService.sync_catalog(url)
  → HTTP GET catalog.json desde GitHub Pages
  → Deserializa → valida → upsert en products_meta
  → Escribe price_history
  → Detecta price drops
  → Si hay drops, dispatchea alertas (app/ntfy/webhook)
```

### Flujo de búsqueda

```
Usuario escribe query
  → FtsSearchService.search()
  → sanitize_fts_input() (limpia operadores FTS5)
  → Construye SQL dinámico con filtros opcionales
  → COUNT subquery + DATA subquery
  → JOIN con products_meta para filtros
  → ORDER BY rank/price/name + LIMIT/OFFSET
  → RawProductRow → RawProduct → SearchResult
  → Serializa vía serde → Tauri IPC → frontend
```

## Decisiones arquitectónicas clave

| Decisión | Por qué |
|----------|---------|
| **Offline-first con SQLite local** | Catálogo completo sin conexión, sin servidor backend, cero costos |
| **FTS5 con trigram tokenizer** | Búsqueda tolerante a typos, funciona para cualquier idioma (incluyendo CJK como "吉他") |
| **Clean Architecture en Rust** | Commands delgados, services con lógica pura, repositories testables con Mock |
| **Ports & Adapters en Python** | Agregar un marketplace = implementar ScraperPort, sin tocar el pipeline |
| **Modelo "sin servidor"** | Scraper corre en GitHub Actions cron, catálogo en gh-pages, app consume JSON |
| **Cache de imágenes LRU con coalescencia** | Múltiples requests concurrentes por la misma URL hacen 1 sola llamada HTTP |
| **AppError unificado** | Serialización consistente al frontend, Display implementado para cada variante |
| **CSP estricto** | default-src 'self', connect-src restringido a ipc, img-src con allowlist |
| **Migrations custom** | Sin dependencia externa, cada migration en su propia transacción, gap detection |
| **Strict TDD** | openspec config con tdd:true, test_command en cada fase |

## Configuración de testing

### Rust (cargo test)
- Ubicación: tests inline en cada módulo (`#[cfg(test)]`)
- Cobertura: domain, services (search, sync, image_cache, alerts, price_drop, export), repository (wishlist, dashboard, price_history, settings, sqlite), migrations (runner + cada migración)
- Mocking: httpmock para HTTP, MockSettingsRepository para settings, in-memory SQLite pools
- ~150+ tests, todos unitarios (sin integration --ignored)

### Python (pytest)
- Unit: `scraper/tests/unit/test_domain.py`, `test_reverb.py`
- Contract: `scraper/tests/contract/test_protocol.py` (verifica que ReverbAdapter conforma ScraperPort)
- Linting: ruff check, mypy --strict
- Security: pip-audit

### Frontend (vitest)
- Unit: 11 test files (componentes, stores, utils)
- Mock global de `@tauri-apps/api/core` → `vi.mock('invoke')`
- jsdom environment
- Config: coverage v8 en src/lib

### E2E (WebdriverIO + tauri-driver)
- Config: `vitest.e2e.config.ts` + `e2e-tests/wdio.conf.ts`
- Corre semanal en CI (sábados) o manual
- Requiere `cargo tauri build --debug --no-bundle`

## Riesgos y bloqueantes

1. **Sin migración de wishlist a 10 columnas**: La migration 006 recrea wishlist pero el código de `WishlistRepo` espera 10 columnas. Datos pre-006 se preservan (test existente), pero si hay datos de producción hay que verificar la migración.

2. **Sin mecanismo de rollback de migraciones**: El `MigrationRunner` no implementa down migrations. Una migration fallida queda en estado inconsistente.

3. **Frontend coverage sin threshold**: `coverage_threshold: 0` en openspec. No hay métrica que enforce.

4. **macOS/Windows builds deferidos**: release.yml solo build para Linux. Si hay usuarios de otras plataformas, no hay binarios.

5. **Sin Astro landing page**: Mencionado en openspec pero sin implementar.

6. **Sin tests E2E escritos**: Config de wdio existe pero directorio `e2e-tests/` vacío de tests.

7. **Category mapping incompleto**: En el catalog-reverb.json de ejemplo, `category` y `subcategory` vienen vacíos (`""`). El scraper no mapea categorías de Reverb.

8. **Scraper sin dockerizar**: Corre directo en GitHub Actions. Para desarrollo local falta un entorno reproducible.

9. **Sin caché de catálogo después de sync**: El `CatalogSyncService` descarga el JSON cada vez que se sincroniza. Podría cachear etag/last-modified.

10. **Sin manejo de errores de red en frontend**: El layout muestra error de sync pero se autolimpia con setTimeout. No hay retry mechanism.

## Recomendaciones

### Corto plazo (MVP)
1. **Categorizar productos en scraper**: Mapear `category` y `subcategory` desde los datos de Reverb (product_type, category_slug, etc.)
2. **Completar frontend faltante**: Settings page con configuración real, collection page full CRUD
3. **Agregar tests E2E básicos**: Login alternativo (la app no tiene login), flujo de sync, búsqueda
4. **Setear coverage threshold**: Empezar con 60% y subir gradualmente

### Mediano plazo
5. **Implementar landing page con Astro** (como está planeado en openspec)
6. **Dockerizar scraper** para desarrollo local reproducible
7. **Agregar HTTP caching** en CatalogSyncService (If-None-Match / If-Modified-Since)
8. **Completar plataformas faltantes**: macOS build + Windows build
9. **Implementar down migrations** en MigrationRunner

### Largo plazo
10. **Empaquetado flatpak/flathub + snap + AUR**
11. **APP móvil con Tauri mobile** (Tauri 2 ya soporta Android/iOS)
12. **Más adapters de scraper**: GuitarCenter, Sweetwater, Thomann, MercadoLibre
13. **Sync automático en background** con schedule configurable
14. **Plugin system** para adapters de terceros

## Ready for Proposal

Sí. La exploración es completa y el proyecto está en un estado excelente para comenzar cambios planificados. El backend Rust está muy completo, el frontend Svelte 5 tiene estructura sólida, y el pipeline CI/CD está funcionando. Recomiendo empezar con cambios pequeños tipo "mvp-ui" o "docs-audit" que ya están en openspec/changes/.
