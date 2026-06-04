# Design: DB Migration Runner

## Technical Approach

Custom lightweight migration runner that discovers `.sql` files by numeric prefix, applies them in order, and tracks state via the existing `schema_meta.db_version` key. No new dependencies — plain `sqlx::query()` execution with transaction wrapping.

## Architecture Decisions

### Decision: Custom runner vs `sqlx::migrate!`

| Option | Tradeoffs | Verdict |
|--------|-----------|---------|
| `sqlx::migrate!` | Creates its own `_sqlx_migrations` table; compile-time macro reduces flexibility; designed for multi-backend (irrelevant — SQLite only) | Rejected |
| **Custom runner** | Uses `schema_meta` as already designed; no new tracking table; testable with temp dirs; <100 LOC | **Chosen** |

**Rationale**: The project already defines `schema_meta.db_version` in `001_init.sql` — using a second tracking table (`_sqlx_migrations`) adds confusion and violates the existing convention. A custom runner is simpler, self-documenting, and has zero additional dependency cost.

### Decision: Numeric prefix ordering

**Choice**: Three-digit numeric prefix (`001_`, `002_`, etc.). Non-numeric files skipped with `tracing::warn!`.
**Alternatives**: Timestamp prefix (`20250603_`), explicit ordering file. **Rationale**: Numeric prefix is the lowest ceremony — everyone understands it, it's human-readable, and it matches the existing files (`001_init.sql`, `002_add_url_validation.sql`).

### Decision: Transaction-per-migration

**Choice**: Each migration runs in its own `sqlx::Transaction`. If it fails, that transaction is rolled back, `db_version` is NOT updated, and the app fails to start.
**Rationale**: A failed partial migration must not leave the DB in an unknown state. Rolling back the single migration is safe because earlier migrations already committed — they're known-good. The app won't start until the bug is fixed.

## Components

```
src-tauri/src/repository/sqlite/
├── mod.rs                    [MODIFIED] — add run_migrations() to init
└── migrations/
    ├── mod.rs                [CREATED] — MigrationRunner struct + impl
    ├── 001_init.sql
    └── 002_add_url_validation.sql
```

```rust
// migrations/mod.rs — public API
pub struct MigrationRunner {
    pool: sqlx::SqlitePool,
    dir: PathBuf,
}

pub struct DiscoveredMigration {
    pub version: u32,
    pub filename: String,
    pub path: PathBuf,
}

#[derive(Debug, thiserror::Error)]
pub enum MigrationError {
    #[error("gap in migration sequence: expected v{expected}, found v{found}")]
    GapInSequence { expected: u32, found: u32 },
    #[error("invalid schema_meta.db_version value: {0}")]
    InvalidVersion(String),
    #[error("migration {filename} failed: {source}")]
    SqlError { filename: String, source: sqlx::Error },
}
```

## Data Flow

```
App::run()
  │
  ├─ SqlitePool::connect("sqlite:guitarhub.db?mode=rwc")
  │
  └─ MigrationRunner::new(pool, "migrations/").run()
       │
       ├─ 1. glob migrations/*.sql
       │     regex `^(\d{3})_` on filenames → sort by version
       │
       ├─ 2. read schema_meta WHERE key = 'db_version'
       │     missing table/key → version = 0
       │     corrupt value     → MigrationError::InvalidVersion
       │
       ├─ 3. filter: unapplied = discovered[current_version..]
       │
       ├─ 4. validate: unapplied versions MUST be consecutive
       │     (e.g., v1 applied, v3 exists but v2 missing → err)
       │
       └─ 5. for each migration in sequence:
             BEGIN TRANSACTION
             execute SQL from file
             if error → ROLLBACK, return MigrationError::SqlError
             UPDATE schema_meta SET value = version WHERE key = 'db_version'
             COMMIT
             tracing::info!("Applied migration {filename}")
```

## Error Handling

| Failure | Behaviour | Recovery |
|---------|-----------|----------|
| Migrations dir missing | Log warning, skip (fresh tests) | App starts with v0 |
| Non-numeric filename | `tracing::warn!` skip file | App continues |
| Gap in sequence | `MigrationError::GapInSequence`, app aborts | Fix migration files, rebuild |
| SQL execution error | Transaction rolled back, app aborts | Fix SQL, rebuild |
| Corrupt `db_version` | `MigrationError::InvalidVersion`, app aborts | Manual fix or reinstall |

## Testing Strategy

| Layer | What | Approach |
|-------|------|----------|
| Unit | File discovery | Temp dir with `001_a.sql`, `002_b.sql`, `nope.sql` → verify ordering + skip |
| Unit | Version parsing | `schema_meta` absent → 0; `abc` → error |
| Unit | Gap detection | v0 with only `003.sql` present → GapInSequence |
| Integration | Full apply | In-memory DB, run migrations → verify tables + `db_version` |
| Integration | Idempotent | Run twice → second run executes zero SQL |
| Integration | Failure isolation | Inject syntax error in `002` → v1 unchanged, `002` rolled back |

## Implementation Order

1. Create `src-tauri/src/repository/sqlite/migrations/mod.rs` with `MigrationRunner` struct
2. Implement `discover()`, `current_version()`, `apply_pending()`
3. Add `run()` method that orchestrates the flow
4. Wire `runner.run()` into app initialization in `lib.rs` (before services)
5. Write unit + integration tests
6. Verify against existing `001_init.sql` + `002_add_url_validation.sql`
