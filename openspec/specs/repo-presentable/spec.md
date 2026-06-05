# Repo Presentable Specification

> **Status**: New capability  
> **Change**: mvp-completion

## Purpose

Establish project identity and onboarding surface: README.md with build instructions and screenshot placeholders, LICENSE under GPL-3.0, and an initial CHANGELOG.md entry.

## Requirements

### Requirement: README.md MUST describe project and build

The system MUST provide `README.md` at the repo root containing: project name and description ("GuitarHub — electric guitar price tracker"), prerequisites (Rust, Node.js, Python), build instructions (`make build`), development setup (`make dev`), screenshot placeholders (at least one of the app), tech stack summary, and testing instructions (`make test`).

| Case | Precondition | Action | Outcome |
|------|-------------|--------|---------|
| Readable on GitHub | README.md exists at root | Visit repo on GitHub | Renders as project homepage |
| Build from scratch | Clean clone, all deps absent | Follow README instructions | `make build` compiles Tauri binary |
| Screenshot placeholders | No actual screenshot yet | README rendered | `![App Screenshot](docs/screenshot.png)` with alt text |
| Tech stack listed | All components mentioned | README read | Rust (Tauri), Svelte 5, SQLite (FTS5), Python listed |

### Requirement: LICENSE MUST be GPL-3.0

The system MUST provide `LICENSE` at the repo root containing the full GPL-3.0 license text as published by the Free Software Foundation.

| Case | Precondition | Action | Outcome |
|------|-------------|--------|---------|
| Detectable by GitHub | LICENSE with "GPL-3.0" in name | Push to GitHub | GitHub shows "GPL-3.0" license badge |
| Full legal text | GPL-3.0 preamble and terms | Open LICENSE | Contains GPL-3.0 copyright and permissions notice |

### Requirement: CHANGELOG.md MUST exist with initial entry

The system MUST provide `CHANGELOG.md` at the repo root in [Keep a Changelog](https://keepachangelog.com/) format. The `[Unreleased]` section SHALL include the MVP completion features grouped by `Added`, `Changed`, and `Fixed`.

| Case | Precondition | Action | Outcome |
|------|-------------|--------|---------|
| Valid format | CHANGELOG.md exists | Render on GitHub | Follows Keep a Changelog structure |
| Unreleased section | Features implemented | Read CHANGELOG | Lists SyncService, SearchService, Scraper under Added |

### Requirement: GPL-3.0 header MUST appear in source files

All Rust and Python source files SHALL contain a GPL-3.0 license header comment: `// SPDX-License-Identifier: GPL-3.0-or-later` (Rust) or `# SPDX-License-Identifier: GPL-3.0-or-later` (Python).

| Case | Precondition | Action | Outcome |
|------|-------------|--------|---------|
| New Rust files | `sync.rs`, `search.rs` created | `head -5 src-tauri/.../sync.rs` | Contains SPDX header |
| New Python files | `reverb_adapter.py` created | `head -3 scraper/.../reverb_adapter.py` | Contains SPDX header |
