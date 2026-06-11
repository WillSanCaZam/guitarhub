# Delta for wu4-repo-hygiene

> **Change**: ci-pipeline-fix-all-issues — Add Python entries to .gitignore

## MODIFIED Requirements

### Requirement: Default ignore patterns

`.gitignore` MUST ignore `target/`, `.env`, `__pycache__/`, `*.pyc`, `node_modules/`, and `.DS_Store` at minimum. Additionally, `.gitignore` MUST ignore Python virtualenv and tooling directories: `.venv/`, `*.egg-info/`, `.mypy_cache/`, `.pytest_cache/`, `.ruff_cache/`.

(Previously: `.gitignore` only covered Rust, Node, and basic Python cache patterns; virtualenv and Python tooling directories were missing.)

#### Scenarios

| Case | Precondition | Action | Outcome |
|------|-------------|--------|---------|
| Rust build artifact | `target/debug/guitarhub` exists | `git status` | `target/` NOT shown as untracked |
| Environment file | `.env` with API keys present | `git status` | `.env` NOT shown as untracked |
| Python cache directory | `__pycache__/` exists in any dir | `git status` | `__pycache__/` NOT shown as untracked |
| Python compiled file | `*.pyc` file exists | `git status` | `*.pyc` NOT shown as untracked |
| Node modules | `node_modules/` exists | `git status` | `node_modules/` NOT shown as untracked |
| macOS metadata | `.DS_Store` exists | `git status` | `.DS_Store` NOT shown as untracked |
| Python virtualenv | `.venv/` directory exists | `git status` | `.venv/` NOT shown as untracked |
| Egg info | `scraper.egg-info/` exists | `git status` | `*.egg-info/` NOT shown as untracked |
| Mypy cache | `.mypy_cache/` exists | `git status` | `.mypy_cache/` NOT shown as untracked |
| Pytest cache | `.pytest_cache/` exists | `git status` | `.pytest_cache/` NOT shown as untracked |
| Ruff cache | `.ruff_cache/` exists | `git status` | `.ruff_cache/` NOT shown as untracked |
