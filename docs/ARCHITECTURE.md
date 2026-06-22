# GuitarHub Architecture

## Overview

GuitarHub is a **Tauri v2** desktop application composed of three main layers:

- **Svelte 5 frontend** (TypeScript) — UI layer with stores, routes, and components
- **Rust backend** — clean architecture with Commands → Services → Repositories
- **Python scraper pipeline** — hexagonal/ports-and-adapters for marketplace data ingestion
- **SQLite database** — local storage with FTS5 full-text search, WAL mode, and a custom migration runner

## Module Tree

```
guitarhub/
├── src/                          # Svelte 5 frontend
│   ├── lib/
│   │   ├── components/           # UI components
│   │   ├── stores/               # Svelte stores
│   │   └── types/                # TypeScript interfaces
│   └── routes/                   # SvelteKit routes
├── src-tauri/                    # Rust backend
│   └── src/
│       ├── commands/             # Tauri IPC commands
│       ├── services/             # Business logic
│       ├── repository/           # Data access layer
│       └── domain/               # Domain types
├── scraper/                      # Python scraper
│   ├── domain.py                 # Domain models (Pydantic)
│   ├── ports.py                  # ScraperPort Protocol
│   ├── adapters/                 # Source adapters (Reverb)
│   └── tests/                    # Unit + contract tests
└── .github/workflows/            # CI/CD
```

## IPC Flow

Data moves between the frontend and backend through Tauri's invoke mechanism:

```
Svelte component → invoke() → Tauri Command → Service → Repository → SQLite
```

1. **Svelte component** calls `invoke('command_name', { args })`
2. **Tauri Command** (in `src-tauri/src/commands/`) validates input and delegates to a service
3. **Service** (in `src-tauri/src/services/`) holds business logic and orchestrates operations
4. **Repository** (in `src-tauri/src/repository/`) abstracts SQLite access
5. **SQLite** stores all local data with FTS5 for full-text search

## Data Flow

Data is ingested through the scraper pipeline and consumed by the desktop app:

```
External API (Reverb) → Scraper Adapter → Catalog JSON → Sync Service → SQLite
```

1. **Scraper** (Python, GitHub Actions cron every 6h) fetches listings from marketplace APIs
2. **Catalog JSON** is published to the `gh-pages` branch
3. **Sync Service** (Rust) downloads, validates, and inserts catalog data into the local SQLite database
4. **Frontend** queries via Tauri commands for search, browsing, and collection management

## Design Decisions

| Decision | Rationale | ADR |
|----------|-----------|-----|
| **Offline-first with local SQLite + FTS5** | Users browse the full catalog without internet; FTS5 enables fast full-text search locally | [ADR-001](adr/001-offline-first-sqlite-fts5.md) |
| **Clean architecture in Rust** | Commands are thin IPC glue, services hold business logic, repositories handle data access — makes testing and swapping storage trivial | [ADR-002](adr/002-clean-architecture-rust.md) |
| **Ports-and-adapters in Python** | Adding a new marketplace only requires implementing a `ScraperPort` interface; no pipeline changes needed | [ADR-003](adr/003-ports-and-adapters-scraper.md) |
| **CSP security headers in Tauri config** | Mitigates XSS and data injection risks by restricting sources for scripts, styles, and connections | [ADR-004](adr/004-csp-security-headers.md) |

## Error Handling

The app uses a unified `AppError` enum with typed variants:

| Variant | Description |
|---------|-------------|
| `NotFound` | Requested resource does not exist |
| `InvalidInput` | User-provided data failed validation |
| `Database` | SQLite operation failed |
| `Network` | External API or download failed |
| `Internal` | Unexpected internal error |
| `SyncInProgress` | Catalog sync is currently running |

All commands return `Result<T, AppError>`, which Tauri serializes to the frontend as structured error objects.
