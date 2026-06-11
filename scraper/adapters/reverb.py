# SPDX-License-Identifier: GPL-3.0-or-later

"""Reverb.com marketplace scraper adapter.

Ports & Adapters: implements ScraperPort protocol.
Fetches product listings from Reverb's JSON API,
and maps fields to CatalogProduct.
"""

import json
import logging
import time
from typing import Any

import requests
from requests.adapters import HTTPAdapter
from urllib3.util.retry import Retry

try:
    import curl_cffi.requests as curl_requests
    HAS_CURL_CFFI = True
except ImportError:
    curl_requests = None
    HAS_CURL_CFFI = False

from scraper.domain import CatalogFile, CatalogProduct
from scraper.ports import FetchError, ParseError

logger = logging.getLogger(__name__)


class ReverbAdapter:
    """Adapter that scrapes product listings from Reverb marketplace via JSON API."""

    BASE_URL = "https://reverb.com/api/listings"

    # Reverb product_type slugs → human-readable category names.
    PRODUCT_TYPE_CATEGORIES: dict[str, str] = {
        "electric-guitars": "Electric Guitars",
        "acoustic-guitars": "Acoustic Guitars",
        "bass-guitars": "Bass Guitars",
        "guitar-amplifiers": "Amplifiers",
        "bass-amplifiers": "Amplifiers",
        "pedals": "Pedals & Effects",
        "effects-pedals": "Pedals & Effects",
        "drums-percussion": "Drums & Percussion",
        "keyboards-synths": "Keyboards & Synths",
        "pro-audio": "Pro Audio",
        "string-instruments": "String Instruments",
        "wind-instruments": "Wind Instruments",
    }

    def __init__(
        self,
        source_id: str = "reverb",
        session: requests.Session | None = None,
        delay: float = 1.0,
        max_pages: int = 10,
        product_type: str = "electric-guitars",
    ):
        """Initialize the adapter.

        Args:
            source_id: Identifier for this source in the catalog.
            session: Optional pre-configured requests Session (for testing).
            delay: Seconds to wait between pagination requests.
            max_pages: Maximum number of pages to scrape.
            product_type: Reverb product_type slug used in API URL and
                category mapping (default: "electric-guitars").
        """
        self.source_id = source_id
        self.session = session or self._build_session()
        self.delay = delay
        self.max_pages = max_pages
        self.product_type = product_type

    # ── ScraperPort interface ──────────────────────────────────────────

    def scrape(self, url: str = "") -> CatalogFile:
        """Scrape Reverb marketplace and return a CatalogFile.

        Args:
            url: Ignored; the adapter builds the API URL from
                ``self.product_type``.

        Returns:
            CatalogFile with scraped products.

        Raises:
            FetchError: on HTTP or network failure.
            ParseError: on JSON parsing failure.
        """
        products: list[CatalogProduct] = []
        page = 1

        while page <= self.max_pages:
            page_url = (
                f"{self.BASE_URL}?product_type={self.product_type}"
                f"&per_page=24&page={page}"
            )
            logger.info("Fetching page %d: %s", page, page_url)

            try:
                data = self._fetch_json(page_url)
            except FetchError:
                raise
            except Exception as exc:
                raise FetchError(f"Failed to fetch {page_url}: {exc}") from exc

            listings = data.get("listings", [])
            total_pages = data.get("total_pages", 1)
            current_page = data.get("current_page", page)

            logger.info(
                "Extracted %d products from page %d/%d",
                len(listings),
                current_page,
                total_pages,
            )

            if not listings:
                break

            for listing in listings:
                try:
                    product = self._map_listing(listing)
                    if product is not None:
                        products.append(product)
                except Exception as exc:
                    logger.warning("Skipping listing due to parse error: %s", exc)
                    continue

            if current_page >= total_pages:
                break

            page += 1
            time.sleep(self.delay)

        return CatalogFile.create(source_id=self.source_id, products=products)

    # ── HTTP helpers ───────────────────────────────────────────────────

    @staticmethod
    def _build_session() -> Any:
        """Build a requests session with retry and user-agent."""
        headers = {
            "User-Agent": (
                "Mozilla/5.0 (compatible; GuitarHub-Scraper/0.1; "
                "+https://github.com/willbennett/guitarhub)"
            ),
            "Accept": "application/json",
            "Accept-Language": "en-US,en;q=0.9",
        }

        if HAS_CURL_CFFI:
            session = curl_requests.Session()
            session.headers.update(headers)
            return session

        session = requests.Session()
        retry_strategy = Retry(
            total=3,
            backoff_factor=1.0,
            status_forcelist=[500, 502, 503, 504],
            allowed_methods=["GET"],
            raise_on_status=False,
        )
        adapter = HTTPAdapter(max_retries=retry_strategy)
        session.mount("https://", adapter)
        session.mount("http://", adapter)
        session.headers.update(headers)
        return session

    def _fetch(self, url: str) -> requests.Response:
        """Fetch a URL with timeout and status check.

        Raises FetchError on permanent HTTP errors.
        Retries transient errors via the session's Retry adapter.
        """
        try:
            if HAS_CURL_CFFI and isinstance(
                self.session, curl_requests.Session
            ):
                response = self.session.get(
                    url, timeout=30, impersonate="chrome120"
                )
            else:
                response = self.session.get(url, timeout=30)
        except requests.exceptions.RetryError as exc:
            raise FetchError(
                f"Max retries exceeded for {url}: {exc}"
            ) from exc
        except requests.exceptions.Timeout as exc:
            raise FetchError(f"Request timed out for {url}: {exc}") from exc
        except requests.exceptions.ConnectionError as exc:
            raise FetchError(f"Connection failed for {url}: {exc}") from exc

        if response.status_code == 404:
            raise FetchError(f"Page not found (404): {url}")
        if response.status_code >= 400:
            raise FetchError(
                f"HTTP {response.status_code} for {url}: "
                f"{response.reason}"
            )

        return response

    def _fetch_json(self, url: str) -> dict:
        """Fetch JSON from a URL.

        Sets Accept: application/json header and parses JSON response.
        Raises ParseError on invalid JSON.
        """
        response = self._fetch(url)
        try:
            return json.loads(response.text)
        except json.JSONDecodeError as exc:
            raise ParseError(f"Invalid JSON from {url}: {exc}") from exc

    # ── Listing mapping ──────────────────────────────────────────────────

    def _map_listing(self, listing: dict) -> CatalogProduct | None:
        """Map a Reverb API listing dict to a CatalogProduct."""
        listing_id = listing.get("id")
        if listing_id is None:
            return None

        title = listing.get("title", "")
        if not title:
            return None

        sku = self._extract_sku(str(listing_id), title)

        price_data = listing.get("price", {}) or {}
        price = self._parse_price(price_data.get("amount", ""))
        currency = listing.get("listing_currency", "USD")

        condition = listing.get("condition_slug", "")
        state = listing.get("state", {}) or {}
        if isinstance(state, str):
            availability = "in_stock" if state == "live" else "unknown"
        else:
            availability = "in_stock" if state.get("slug") == "live" else "unknown"

        links = listing.get("_links", {})
        web_link = links.get("web", {}) if isinstance(links, dict) else {}
        photo_link = links.get("photo", {}) if isinstance(links, dict) else {}
        product_url = web_link.get("href", "") if isinstance(web_link, dict) else ""
        image_url = photo_link.get("href", "") if isinstance(photo_link, dict) else ""

        # Fallback to first photo if no photo link
        if not image_url:
            photos = listing.get("photos", [])
            if photos and isinstance(photos, list):
                first_photo = photos[0]
                if isinstance(first_photo, dict):
                    photo_links = first_photo.get("_links", {})
                    if isinstance(photo_links, dict):
                        image_url = photo_links.get("large_crop", {}).get("href", "") or photo_links.get("small_crop", {}).get("href", "")

        brand = listing.get("make", "")
        model = listing.get("model", "")
        seller = listing.get("shop_name", "")
        location = listing.get("slug", "")

        # Derive category from the product_type used in the API request.
        # The list endpoint doesn't return per-listing category data, but
        # we know the product_type from the URL filter. Fall back to the
        # slug itself if the mapping is unknown.
        category = self.PRODUCT_TYPE_CATEGORIES.get(
            self.product_type, self.product_type.replace("-", " ").title()
        )

        return CatalogProduct(
            sku=sku,
            name=title.strip(),
            brand=brand.strip(),
            model=model.strip(),
            category=category,
            subcategory="",
            price=price,
            currency=currency,
            condition=condition.strip().lower(),
            availability=availability,
            url=product_url,
            image_url=image_url,
            seller=seller.strip(),
            location=location.strip(),
        )

    @staticmethod
    def _extract_sku(listing_id: str, name: str) -> str:
        """Generate a stable SKU from listing id."""
        return f"reverb-{listing_id}"

    @staticmethod
    def _parse_price(text: str) -> float:
        """Parse a price string like '1599.99' or '$1,599.99'."""
        if not text:
            return 0.0
        cleaned = text.replace(",", "").replace("$", "")
        match = __import__("re").search(r"[\d.]+(?:\.\d{2})?", cleaned)
        if match:
            try:
                return float(match.group(0))
            except ValueError:
                return 0.0
        return 0.0
