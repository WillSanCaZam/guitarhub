.PHONY: setup dev build test test-app test-scraper test-e2e lint lint-rust lint-py lint-frontend clean audit help

CARGO = cd src-tauri && cargo

# Dirs for scraper operations
SCRAPER_DIR = scraper

# ── Setup ────────────────────────────────────────────────────────────────────

## Install all dependencies, hooks, and build the Rust project
setup:
	pip install -r $(SCRAPER_DIR)/requirements.txt 2>/dev/null || echo "No scraper/requirements.txt found"
	pre-commit install
	$(CARGO) build

# ── Development ──────────────────────────────────────────────────────────────

## Start the Tauri dev server with hot reload
dev:
	cargo tauri dev

# ── Build ────────────────────────────────────────────────────────────────────

## Build the Rust project in release mode
build:
	$(CARGO) build --release

# ── Test ─────────────────────────────────────────────────────────────────────

## Run all tests (Rust + Python + Frontend)
test: test-app test-scraper test-frontend test-e2e

## Run only frontend tests
## Run only frontend tests
test-frontend:
	npm run test

## Run only Rust tests
test-app:
	$(CARGO) test

## Run only Python tests (unit + contract)
test-scraper:
	@if [ -d "$(SCRAPER_DIR)" ]; then \
		if [ -d ".venv" ]; then \
			source .venv/bin/activate && python -m pytest scraper/tests/ -v; \
		else \
			python3 -m pytest scraper/tests/ -v; \
		fi; \
	else \
		echo "No $(SCRAPER_DIR)/ directory found — skipping Python tests"; \
	fi

## Run E2E tests (requires cargo tauri build --debug --no-bundle first)
test-e2e:
	@if command -v tauri-driver >/dev/null 2>&1 && [ -f "./src-tauri/target/debug/guitarhub" ]; then \
		npm run test:e2e; \
	else \
		echo "tauri-driver or debug binary not found — skipping E2E tests. Run: cargo install tauri-driver && cargo tauri build --debug --no-bundle"; \
	fi

# ── Lint ─────────────────────────────────────────────────────────────────────

## Run all linters (Rust + Python + Frontend)
lint: lint-rust lint-py lint-frontend

## Run only Rust linters (clippy)
lint-rust:
	$(CARGO) clippy --all-targets -- -D warnings

## Run only Python linters (ruff + mypy)
lint-py:
	@if [ -d "$(SCRAPER_DIR)" ]; then \
		if command -v ruff >/dev/null 2>&1; then \
			ruff check scraper/; \
		else \
			echo "ruff not installed — skipping ruff check"; \
		fi; \
		if command -v mypy >/dev/null 2>&1; then \
			mypy scraper/ --strict; \
		else \
			echo "mypy not installed — skipping mypy check"; \
		fi; \
	else \
		echo "No $(SCRAPER_DIR)/ directory found — skipping Python linting"; \
	fi

## Run only frontend linter (svelte-check)
lint-frontend:
	npx svelte-check --tsconfig ./tsconfig.json

# ── Clean ────────────────────────────────────────────────────────────────────

## Remove build artifacts and cache files
clean:
	$(CARGO) clean
	@if [ -d "$(SCRAPER_DIR)" ]; then \
		find "$(SCRAPER_DIR)" -type d -name __pycache__ -exec rm -rf {} + 2>/dev/null; \
	fi; true

# ── Audit ────────────────────────────────────────────────────────────────────

## Run security audits (cargo-audit + pip-audit)
audit:
	$(CARGO) audit
	@if [ -f "$(SCRAPER_DIR)/requirements.txt" ]; then \
		pip-audit -r "$(SCRAPER_DIR)/requirements.txt" --desc on; \
	else \
		echo "No $(SCRAPER_DIR)/requirements.txt found — skipping pip-audit"; \
	fi

# ── Scrape Targets ──────────────────────────────────────────────────────────

## Ensure artifacts directory exists
artifacts:
	mkdir -p artifacts

## Scrape Reverb catalog and save to artifacts/catalog.json
scrape-reverb: artifacts
	@if [ -d "$(SCRAPER_DIR)" ]; then \
		if [ -d ".venv" ]; then \
			source .venv/bin/activate && python -m scraper --adapter reverb --output artifacts/catalog.json; \
		else \
			python3 -m scraper --adapter reverb --output artifacts/catalog.json; \
		fi; \
		echo "Scraped to artifacts/catalog.json"; \
	else \
		echo "No $(SCRAPER_DIR)/ directory found — skipping"; \
	fi

## Scrape Guitar Center catalog (requires GC_ALGOLIA_* env vars)
scrape-guitarcenter:
	@if [ -d "$(SCRAPER_DIR)" ]; then \
		if [ -d ".venv" ]; then \
			source .venv/bin/activate && python -m scraper --adapter guitarcenter --output artifacts/catalog-gc.json; \
		else \
			python3 -m scraper --adapter guitarcenter --output artifacts/catalog-gc.json; \
		fi; \
	else \
		echo "No $(SCRAPER_DIR)/ directory found — skipping"; \
	fi

## Scrape + Dev in one step: populate data, then start the app
dev-with-data: scrape-reverb
	GUITARHUB_AUTO_IMPORT=artifacts/catalog.json cargo tauri dev

# ── Help ─────────────────────────────────────────────────────────────────────

## Show this help message
help:
	@echo "GuitarHub — Development Makefile"
	@echo ""
	@echo "Usage:"
	@echo "  make <target>"
	@echo ""
	@echo "Targets:"
	@grep -Eh '^[a-z_-]+:.*?##' $(MAKEFILE_LIST) | sort | \
		awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[36m%-20s\033[0m %s\n", $$1, $$2}'
