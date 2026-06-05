# SPDX-License-Identifier: GPL-3.0-or-later

"""Reverb.com marketplace scraper adapter.

Ports & Adapters: implements ScraperPort protocol.
Fetches product listings from Reverb's marketplace pages,
parses HTML, and maps fields to CatalogProduct.
"""

import logging
import re
import time
from urllib.parse import urljoin

import requests
from bs4 import BeautifulSoup
from requests.adapters import HTTPAdapter
from urllib3.util.retry import Retry

from scraper.domain import CatalogFile, CatalogProduct
from scraper.ports import FetchError, ParseError, ScraperPort

logger = logging.getLogger(__name__)

# CSS selector patterns for Reverb marketplace listing cards.
# Reverb's HTML structure may change; these target known patterns.
CARDS_SELECTOR = "div.grid-card, article.listing-card, div[data-listing-id]"
TITLE_SELECTORS = [
    ".grid-card__title",
    ".listing-card__title",
    "h2",
    "h3",
    ".product-title",
    ".card-title",
]
BRAND_SELECTORS = [
    ".grid-card__brand",
    ".listing-card__brand",
    ".brand",
]
PRICE_SELECTORS = [
    ".grid-card__price",
    ".listing-card__price",
    ".price",
    "[data-testid='price']",
]
CONDITION_SELECTORS = [
    ".grid-card__condition",
    ".listing-card__condition",
    ".condition",
]
SELLER_SELECTORS = [
    ".grid-card__seller",
    ".listing-card__seller",
    ".seller-name",
]
LOCATION_SELECTORS = [
    ".grid-card__location",
    ".listing-card__location",
    ".location",
]
NEXT_PAGE_SELECTORS = [
    "a.pagination__next",
    "a[rel='next']",
    "a.next",
    "button.load-more",
    "[data-load-more]",
]


