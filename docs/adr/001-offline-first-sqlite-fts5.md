# ADR-001: Offline-first with SQLite + FTS5

## Status

Accepted

## Context

GuitarHub is a desktop app for browsing guitar gear listings. Users may have unreliable internet connections or want to browse catalogs without being online. Search is a core feature — users need to find products by name, brand, model, or category quickly.

The app needs to:
- Store the full catalog locally
- Support fast full-text search across all product fields
- Work completely offline after initial sync
- Keep the database small enough for a desktop app

## Decision

Use SQLite as the local database with FTS5 (Full-Text Search) extension using the trigram tokenizer. Enable WAL (Write-Ahead Logging) journal mode for concurrent read access during sync.

Key implementation details:
- FTS5 trigram tokenizer for fuzzy search (handles typos, partial matches)
- `sanitize_fts_input()` strips FTS5 operators and wraps terms in quotes to prevent injection
- Custom migration runner (no external crate) for schema management
- WAL mode enabled before any schema operations
- All data stored locally, synced from GitHub Pages catalog JSON

## Consequences

**Easier:**
- Full catalog available offline after first sync
- Fast search with trigram tokenizer (sub-100ms for typical queries)
- No server costs — all data lives on the user's machine
- Simple deployment — no backend to maintain

**More difficult:**
- Database size grows with catalog size (mitigated by compression)
- Sync conflict resolution needed if catalog updates while user has stale data
- FTS5 index adds overhead to writes during sync

## Alternatives Considered

1. **IndexedDB via Tauri webview** — Rejected: Tauri has first-class SQLite support; IndexedDB would add complexity without benefit
2. **Server-side search API** — Rejected: Violates offline-first principle, adds server costs, creates tracking risk
3. **SQLite without FTS5** — Rejected: LIKE queries are too slow for full-text search on large catalogs
4. **External search engine (MeiliSearch, Typesense)** — Rejected: Adds runtime dependency, violates zero-server-cost principle
