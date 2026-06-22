---
name: guitarhub-rust-backend
trigger: Touch src-tauri/ (commands, services, repository)
scope: rust
---

# Rust Backend Skill — GuitarHub

## Architecture Rules

When working on `src-tauri/`, follow these MANDATORY patterns:

### Layer Separation (Clean Architecture)

```
commands/  → Thin IPC glue only. Delegate to services/. NO business logic.
services/  → All business logic. NO direct DB access.
repository/ → Data access layer. Trait-based abstraction.
domain/    → Pure entities. Zero imports from I/O.
```

### Code Style

- `cargo fmt` before any commit
- `clippy -D warnings`: zero warnings tolerated
- `Result<T, E>` explicit everywhere. NO `.unwrap()` in production code
- Doc comments `///` on every public function and struct
- Migrations: create new with next number, NEVER modify existing

### Testing

- Unit tests in `#[cfg(test)]` modules
- Integration tests use in-memory SQLite (`:memory:`)
- Run: `make test-app` or `cargo test` from `src-tauri/`

### Error Handling

Use the unified `AppError` enum in `lib.rs`. Add new variants with `#[serde(rename)]` for IPC serialization.

### Database

- SQLite with FTS5 trigram tokenizer
- WAL journal mode enabled
- Custom migration runner (no external crate)
- All SQL in `repository/sqlite/` only
