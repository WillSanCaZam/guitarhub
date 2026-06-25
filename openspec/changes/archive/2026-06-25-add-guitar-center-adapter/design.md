# Design: Guitar Center Scraper Adapter

## Technical Approach

Add a new `GuitarCenterAdapter` implementing the existing `ScraperPort` protocol in `scraper/ports.py`. Unlike the Reverb adapter — which consumes a documented JSON API — Guitar Center uses **Algolia** as its client-side search backend. The adapter will talk directly to the Algolia REST endpoint (`*.algolia.net`) with publicly-exposed credentials, NOT to `guitarcenter.com` HTML pages (which are behind Cloudflare).

This is the same approach used by [`gcgeartracker.com`](https://gcgeartracker.com/) and [`cbuggelli/guitar-inventory-tracker`](https://github.com/cbuggelli/guitar-inventory-tracker) — both proven in production. No headless browser, no Cloudflare bypass needed.

The adapter follows the exact same Ports & Adapters pattern as `ReverbAdapter`: sync HTTP via `requests`, pagination via offset param, field mapping to `CatalogProduct`, output via `CatalogFile.create()`.

## Architecture Decisions

| Decision | Option | Tradeoff | Choice |
|----------|--------|----------|--------|
| Data source | Guitar Center HTML pages vs Algolia API | HTML requires Cloudflare bypass + headless browser; Algolia is a direct JSON API with simple auth headers | **Algolia API** — proven approach, no new infra, `requests`-only |
| Credential strategy | Hardcoded constants vs env vars | Hardcoded is simpler but breaks if keys rotate; env vars require documented re-extraction steps | **Env vars** (`GC_ALGOLIA_APP_ID`, `GC_ALGOLIA_API_KEY`) — documented in `.env.example` and `docs/` |
| Condition normalization | At adapter level vs rely on Rust `sanitize()` | Adapter-level ensures `CatalogFile` JSON already has normalized values (matching CHECK constraint); Rust `sanitize()` only lowercases | **Adapter-level** — map GC conditions to the 4-value vocabulary (`new`, `used`, `refurbished`, `unknown`) before output, keeping `stickers` in `specs_json` for lossless preservation |
| Rate limiting | Per-adapter sleep vs global throttle in sync_state | Per-adapter is simpler; global is more realistic once we have 3+ sources | **Per-adapter delay** (`delay` param, matching `ReverbAdapter`), configurable via constructor arg |
| Pagination | Offset-based (Algolia `page` param) vs cursor-based | Offset is what Algolia exposes; cursor-based would require an adapter-level abstraction | **Offset-based** — matches Algolia's native pagination, same pattern as Reverb's page param |
| Soft-delete handling | Adapter sets `availability` (stock state) vs adapter invents delisting signal | `availability` describes whether an item is in stock right now — NOT whether it still exists in the source catalog. Conflating them breaks the column contract and couples the adapter to a migration that does not exist yet. | **Out of scope for this adapter.** The adapter MUST set `availability` based on whatever stock signal Algolia returns (TBD during Algolia query). Soft-delete (`is_active`, delisting detection) is owned by the sync layer's diff logic in the pending `add-soft-delete-products` change, which must merge BEFORE this adapter. |

## Data Flow

```
┌──────────────┐     ┌──────────────────────────────────┐     ┌─────────────────────┐
│  GHA Cron    │ ──► │  adapter: GuitarCenterAdapter    │ ──► │  catalog-gc.json    │
│  scrape.yml  │     │  POST *.algolia.net/1/indexes/   │     │  (CatalogFile)      │
└──────────────┘     │  queries?x-algolia-agent=...     │     └──────────┬──────────┘
                     │                                  │                │
                     │  Paginate: page 0..N           │                │
                     │  Auth: X-Algolia-* headers      │                │
                     │  Map: AlgoliaHit→CatalogProduct  │                │
                     └──────────────────────────────────┘                │
                                                                         ▼
                              ┌──────────────────────────────────────────────┐
                               │  SyncService::sync_catalog(url)              │
                               │  ├─ HTTP GET → CatalogFile                   │
                               │  ├─ Iter_mut().for_each(sanitize)            │
                               │  ├─ UPSERT INTO products_meta                │
                               │  └─ Diff-based soft-delete: mark SKUs not    │
                               │     in current batch as is_active=0          │
                               │     (from add-soft-delete-products)          │
                              └──────────────────────────────────────────────┘
```

## Field Mapping (Algolia Hit → CatalogProduct)

| Algolia field | CatalogProduct | Transform |
|---|---|---|
| `display_name` | `name` | `.strip()` |
| `brand` | `brand` | Direct, fallback `"Unknown"` |
| `product_id` | `sku` | Prefix with `gc-` |
| `current_price` | `price` | Parse float |
| `currency` | `currency` | Default `"USD"` |
| `condition.lvl0` / `lvl1` | `condition` | See Condition map below |
| `sticker` | `specs_json` | Preserved as `{"stickers": [...]}`. Promotional badges (Price Drop, Vintage), NOT condition. |
| `seoUrl` | `url` | Direct |
| `imageId` | `image_url` | Prepend `https://media.guitarcenter.com/is/image/MMGS7/` |
| Category path | `category`, `subcategory` | From `categories.lvl0..lvl5` facet. Last non-empty → `subcategory`, parent → `category` |
| `inventoryStatus` + `stores[]` | `availability` | `stores` non-empty AND `inventoryStatus` in (1000, 1003) → `in_stock`. Otherwise → `out_of_stock`. Never used as proxy for delisting. |

### Condition normalization (adapter-level)

Replaces the need for changes to the Rust `sanitize()` function. Maps GC's vocabulary to the SQLite CHECK constraint values:

| GC value | `CatalogProduct.condition` | Notes |
|---|---|---|
| `Excellent` | `used` | |
| `Great` | `used` | |
| `Good` | `used` | |
| `Fair` | `used` | |
| `Poor` | `used` | |
| `New` | `new` | |
| `Open Box` | `new` | `specs_json.stickers` gets `"open_box"` |
| `Blemished` | `new` | `specs_json.stickers` gets `"blemished"` |
| `Restock` | `refurbished` | `specs_json.stickers` gets `"restock"` |
| unknown / missing | `unknown` | Fallback |

This normalization is tested exhaustively in the unit tests. The raw GC condition value (e.g., `"Great"`) is preserved in `specs_json.condition_original` so no information is lost.

## File Changes

| File | Action | Description |
|---|---|---|
| `scraper/adapters/guitarcenter.py` | Create | `GuitarCenterAdapter(ScraperPort)` — Algolia API sync client, pagination, field mapping |
| `scraper/cli.py` | Modify | Add `"guitarcenter"` to `--adapter` choices |
| `scraper/tests/fixtures/guitarcenter-sample.json` | Create | Sample Algolia response fixture with all condition variants |
| `scraper/tests/unit/test_guitarcenter.py` | Create | Unit tests: field mapping (every condition), pagination, error handling, URL construction |
| `scraper/tests/contract/test_protocol.py` | Modify | Add `isinstance(GuitarCenterAdapter(), ScraperPort)` assertion |
| `.env.example` | Modify | Add `GC_ALGOLIA_APP_ID` and `GC_ALGOLIA_API_KEY` with descriptions |
| `Makefile` | Modify | Add `scrape-guitarcenter` target |
| `docs/CONTRIBUTING.md` | Modify | Update "Adding a new source adapter" section to reflect actual file paths |

## Interfaces / Contracts

### GuitarCenterAdapter (new)

```python
class GuitarCenterAdapter:
    """Adapter that scrapes Guitar Center catalog via Algolia search API."""

    ALGOLIA_APP_ID: str = ""          # from env GC_ALGOLIA_APP_ID
    ALGOLIA_API_KEY: str = ""         # from env GC_ALGOLIA_API_KEY
    ALGOLIA_INDEX: str = "guitarcenter"  # confirmed from live Algolia query

    def __init__(
        self,
        source_id: str = "guitarcenter",
        session: requests.Session | None = None,
        delay: float = 1.0,
        max_pages: int = 50,
        algolia_app_id: str | None = None,
        algolia_api_key: str | None = None,
    ):
        ...

    def scrape(self, url: str = "") -> CatalogFile:
        """Scrape Guitar Center catalog via Algolia and return a CatalogFile."""
        ...
```

### CLI integration

```python
# In cli.py --adapter choices:
choices=["reverb", "guitarcenter"]

if args.adapter == "guitarcenter":
    from scraper.adapters.guitarcenter import GuitarCenterAdapter
    adapter = GuitarCenterAdapter()
```

## Testing Strategy

| Layer | What | Approach |
|---|---|---|
| Unit (Python) | All 10 condition values map correctly | Load fixture with one product per condition variant, assert each maps to correct `CatalogProduct.condition` + `specs_json.stickers` |
| Unit (Python) | Pagination stops when Algolia returns empty hits array | Mock `_fetch_json`, verify page 0 returns products, page 1 returns empty → stop |
| Unit (Python) | Missing Algolia credentials raise clear error | `GuitarCenterAdapter()` without env vars → `ValueError` mentioning env var names |
| Unit (Python) | Empty catalog returns valid `CatalogFile` with `products: []` | Mock empty hits, verify `catalog.products == []` |
| Contract (Python) | Adapter satisfies `ScraperPort` protocol | `isinstance(adapter, ScraperPort)` via runtime_checkable |
| Contract (Python) | `mypy --strict` passes on `guitarcenter.py` | Already overridden for `adapters.*` in `pyproject.toml` |

## Dependencies & Rollout

This change depends on two upstream changes that MUST merge before the GC adapter can ship.

### Required: add-soft-delete-products

**Blocker.** The current sync layer does `INSERT OR REPLACE` with zero diff logic — items that disappear from a source catalog remain in the DB forever as if they were still active. Before adding a second source (GC), the Rust sync layer needs basic delisting detection:

- A migration adding `is_active INTEGER DEFAULT 1` and `delisted_at INTEGER` to `products_meta`.
- After `upsert_products` completes, a diff pass marks SKUs present in the previous sync batch but absent from the current batch as `is_active = 0, delisted_at = now()`.
- The column `availability` is **NOT** used for this signal. `availability` describes current stock state (`in_stock`/`out_of_stock`/`unknown`) for items that are active. Soft-delete is orthogonal: an item can be in stock (`availability: "in_stock"`) and later delisted (`is_active: 0`) when it disappears from the source.

This change must be designed, spec'd, and merged as its own PR before the GC adapter.

### Required: fix-condition-normalization

GC's 5-value used vocabulary (`Excellent`, `Great`, `Good`, `Fair`, `Poor`) needs the same `condition` → CHECK constraint normalization path as the existing Reverb slugs. This change should normalize conditions in one place (preferably the Rust `sanitize()`), not per-adapter. Until that lands, the adapter-level mapping (see Condition map above) serves as a self-contained workaround but should be replaced once the centralized path exists.

### Rollout plan

```
PR 0: add-soft-delete-products  (migration + diff logic, no adapter changes)
PR 1: GuitarCenterAdapter + tests + CLI + Makefile + docs
      (depends on PR 0 merged first)
```

No migration needed for GC data — it goes into the same `products_meta` table with `source_id = "guitarcenter"`. Price history already has a `source_id` column (migration 004).

## Risk Notes

- **Algolia credential stability**: Keys are `NEXT_PUBLIC_` env vars (exposed to browser in `__ENV.js`). Part of GC's deploy config — rotate with deploys, not per-session. Document re-extraction steps in `.env.example` and `docs/` (browser DevTools or Wayback Machine `__ENV.js`). Rate-limited client required — cap at 10 QPS initially.
- **Algolia key scope**: Confirmed read-only — API key `d04d765e552eb08aff3601eae8f2b729` only grants search query access to the `guitarcenter` and related indices. No write capabilities exposed.
- **robots.txt posture**: GC's `robots.txt` disallows `/search` and various HTML routes but cannot disallow Algolia API calls. Recommend adding a `robots.txt` note in the adapter docstring that the adapter targets Algolia, not GC origin servers.
- **Rate limiting**: Per-adapter `delay` param (matching Reverb pattern), configurable via constructor arg.

## Resolved Questions

The following were confirmed via live Algolia query on 2026-06-25:

| Question | Answer |
|---|---|
| Index name | `guitarcenter` |
| Credentials stability | Public browser env vars (`NEXT_PUBLIC_ALGOLIA_APPID`, `NEXT_PUBLIC_ALGOLIA_APIKEY`). Stable across sessions, rotate with deploy config. |
| Non-USD currencies | No. All prices in USD. No `currency` field in Algolia response. `price` is numeric. |
| Open Box / Blemished / Restock mapping | Confirm current design: map to `condition` + preserve original in `specs_json.stickers`. These use `skuCondition` codes (2=Restock, 3=Open Box, 11=Blemished). |
| Availability/stock field | `inventoryStatus` (1000/1003 = in stock) + `stores[]` (non-empty = available in at least one store). Use both signals: `stores` non-empty AND `inventoryStatus` in (1000, 1003) → `in_stock`. |