class ReverbAdapter:
    """Adapter that scrapes product listings from Reverb marketplace."""

    BASE_URL = "https://reverb.com/marketplace"

    def __init__(
        self,
        source_id: str = "reverb",
        session: requests.Session | None = None,
        delay: float = 1.0,
        max_pages: int = 10,
    ):
        """Initialize the adapter.

        Args:
            source_id: Identifier for this source in the catalog.
            session: Optional pre-configured requests Session (for testing).
            delay: Seconds to wait between pagination requests.
            max_pages: Maximum number of pages to scrape.
        """
        self.source_id = source_id
        self.session = session or self._build_session()
        self.delay = delay
        self.max_pages = max_pages

    # ── ScraperPort interface ──────────────────────────────────────────

    def scrape(self, url: str = "") -> CatalogFile:
        """Scrape Reverb marketplace and return a CatalogFile.

        Args:
            url: Marketplace URL to scrape. Defaults to electric guitars.

        Returns:
            CatalogFile with scraped products.

        Raises:
            FetchError: on HTTP or network failure.
            ParseError: on HTML parsing failure.
        """
        start_url = url or f"{self.BASE_URL}/electric-guitars"
        products: list[CatalogProduct] = []
        page = 1

        while page <= self.max_pages:
            page_url = f"{start_url}?page={page}"
            logger.info("Fetching page %d: %s", page, page_url)

            try:
                response = self._fetch(page_url)
            except FetchError:
                raise
            except Exception as exc:
                raise FetchError(f"Failed to fetch {page_url}: {exc}") from exc

            try:
                soup = BeautifulSoup(response.text, "html.parser")
            except Exception as exc:
                raise ParseError(f"Failed to parse HTML from {page_url}: {exc}") from exc

            page_products = self._extract_products(soup)
            logger.info("Extracted %d products from page %d", len(page_products), page)

            if not page_products:
                break

            products.extend(page_products)

            if not self._has_next_page(soup):
                break

            page += 1
            time.sleep(self.delay)

        return CatalogFile.create(source_id=self.source_id, products=products)

    # ── HTTP helpers ───────────────────────────────────────────────────

    @staticmethod
    def _build_session() -> requests.Session:
        """Build a requests session with retry and user-agent."""
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

        session.headers.update({
            "User-Agent": (
                "Mozilla/5.0 (compatible; GuitarHub-Scraper/0.1; "
                "+https://github.com/willbennett/guitarhub)"
            ),
            "Accept": "text/html,application/xhtml+xml",
            "Accept-Language": "en-US,en;q=0.9",
        })

        return session

    def _fetch(self, url: str) -> requests.Response:
        """Fetch a URL with timeout and status check.

        Raises FetchError on permanent HTTP errors.
        Retries transient errors via the session's Retry adapter.
        """
        try:
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

    # ── HTML extraction ────────────────────────────────────────────────

    def _extract_products(self, soup: BeautifulSoup) -> list[CatalogProduct]:
        """Extract CatalogProduct instances from parsed HTML."""
        cards = soup.select(CARDS_SELECTOR)
        products: list[CatalogProduct] = []

        for card in cards:
            try:
                product = self._extract_one(card)
                if product is not None:
                    products.append(product)
            except Exception as exc:
                logger.warning("Skipping card due to parse error: %s", exc)
                continue

        return products

    def _extract_one(self, card: BeautifulSoup) -> CatalogProduct | None:
        """Extract a single CatalogProduct from a listing card element."""
        # ── Name / title ────────────────────────────────────────────
        name = self._select_text(card, TITLE_SELECTORS)
        if not name:
            return None  # skip cards without a title

        # ── Product URL ─────────────────────────────────────────────
        link = card.select_one("a[href*='/item/']")
        relative_url = link.get("href") if link else ""
        product_url = urljoin("https://reverb.com", relative_url) if relative_url else ""

        # ── SKU from URL ────────────────────────────────────────────
        sku = self._extract_sku(relative_url or product_url, name)

        # ── Image URL ───────────────────────────────────────────────
        img = card.select_one("img")
        image_url = ""
        if img:
            image_url = (
                img.get("src")
                or img.get("data-src")
                or img.get("data-lazy-src")
                or ""
            )

        # ── Price ───────────────────────────────────────────────────
        price = self._parse_price(self._select_text(card, PRICE_SELECTORS))

        # ── Other fields ────────────────────────────────────────────
        brand = self._select_text(card, BRAND_SELECTORS) or ""
        condition = self._select_text(card, CONDITION_SELECTORS) or ""
        seller = self._select_text(card, SELLER_SELECTORS) or ""
        location = self._select_text(card, LOCATION_SELECTORS) or ""

        return CatalogProduct(
            sku=sku,
            name=name.strip(),
            brand=brand.strip(),
            price=price,
            url=product_url,
            image_url=image_url,
            condition=condition.strip().lower(),
            seller=seller.strip(),
            location=location.strip(),
        )

    # ── Selector helpers ───────────────────────────────────────────────

    @staticmethod
    def _select_text(
        element: BeautifulSoup, selectors: list[str]
    ) -> str:
        """Return text from the first matching selector, stripped."""
        for selector in selectors:
            found = element.select_one(selector)
            if found and found.get_text(strip=True):
                return found.get_text(strip=True)
        return ""

    @staticmethod
    def _extract_sku(url_fragment: str, name: str) -> str:
        """Generate a stable-ish SKU from URL and product name.

        Uses the Reverb item UUID from the URL when available,
        otherwise falls back to a sanitized name hash.
        """
        # Reverb item URLs: /item/<uuid>-<slug>
        match = re.search(r"/item/([a-f0-9-]+)", url_fragment)
        if match:
            item_id = match.group(1)
            short_id = item_id.split("-")[0][:8]
            return f"reverb-{short_id}"

        # Fallback: hash the name
        name_slug = re.sub(r"[^a-zA-Z0-9]", "", name)[:20].lower()
        return f"reverb-{name_slug}"

    @staticmethod
    def _parse_price(text: str) -> float:
        """Parse a price string like '$1,599.99' or 'Price: $2,000'."""
        if not text:
            return 0.0
        match = re.search(r"[\d,]+(?:\.\d{2})?", text.replace(",", ""))
        if match:
            try:
                return float(match.group(0).replace(",", ""))
            except ValueError:
                return 0.0
        return 0.0

    @staticmethod
    def _has_next_page(soup: BeautifulSoup) -> bool:
        """Check if the page has a next-page / load-more control."""
        for selector in NEXT_PAGE_SELECTORS:
            element = soup.select_one(selector)
            if element is not None:
                # Check it's not disabled
                disabled = (
                    element.get("disabled")
                    or element.get("aria-disabled")
                    or False
                )
                if not disabled:
                    return True
        return False
