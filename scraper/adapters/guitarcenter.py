# SPDX-License-Identifier: GPL-3.0-or-later

"""Guitar Center marketplace scraper adapter.

Ports & Adapters: implements ScraperPort protocol.
Fetches product listings via the Algolia search API that powers
Guitar Center's client-side search — NOT the guitarcenter.com HTML
pages (which are behind Cloudflare).

See design.md for the full technical rationale and field mapping.
"""

import json
import logging
import os
import time
from typing import Any

import requests
from requests.adapters import HTTPAdapter
from urllib3.util.retry import Retry

from scraper.domain import CatalogFile, CatalogProduct
from scraper.ports import FetchError, ParseError

logger = logging.getLogger(__name__)

# ── Condition normalization maps ──────────────────────────────────────────

_CONDITION_MAP: dict[str, str] = {
    "New": "new",
    "Open Box": "new",
    "Blemished": "new",
    "Restock": "refurbished",
    "Used > Excellent": "used",
    "Used > Great": "used",
    "Used > Good": "used",
    "Used > Fair": "used",
    "Used > Poor": "used",
}

# skuCondition codes from GC's Algolia index
_SKU_CONDITION_MAP: dict[int, tuple[str, str | None]] = {
    1: ("new", None),       # New
    2: ("refurbished", "restock"),      # Restock
    3: ("new", "open_box"),            # Open Box
    11: ("new", "blemished"),          # Blemished
}

# inventoryStatus values that indicate available stock
_IN_STOCK_STATUSES: set[int] = {1000, 1003}

# Base URL for GC product images
_IMAGE_BASE = "https://media.guitarcenter.com/is/image/MMGS7/"


