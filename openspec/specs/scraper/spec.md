# Scraper Specification

> **Status**: New capability  
> **Change**: mvp-completion

## Purpose

Provide a Python-based catalog scraper following Ports & Adapters architecture. The initial adapter targets Reverb.com. Output JSON SHALL match the `CatalogFile` schema consumed by SyncService. A GitHub Actions workflow SHALL run the scraper every 6 hours with security and validation gates.

## Requirements

### Requirement: Scraper SHALL output CatalogFile-compatible JSON

The scraper MUST produce a JSON file matching the `CatalogFile` schema: `{ schema_version: String, source_id: String, generated_at: DateTime, run_id: String, products: Vec<CatalogProduct> }`. Each `CatalogProduct` MUST include `sku`, `name`, `brand`, `price`, `currency`, `image_url`, `product_url`, `category`, `condition`, and `source`.

| Case | Precondition | Action | Outcome |
|------|-------------|--------|---------|
| Valid output | Scraper runs against Reverb | Generate JSON | Output validates against `CatalogFile` schema |
| Empty catalog | Reverb returns no listings | Scraper runs | Output has `products: []`, still valid JSON |
| Field mapping | Reverb listing has all fields | Scraper processes it | All `CatalogProduct` fields populated correctly |

### Requirement: Scraper MUST use Ports & Adapters pattern

The `scraper/` package SHALL define a `ScraperPort` protocol/ABC. `ReverbAdapter` SHALL implement it. Adapters SHALL be swappable via dependency injection.

| Case | Precondition | Action | Outcome |
|------|-------------|--------|---------|
| Adapter contract enforced | `ScraperPort` defined | Instantiate without `scrape()` | TypeError on missing method |
| Reverb adapter injectable | `ReverbAdapter(ScraperPort)` | Pass to scraper CLI | Runs without import errors |
| New adapter possible | `ShopifyAdapter(ScraperPort)` | Implement interface | Works without changing core code |

### Requirement: Scraper MUST handle HTTP errors gracefully

The adapter SHALL retry on transient HTTP errors (5xx, timeout) with exponential backoff (3 retries). Permanent errors (4xx) SHALL fail immediately with a descriptive error.

| Case | Precondition | Action | Outcome |
|------|-------------|--------|---------|
| 503 transient error | Server returns 503 | First attempt fails | Retries up to 3 times with backoff |
| 404 permanent error | Listing page not found | Adapter fetches URL | Fails immediately, descriptive error |
| Timeout | Server hangs >30s | HTTP request times out | Retries with backoff, then fails |

### Requirement: Scraper MUST run via GitHub Actions cron

`.github/workflows/scrape.yml` SHALL schedule every 6 hours via `cron: '0 */6 * * *'`. It SHALL run `pip-audit` before the scraper for dependency security, and SHALL use `--validate-input` (or equivalent schema validation) before publishing output.

| Case | Precondition | Action | Outcome |
|------|-------------|--------|---------|
| Cron trigger | 6h elapsed | GitHub Actions runs | Workflow starts, checks out repo |
| Security gate | Dependency has known vuln | `pip-audit` runs | Workflow fails before scraper executes |
| Validation gate | Output JSON invalid | Post-scraper validation | Workflow fails, output not published |
| Manual trigger | Workflow dispatch | `workflow_dispatch` event | Scraper runs on demand |

### Requirement: Scraper SHALL support CLI invocation

The scraper SHALL provide a CLI entry point: `scraper --adapter reverb --output catalog.json`. It SHALL accept `--adapter` (select adapter), `--output` (output path), and `--validate` (validate output before exit).

| Case | Precondition | Action | Outcome |
|------|-------------|--------|---------|
| CLI runs | `scraper` installed | `scraper --adapter reverb --output out.json` | Generates `out.json`, exit 0 |
| Validation flag | `--validate` passed | Output validated after scrape | Exit 1 if schema invalid |
