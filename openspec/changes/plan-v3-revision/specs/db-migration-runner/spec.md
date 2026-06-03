# DB Migration Runner Specification

## Purpose

Schema migration discovery, tracking, and application for the app's local SQLite database. Guarantees the on-disk schema matches the code the app was built with.

## Requirements

### Requirement: Discover `.sql` migration files

The system MUST scan a designated `migrations/` directory at startup and collect all `.sql` files sorted by numerical prefix (e.g., `001_init.sql`, `002_add_index.sql`).

#### Scenarios

| Case | Precondition | Action | Outcome |
|------|-------------|--------|---------|
| Happy | Directory has `001_init.sql`, `002_add_index.sql` | Scan | Returns `[001_init.sql, 002_add_index.sql]` in order |
| Empty dir | No files in `migrations/` | Scan | Returns empty list, no error |
| Non-numeric prefix | File `setup.sql` exists | Scan | Skips file, logs warning |

### Requirement: Track applied state in `schema_meta`

The system MUST read `db_version` from `schema_meta` to determine current schema version. Missing table or key MUST be treated as version `0`.

#### Scenarios

| Case | Precondition | Action | Outcome |
|------|-------------|--------|---------|
| Happy | `schema_meta` has `db_version = '2'` | Read version | Returns `2` |
| Fresh DB | No `schema_meta` table | Read version | Assumes `0`, applies all migrations |
| Corrupt value | `db_version = 'abc'` | Read version | Returns error, app must not start |

### Requirement: Apply unapplied migrations in order

The system MUST execute each unapplied `.sql` file sequentially. If migration `001` is applied but `003` exists without `002`, the runner MUST fail.

#### Scenarios

| Case | Precondition | Action | Outcome |
|------|-------------|--------|---------|
| Fresh DB | Version `0`, migrations `001` and `002` exist | Apply | Both run, `db_version` set to `2` |
| Incremental | Version `1`, `002_add_index.sql` exists | Apply | Only `002` runs, `db_version` updated to `2` |
| Up-to-date | Version `2`, all migrations present | Check | No SQL executed, no error |
| Gap in sequence | Version `0`, only `002` exists | Apply | Error returned, no partial apply |
| SQL failure | `001_init.sql` has syntax error | Execute | Error returned, DB unchanged |

### Requirement: Log migration activity

The system MUST log each applied migration. The system MAY use the `tracing` crate.

#### Scenario: Success logged

- GIVEN migration `002_add_index.sql` is applied
- WHEN it succeeds
- THEN a `tracing::info!` records `"Applied migration 002_add_index.sql"`

## Acceptance Criteria

| Criterion | How to verify |
|-----------|---------------|
| Fresh DB gets all migrations | Start with empty DB — all `.sql` applied, `db_version` matches count |
| Upgrade only new migrations | Set `db_version=1`, start — only `002` runs |
| Idle is no-op | Start up-to-date — zero SQL executed |
| Bad SQL fails cleanly | Inject `CREAT TABLE` — error, schema unchanged |
| Missing sequence fails | Remove `001`, start — error, no partial apply |

## Out of Scope

- Migration rollback or undo
- Python-side migrations (scraper has no local SQLite)
- Data migrations (DDL only)
- Visual migration progress UI
