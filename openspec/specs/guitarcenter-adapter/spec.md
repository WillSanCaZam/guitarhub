# Guitar Center Adapter Specification

## Purpose

`GuitarCenterAdapter` implements `ScraperPort` to sync Guitar Center's used gear catalog via Algolia API. Maps Algolia fields to `CatalogProduct`, normalizes GC condition vocabulary, signals availability — orthogonal to delisting.

## Requirements

### Requirement: Adapter SHALL implement ScraperPort

`GuitarCenterAdapter` MUST implement `scrape(url: str = "") -> CatalogFile`. Credentials from constructor args or env vars `GC_ALGOLIA_*`.

| Case | Precondition | Action | Outcome |
|------|-------------|--------|---------|
| Valid adapter | Env vars set | Instantiate and `scrape()` | Returns valid `CatalogFile` |
| Missing creds | No args, no env vars | `GuitarCenterAdapter()` | Raises `ValueError` with var names |

### Requirement: Adapter SHALL paginate via Algolia page param

Fetches pages with configurable delay. Stops when hits empty or `max_pages` reached.

| Case | Precondition | Action | Outcome |
|------|-------------|--------|---------|
| Multiple pages | 48 results across 2 pages | Pages 0-1 fetched | All 48 products collected |
| Empty catalog | Algolia returns `hits: []` | Scrape completes | `products: []`, valid JSON |
| Max pages | 1000 results, `max_pages=3` | Stops at page 2 | 72 items across 3 pages |
| Rate limit | `delay=1.0` | Paginated scrape | ≥1s between successive requests |

### Requirement: Adapter SHALL map Algolia fields to CatalogProduct

Maps per design field table. Prefixes SKU with `gc-`, constructs image URLs from `imageId`, derives category from hierarchical facets.

| Case | Precondition | Action | Outcome |
|------|-------------|--------|---------|
| Complete hit | All fields present | Map to `CatalogProduct` | All fields populated |
| Missing brand | `brand` absent | Default | `brand = "Unknown"` |
| Missing image | `imageId` absent | Default | `image_url = ""` |
| SKU prefix | `identifiers.gcItemNumber = "12345"` | Map | `sku = "gc-12345"` |

### Requirement: Adapter SHALL normalize GC conditions to 4-value vocabulary

Maps GC conditions (`Excellent`, `Great`, `Good`, `Fair`, `Poor`, `New`, `Open Box`, `Blemished`, `Restock`) to (`new`, `used`, `refurbished`, `unknown`). Raw value in `specs_json`.

| Case | Precondition | Action | Outcome |
|------|-------------|--------|---------|
| Used variants | `condition.lvl1 = "Used > Excellent"` | Normalize | `condition = "used"`, original in `specs_json` |
| New | `condition.lvl0 = "New"` | Normalize | `condition = "new"` |
| Open Box | `skuCondition = 3` | Normalize | `condition = "new"`, sticker `"open_box"` |
| Blemished | `skuCondition = 11` | Normalize | `condition = "new"`, sticker `"blemished"` |
| Restock | `skuCondition = 2` | Normalize | `condition = "refurbished"`, sticker `"restock"` |
| Unknown | No condition data | Normalize | `condition = "unknown"` |

### Requirement: Adapter SHALL set availability from Algolia stock signals

Availability from `inventoryStatus` + `stores[]`. Both in (1000, 1003) AND stores non-empty → `in_stock`. Otherwise → `out_of_stock`.

| Case | Precondition | Action | Outcome |
|------|-------------|--------|---------|
| In stock | InvStatus=1000, stores non-empty | Map | `availability = "in_stock"` |
| Out of stock | InvStatus=1003, `stores=[]` | Map | `availability = "out_of_stock"` |
| No stores | `stores=[]`, any InvStatus | Map | `availability = "out_of_stock"` |
| No inventory | `inventoryStatus` absent | Map | `availability = "out_of_stock"` |

### Requirement: Adapter SHALL handle errors per ScraperPort contract

Raises `FetchError` on HTTP failures, `ParseError` on bad JSON.

| Case | Precondition | Action | Outcome |
|------|-------------|--------|---------|
| HTTP error | Algolia returns 4xx/5xx | Fetch fails | Raises `FetchError` |
| Timeout | Connection hangs >30s | Times out | Raises `FetchError` |
| Invalid JSON | Malformed response | Parse fails | Raises `ParseError` |

### Requirement: CLI SHALL support --adapter guitarcenter

CLI MUST accept `"guitarcenter"` in `--adapter` choices.

| Case | Precondition | Action | Outcome |
|------|-------------|--------|---------|
| CLI invocation | Env vars set | `--adapter guitarcenter --output catalog-gc.json` | exit 0 |
| Invalid name | Typo in adapter | `--adapter guitarcentr` | Exit with usage error |
