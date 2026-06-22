# ADR-002: Clean Architecture in Rust Backend

## Status

Accepted

## Context

The Rust backend handles IPC between the Svelte frontend and SQLite database. Without clear layer separation, business logic can leak into IPC handlers, making the code hard to test and maintain. The backend needs to be:
- Testable in isolation (services without IPC)
- Maintainable as features grow
- Clear about where business logic lives

## Decision

Implement Clean Architecture with three distinct layers:

```
commands/  → Thin IPC glue only. Validates input, delegates to services.
services/  → All business logic. Orchestrates operations, applies rules.
repository/ → Data access layer. Trait-based abstraction over SQLite.
```

Key rules:
- Commands contain NO business logic — they validate input and call services
- Services contain NO direct database access — they use repository traits
- Repositories are the ONLY place SQL lives
- Domain entities in `domain/` have zero imports from I/O or frameworks

Example flow:
```
Svelte invoke() → search_command.rs → FtsSearchService → ProductRepository trait → SqliteProductRepository
```

## Consequences

**Easier:**
- Services can be unit-tested with mock repositories
- Swapping storage (e.g., for testing) only requires a new repository impl
- Clear responsibility boundaries — easy to find where logic lives
- Commands stay small and readable (avg 23 lines)

**More difficult:**
- More files and modules to navigate
- Trait abstraction adds indirection
- New contributors need to understand the layer hierarchy

## Alternatives Considered

1. **Fat commands with inline SQL** — Rejected: Hard to test, logic scattered across IPC handlers, unmaintainable at scale
2. **Service layer without repository abstraction** — Rejected: Tightly couples business logic to SQLite, makes testing harder
3. **Actor model (Actix)** — Rejected: Overkill for this app's concurrency needs; Tauri already manages async via Tokio
