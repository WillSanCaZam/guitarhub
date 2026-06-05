# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- **SyncService** — catalog synchronization with HTTP fetch, state machine (idle → downloading → validating → sanitizing → inserting → done/failed), and SQLite upsert
- **SearchService** — FTS5 full-text search with sanitized MATCH queries, filters (category, price range, source), sort orders (relevance, price asc/desc, date), and pagination (limit/offset)
- **Frontend scaffold** — Svelte 5 pages with search bar, ProductCard grid, loading/empty/error states, sync button with progress indicator, and "Load more" pagination
- **Python scraper** — ports-and-adapters scraper package with Reverb adapter (HTTP retry+backoff, BeautifulSoup HTML extraction), Pydantic domain models, and CLI entry point
- **CI workflows** — Rust CI (`cargo test`, clippy, formatting) and Python CI (pytest, mypy, pip-audit)
- **Scraper scheduled workflow** — GitHub Actions cron every 6 hours with pip-audit gate and schema validation
- **License, changelog, and README** — GPL-3.0 license, Keep a Changelog format, and project documentation
- **SPDX headers** — GPL-3.0-or-later license identifiers on all new source files

### Changed

- SDD architecture artifacts updated for all phases (proposal, specs, design, tasks)

### Fixed

- (none)