class GuitarCenterAdapter:
    """Adapter that scrapes Guitar Center catalog via Algolia search API.

    This adapter targets the Algolia REST endpoint, NOT Guitar Center's
    HTML pages. GC's robots.txt disallows /search and various HTML routes
    but does not govern Algolia API calls. The Algolia credentials used
    here are exposed to browsers as NEXT_PUBLIC_ env vars — they are
    public, read-only search credentials, not secrets.
    """

    ALGOLIA_APP_ID: str = ""
    ALGOLIA_API_KEY: str = ""
    ALGOLIA_INDEX: str = "guitarcenter"

    def __init__(
        self,
        source_id: str = "guitarcenter",
        session: requests.Session | None = None,
        delay: float = 1.0,
        max_pages: int = 50,
        algolia_app_id: str | None = None,
        algolia_api_key: str | None = None,
    ) -> None:
        """Initialize the adapter.

        Args:
            source_id: Identifier for this source in the catalog.
            session: Optional pre-configured requests Session (for testing).
            delay: Seconds to wait between pagination requests.
            max_pages: Maximum number of pages to scrape.
            algolia_app_id: Algolia application ID. Falls back to
                GC_ALGOLIA_APP_ID env var.
            algolia_api_key: Algolia API key. Falls back to
                GC_ALGOLIA_API_KEY env var.

        Raises:
            ValueError: If neither constructor args nor env vars provide
                credentials.
        """
        self.source_id = source_id
        self.delay = delay
        self.max_pages = max_pages

        # Resolve credentials: constructor args > env vars
        resolved_app_id = algolia_app_id or os.environ.get("GC_ALGOLIA_APP_ID", "")
        resolved_api_key = algolia_api_key or os.environ.get(
            "GC_ALGOLIA_API_KEY", ""
        )

        missing_vars: list[str] = []
        if not resolved_app_id:
            missing_vars.append("GC_ALGOLIA_APP_ID")
        if not resolved_api_key:
            missing_vars.append("GC_ALGOLIA_API_KEY")

        if missing_vars:
            raise ValueError(
                f"Missing Guitar Center Algolia credential(s): "
                f"{', '.join(missing_vars)}. "
                f"Set them as environment variables or pass via constructor."
            )

        self.algolia_app_id = resolved_app_id
        self.algolia_api_key = resolved_api_key
        self.session = session or self._build_session()

    @property
    def query_url(self) -> str:
        """Algolia REST query endpoint for the guitarcenter index."""
        return (
            f"https://{self.algolia_app_id}-dsn.algolia.net"
            f"/1/indexes/{self.ALGOLIA_INDEX}/query"
        )

    # ── ScraperPort interface ──────────────────────────────────────────

    def scrape(self, url: str = "") -> CatalogFile:
        """Scrape Guitar Center catalog via Algolia and return a CatalogFile.

        Args:
            url: Ignored; the adapter always queries the configured Algolia
                index.

        Returns:
            CatalogFile with scraped products.

        Raises:
            FetchError: on HTTP or network failure.
            ParseError: on JSON parsing failure.
        """
        products: list[CatalogProduct] = []
        page = 0

        while page < self.max_pages:
            logger.info(
                "Fetching page %d from Algolia index '%s'",
                page,
                self.ALGOLIA_INDEX,
            )

            try:
                data = self._fetch_json(page)
            except FetchError:
                raise
            except Exception as exc:
                raise FetchError(
                    f"Failed to fetch page {page} from Algolia: {exc}"
                ) from exc

            hits = data.get("hits", [])
            nb_pages = data.get("nbPages", 0)

            if not hits:
                logger.info("No hits on page %d — stopping pagination", page)
                break

            logger.info(
                "Extracted %d products from page %d of %d",
                len(hits),
                page,
                nb_pages,
            )

            for hit in hits:
                try:
                    product = self._map_hit(hit)
                    if product is not None:
                        products.append(product)
                except Exception as exc:
                    logger.warning("Skipping hit due to parse error: %s", exc)
                    continue

            # Stop if this is the last page per Algolia
            if nb_pages > 0 and page + 1 >= nb_pages:
                logger.info(
                    "Page %d is the last (nbPages=%d) — stopping pagination",
                    page,
                    nb_pages,
                )
                break

            page += 1
            time.sleep(self.delay)

        return CatalogFile.create(source_id=self.source_id, products=products)

    # ── HTTP helpers ───────────────────────────────────────────────────

    def _build_session(self) -> requests.Session:
        """Build a requests session with Algolia auth headers and retry."""
        session = requests.Session()
        session.headers.update({
            "X-Algolia-Application-Id": self.algolia_app_id,
            "X-Algolia-API-Key": self.algolia_api_key,
            "Content-Type": "application/json",
            "Accept": "application/json",
            "User-Agent": (
                "Mozilla/5.0 (compatible; GuitarHub-Scraper/0.1; "
                "+https://github.com/willbennett/guitarhub)"
            ),
        })
        retry_strategy = Retry(
            total=3,
            backoff_factor=1.0,
            status_forcelist=[500, 502, 503, 504],
            allowed_methods=["POST"],
            raise_on_status=False,
        )
        adapter = HTTPAdapter(max_retries=retry_strategy)
        session.mount("https://", adapter)
        session.mount("http://", adapter)
        return session

    def _fetch(self, page: int) -> requests.Response:
        """POST a query to Algolia and return the raw response.

        Args:
            page: Zero-based page number for pagination.

        Returns:
            requests.Response from Algolia.

        Raises:
            FetchError: on HTTP errors, timeouts, or connection failures.
        """
        url = self.query_url
        body = {
            "params": f"page={page}&hitsPerPage=50",
        }

        try:
            response = self.session.post(url, json=body, timeout=30)
        except requests.exceptions.RetryError as exc:
            raise FetchError(
                f"Max retries exceeded for {url}: {exc}"
            ) from exc
        except requests.exceptions.Timeout as exc:
            raise FetchError(f"Request timed out for {url}: {exc}") from exc
        except requests.exceptions.ConnectionError as exc:
            raise FetchError(f"Connection failed for {url}: {exc}") from exc

        if response.status_code == 404:
            raise FetchError(f"Index not found (404): {url}")
        if response.status_code >= 400:
            raise FetchError(
                f"HTTP {response.status_code} for {url}: "
                f"{response.reason}"
            )

        return response

    def _fetch_json(self, page: int) -> dict[str, Any]:
        """Fetch JSON from Algolia for the given page.

        Args:
            page: Zero-based page number.

        Returns:
            Parsed JSON response dict.

        Raises:
            ParseError: on invalid JSON response.
        """
        response = self._fetch(page)
        try:
            result: dict[str, Any] = json.loads(response.text)
            return result
        except json.JSONDecodeError as exc:
            raise ParseError(
                f"Invalid JSON from Algolia (page {page}): {exc}"
            ) from exc

    # ── Hit mapping ────────────────────────────────────────────────────

    def _map_hit(self, hit: dict[str, Any]) -> CatalogProduct | None:
        """Map an Algolia hit dict to a CatalogProduct.

        Args:
            hit: A single hit from Algolia search results.

        Returns:
            CatalogProduct with mapped fields, or None if the hit is
            missing critical data (no display_name).
        """
        name = hit.get("display_name", "")
        if not name:
            return None

        # ── SKU ────────────────────────────────────────────────────────
        product_id = hit.get("product_id", "")
        if not product_id:
            identifiers = hit.get("identifiers", {}) or {}
            product_id = identifiers.get("gcItemNumber", "")
        sku = f"gc-{product_id}" if product_id else "gc-"

        # ── Price ──────────────────────────────────────────────────────
        price = _parse_price(hit.get("current_price"))

        # ── Brand ──────────────────────────────────────────────────────
        brand = hit.get("brand")
        if brand is None or brand == "":
            brand = "Unknown"

        # ── Image URL ─────────────────────────────────────────────────
        image_id = hit.get("imageId")
        image_url = f"{_IMAGE_BASE}{image_id}" if image_id else ""

        # ── URL ────────────────────────────────────────────────────────
        seo_url = hit.get("seoUrl", "")

        # ── Category ───────────────────────────────────────────────────
        category, subcategory = _extract_category(hit)

        # ── Condition ──────────────────────────────────────────────────
        condition, specs = self._normalize_condition(hit)

        # ── Stickers ───────────────────────────────────────────────────
        _merge_stickers(hit, specs)

        # ── Availability ───────────────────────────────────────────────
        availability = _get_availability(hit)

        return CatalogProduct(
            sku=sku,
            name=name.strip(),
            brand=brand.strip(),
            model="",
            category=category,
            subcategory=subcategory,
            price=price,
            currency="USD",
            condition=condition,
            availability=availability,
            url=seo_url,
            image_url=image_url,
            specs_json=json.dumps(specs, separators=(",", ":")),
        )

    def _normalize_condition(self, hit: dict[str, Any]) -> tuple[str, dict[str, Any]]:
        """Normalize GC condition to 4-value vocabulary.

        Maps GC's 9-condition vocabulary to (new, used, refurbished,
        unknown). Preserves the original raw condition value in the
        returned specs dict under ``condition_original``.

        Returns:
            Tuple of (normalized_condition, specs_dict).
        """
        specs: dict[str, Any] = {}
        condition_lvl1 = ""
        condition_lvl0 = ""

        condition = hit.get("condition", {}) or {}
        if isinstance(condition, dict):
            condition_lvl0 = condition.get("lvl0", "")
            condition_lvl1 = condition.get("lvl1", "")

        # Check skuCondition first (for Open Box, Blemished, Restock)
        sku_condition = hit.get("skuCondition")
        if sku_condition is not None:
            normalized, sticker = _SKU_CONDITION_MAP.get(
                int(sku_condition), ("unknown", None)
            )
            # Derive original name from skuCondition
            sku_names = {2: "Restock", 3: "Open Box", 11: "Blemished"}
            original = sku_names.get(int(sku_condition), condition_lvl1 or condition_lvl0)
            specs["condition_original"] = original
            if sticker:
                specs.setdefault("stickers", []).append(sticker)
            return normalized, specs

        # Use condition.lvl1 if available (e.g., "Used > Excellent")
        raw_condition = condition_lvl1 or condition_lvl0
        specs["condition_original"] = raw_condition or ""

        if not raw_condition:
            return "unknown", specs

        normalized = _CONDITION_MAP.get(raw_condition, "unknown")
        return normalized, specs


