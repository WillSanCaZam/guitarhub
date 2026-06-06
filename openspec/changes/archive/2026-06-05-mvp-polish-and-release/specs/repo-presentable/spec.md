# Delta for repo-presentable

## MODIFIED Requirements

### Requirement: CHANGELOG.md MUST exist with initial entry

The system MUST provide `CHANGELOG.md` at the repo root in [Keep a Changelog](https://keepachangelog.com/) format. The `[Unreleased]` section SHALL include the MVP completion features grouped by `Added`, `Changed`, and `Fixed`.

The `[Unreleased]` section MUST be replaced with a dated `v0.1.0` release section. The `v0.1.0` section MUST contain the release date in ISO format (`YYYY-MM-DD`). The `v0.1.0` section MUST include a summary of MVP features and a link to the full commit history or comparison URL.

(Previously: The CHANGELOG contained only an `[Unreleased]` section with MVP features. No dated release section existed.)

#### Scenario: v0.1.0 section with date

- GIVEN `CHANGELOG.md` is opened
- WHEN reading the release sections
- THEN a `## [0.1.0] - YYYY-MM-DD` section is present (where YYYY-MM-DD is the release date)
- AND the section appears before any `[Unreleased]` section

#### Scenario: v0.1.0 feature summary

- GIVEN the `v0.1.0` section is read
- WHEN inspecting the content
- THEN the `Added` subsection lists MVP features: SyncService, SearchService, Scraper, Dashboard, Collection, Wishlist, Price Insights
- AND the `Changed` subsection lists any behavior modifications
- AND the `Fixed` subsection lists any bug fixes

#### Scenario: Unreleased section is preserved or empty

- GIVEN the `v0.1.0` section is added
- WHEN reading the CHANGELOG
- THEN an empty `## [Unreleased]` section exists after `v0.1.0`
- OR the `[Unreleased]` section is present and ready for post-release changes

#### Scenario: Comparison link

- GIVEN the `v0.1.0` section is added
- WHEN reading the bottom of the CHANGELOG
- THEN a link definition `[0.1.0]: https://github.com/william/guitarhub/compare/v0.0.0...v0.1.0` or equivalent is present

## ADDED Requirements

### Requirement: CHANGELOG v0.1.0 MUST credit contributors

The `v0.1.0` section SHOULD include a `Contributors` or `Thanks` line listing the main contributors to the release.

#### Scenario: Contributors listed

- GIVEN the `v0.1.0` section is read
- WHEN inspecting the bottom of the section
- THEN a contributors line is present
- AND it names the primary authors of the release

---

### Requirement: README.md MUST reflect v0.1.0 status

The `README.md` MUST be updated to indicate the current release is `v0.1.0`. The README SHOULD include a badge or line showing the latest release version.

#### Scenario: Version badge in README

- GIVEN `README.md` is rendered on GitHub
- WHEN viewing the project header
- THEN a version badge or line indicates "v0.1.0"
- AND the badge links to the GitHub releases page

---

### Requirement: LICENSE MUST be GPL-3.0

The system MUST provide `LICENSE` at the repo root containing the full GPL-3.0 license text as published by the Free Software Foundation.

#### Scenario: LICENSE detectable

- GIVEN `LICENSE` exists at the repo root
- WHEN pushed to GitHub
- THEN GitHub shows "GPL-3.0" in the license badge

#### Scenario: Full legal text

- GIVEN `LICENSE` is opened
- WHEN reading the file
- THEN it contains the GPL-3.0 preamble and terms
- AND it contains the copyright and permissions notice

---

### Requirement: GPL-3.0 header MUST appear in source files

All Rust and Python source files SHALL contain a GPL-3.0 license header comment: `// SPDX-License-Identifier: GPL-3.0-or-later` (Rust) or `# SPDX-License-Identifier: GPL-3.0-or-later` (Python).

#### Scenario: Rust files have SPDX header

- GIVEN any `.rs` file in `src-tauri/`
- WHEN reading the first 5 lines
- THEN the SPDX header is present

#### Scenario: Python files have SPDX header

- GIVEN any `.py` file in `scraper/`
- WHEN reading the first 3 lines
- THEN the SPDX header is present
