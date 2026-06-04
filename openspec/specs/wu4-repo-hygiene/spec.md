# Repo Hygiene Specification

## Purpose

Prevent accidental commits of build artifacts, secrets, and generated files via `.gitignore`. Verify existing migration integrity for FTS5 trigger column references.

## Requirements

### Requirement: Default ignore patterns

`.gitignore` MUST ignore `target/`, `.env`, `__pycache__/`, `*.pyc`, `node_modules/`, and `.DS_Store` at minimum.

#### Scenarios

| Case | Precondition | Action | Outcome |
|------|-------------|--------|---------|
| Rust build artifact | `target/debug/guitarhub` exists | `git status` | `target/` NOT shown as untracked |
| Environment file | `.env` with API keys present | `git status` | `.env` NOT shown as untracked |
| Python cache directory | `__pycache__/` exists in any dir | `git status` | `__pycache__/` NOT shown as untracked |
| Python compiled file | `*.pyc` file exists | `git status` | `*.pyc` NOT shown as untracked |
| Node modules | `node_modules/` exists | `git status` | `node_modules/` NOT shown as untracked |
| macOS metadata | `.DS_Store` exists | `git status` | `.DS_Store` NOT shown as untracked |

### Requirement: FTS5 trigger column correctness

`001_init.sql` MUST have FTS5 triggers that reference real column names from the source table (`new.name`, `new.brand`, `new.model`, etc.). Triggers MUST NOT reference synthetic or non-existent columns.

#### Scenarios

| Case | Precondition | Action | Outcome |
|------|-------------|--------|---------|
| Insert trigger fires | New row inserted into source table | `AFTER INSERT` trigger executes | Trigger reads `new.name`, `new.brand` — valid columns |
| Delete trigger fires | Row deleted from source table | `AFTER DELETE` trigger executes | Trigger reads `old.name`, `old.brand` — valid columns |
| Update trigger fires | Row updated in source table | `AFTER UPDATE` trigger executes | Trigger reads `new.name`, `old.name` — qualified correctly |

## Acceptance Criteria

| Criterion | How to verify |
|-----------|---------------|
| `.gitignore` effective | Create `target/test.tmp` → `git status` shows no untracked for that path |
| FTS5 triggers reference valid columns | Review `001_init.sql`: every `new.` and `old.` reference matches a `CREATE TABLE` column |
| No build artifacts tracked | `git ls-files | grep 'target/'` — returns empty |
