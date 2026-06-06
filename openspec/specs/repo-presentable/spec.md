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

The `[Unreleased]` section MUST be replaced with a dated `v0.1.0` release section. The `v0.1.0` section MUST contain the release date in ISO format (`YYYY-MM-DD`). The `v0.1.0` section MUST include a summary of MVP features and a link to the full commit history or comparison URL.

(Previously: The CHANGELOG contained only an `[Unreleased]` section with MVP features. No dated release section existed.)

| Case | Precondition | Action | Outcome |
|------|-------------|--------|---------|
| Valid format | CHANGELOG.md exists | Render on GitHub | Follows Keep a Changelog structure |
| Unreleased section | Features implemented | Read CHANGELOG | Lists SyncService, SearchService, Scraper under Added |
| v0.1.0 section with date | CHANGELOG.md is opened | Read release sections | `## [0.1.0] - YYYY-MM-DD` section is present and appears before any `[Unreleased]` section |
| v0.1.0 feature summary | v0.1.0 section is read | Inspect content | `Added` lists MVP features; `Changed` lists behavior modifications; `Fixed` lists bug fixes |
| Unreleased preserved | v0.1.0 section is added | Read CHANGELOG | Empty `## [Unreleased]` section exists after `v0.1.0` |
| Comparison link | v0.1.0 section is added | Read bottom of CHANGELOG | Link definition `[0.1.0]: https://github.com/william/guitarhub/compare/v0.0.0...v0.1.0` or equivalent is present |

---

### Requirement: CHANGELOG v0.1.0 MUST credit contributors

The `v0.1.0` section SHOULD include a `Contributors` or `Thanks` line listing the main contributors to the release.

| Case | Precondition | Action | Outcome |
|------|-------------|--------|---------|
| Contributors listed | v0.1.0 section is read | Inspect bottom of section | A contributors line is present naming the primary authors |

---

### Requirement: README.md MUST reflect v0.1.0 status

The `README.md` MUST be updated to indicate the current release is `v0.1.0`. The README SHOULD include a badge or line showing the latest release version.

| Case | Precondition | Action | Outcome |
|------|-------------|--------|---------|
| Version badge in README | README.md is rendered on GitHub | View project header | A version badge or line indicates "v0.1.0" and links to the GitHub releases page |

### Requirement: GPL-3.0 header MUST appear in source files

All Rust and Python source files SHALL contain a GPL-3.0 license header comment: `// SPDX-License-Identifier: GPL-3.0-or-later` (Rust) or `# SPDX-License-Identifier: GPL-3.0-or-later` (Python).

| Case | Precondition | Action | Outcome |
|------|-------------|--------|---------|
| New Rust files | `sync.rs`, `search.rs` created | `head -5 src-tauri/.../sync.rs` | Contains SPDX header |
| New Python files | `reverb_adapter.py` created | `head -3 scraper/.../reverb_adapter.py` | Contains SPDX header |