# ── Standalone helpers ────────────────────────────────────────────────────


def _parse_price(value: Any) -> float:
    """Parse price from a numeric field.

    Args:
        value: A numeric value (int or float) or None.

    Returns:
        Float price, or 0.0 if unparseable.
    """
    if value is None:
        return 0.0
    try:
        return float(value)
    except (ValueError, TypeError):
        return 0.0


def _extract_category(hit: dict[str, Any]) -> tuple[str, str]:
    """Extract category and subcategory from Algolia hierarchical facets.

    ``categories.lvl0`` through ``categories.lvl5`` form a hierarchy.
    The last non-empty level becomes ``subcategory`` and its parent
    becomes ``category``.

    Returns:
        Tuple of (category, subcategory).
    """
    categories = hit.get("categories", {}) or {}
    if not categories:
        return "", ""

    levels: list[str] = []
    for i in range(6):
        key = f"lvl{i}"
        val = categories.get(key, "")
        if val:
            levels.append(val.strip())

    if not levels:
        return "", ""

    if len(levels) == 1:
        return levels[0], ""

    return levels[-2], levels[-1]


def _get_availability(hit: dict[str, Any]) -> str:
    """Determine availability from Algolia stock signals.

    Both conditions must be met for ``in_stock``:
    - ``inventoryStatus`` is present and in (1000, 1003)
    - ``stores`` is a non-empty list

    Otherwise returns ``out_of_stock``.
    """
    inventory_status = hit.get("inventoryStatus")
    stores = hit.get("stores", None)

    if inventory_status is None:
        return "out_of_stock"

    try:
        is_valid_status = int(inventory_status) in _IN_STOCK_STATUSES
    except (ValueError, TypeError):
        is_valid_status = False

    if not is_valid_status:
        return "out_of_stock"

    if not stores or not isinstance(stores, list) or len(stores) == 0:
        return "out_of_stock"

    return "in_stock"


def _merge_stickers(hit: dict[str, Any], specs: dict[str, Any]) -> None:
    """Merge original stickers from the hit into the specs dict.

    Original stickers from the ``sticker`` field (Price Drop, Vintage,
    etc.) are added to the ``stickers`` list in ``specs`` alongside
    any condition-derived stickers already present. The ``stickers``
    key is always present (defaults to empty list).

    Args:
        hit: The Algolia hit dict.
        specs: The mutable specs dict to update.
    """
    specs.setdefault("stickers", [])
    original_stickers = hit.get("sticker")

    if original_stickers is None:
        return

    if isinstance(original_stickers, list):
        for s in original_stickers:
            if s and s not in specs["stickers"]:
                specs["stickers"].append(s)
    elif isinstance(original_stickers, str) and original_stickers:
        if original_stickers not in specs["stickers"]:
            specs["stickers"].append(original_stickers)
