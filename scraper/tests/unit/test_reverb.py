"""Unit tests for ReverbAdapter — HTML extraction and URL generation."""

from pathlib import Path
from unittest.mock import MagicMock, patch

import pytest
import requests
from bs4 import BeautifulSoup

from scraper.adapters.reverb import ReverbAdapter
from scraper.ports import FetchError

FIXTURES_DIR = Path(__file__).parent.parent / "fixtures"


# ── Fixtures ─────────────────────────────────────────────────────────────


@pytest.fixture
def adapter() -> ReverbAdapter:
    """Return a ReverbAdapter with no real HTTP session."""
    return ReverbAdapter(source_id="reverb", session=MagicMock())


@pytest.fixture
def sample_html() -> str:
    """Load the sample Reverb marketplace HTML fixture."""
    path = FIXTURES_DIR / "reverb-sample.html"
    return path.read_text(encoding="utf-8")


@pytest.fixture
def sample_soup(sample_html: str) -> BeautifulSoup:
    """Parse sample HTML into BeautifulSoup."""
    return BeautifulSoup(sample_html, "html.parser")


# ── URL generation ───────────────────────────────────────────────────────


class TestUrlGeneration:
    """Adapter constructs correct marketplace URLs."""

    def test_default_url_is_electric_guitars(self):
        """Default scrape URL points to electric guitars marketplace."""
        adapter = ReverbAdapter()
        assert adapter.BASE_URL == "https://reverb.com/marketplace"

    def test_scrape_uses_provided_url(self):
        """Adapter uses url parameter when provided."""
        adapter = ReverbAdapter(session=MagicMock())
        adapter._fetch = MagicMock(return_value=MagicMock(text="<html></html>"))
        adapter._extract_products = MagicMock(return_value=[])

        custom_url = "https://reverb.com/marketplace/bass-guitars"
        adapter.scrape(url=custom_url)
        adapter._fetch.assert_called_once()
        call_url = adapter._fetch.call_args[0][0]
        assert custom_url in call_url


# ── HTML extraction ──────────────────────────────────────────────────────


class TestExtractProducts:
    """HTML parsing and field mapping from fixture."""

    def test_extract_products_from_fixture(self, adapter, sample_soup):
        """Extract all products from the sample HTML."""
        products = adapter._extract_products(sample_soup)
        assert len(products) == 2, "Expected 2 products in fixture"

    def test_field_mapping_first_product(self, adapter, sample_soup):
        """First product fields are correctly mapped."""
        products = adapter._extract_products(sample_soup)
        product = products[0]

        assert product.sku == "reverb-a1b2c3d4"
        assert "Fender American Professional II Stratocaster" in product.name
        assert product.brand == "Fender"
        assert product.price == 1599.99
        assert product.condition == "new"
        assert "fender-american-professional" in product.url
        assert "images.reverb.com" in product.image_url
        assert product.seller == "Reverb Bazaar"
        assert product.location == "Austin, TX"

    def test_field_mapping_second_product(self, adapter, sample_soup):
        """Second product fields are correctly mapped."""
        products = adapter._extract_products(sample_soup)
        product = products[1]

        assert "Gibson Les Paul Standard" in product.name
        assert product.brand == "Gibson"
        assert product.price == 2499.99
        assert product.condition == "used"

    def test_defaults_for_missing_fields(self, adapter):
        """Cards with missing optional fields still produce valid products."""
        html = """
        <div class="grid-card">
            <a href="/item/test-001-minimal">
                <p class="grid-card__title">Minimal Product</p>
                <p class="grid-card__price">$99.99</p>
            </a>
        </div>
        """
        soup = BeautifulSoup(html, "html.parser")
        products = adapter._extract_products(soup)
        assert len(products) == 1
        p = products[0]
        assert p.name == "Minimal Product"
        assert p.price == 99.99
        assert p.brand == ""  # default
        assert p.condition == ""  # default
        assert p.seller == ""  # default
        assert p.location == ""  # default
        assert p.currency == "USD"  # default
        assert p.availability == "in_stock"  # default

    def test_skip_card_without_title(self, adapter):
        """Cards missing a title are skipped (no name → no product)."""
        html = """
        <div class="grid-card">
            <a href="/item/test-no-title">
                <p class="grid-card__price">$99.99</p>
            </a>
        </div>
        """
        soup = BeautifulSoup(html, "html.parser")
        products = adapter._extract_products(soup)
        assert len(products) == 0

    def test_empty_page_returns_empty_list(self, adapter):
        """A page with no listing cards returns empty list."""
        html = "<html><body><p>No listings found.</p></body></html>"
        soup = BeautifulSoup(html, "html.parser")
        products = adapter._extract_products(soup)
        assert len(products) == 0

    def test_multiple_selectors_fallback(self, adapter):
        """Adapter tries alternative selectors if primary ones fail."""
        html = """
        <article class="listing-card">
            <a href="/item/test-alt-001">
                <h3 class="listing-card__title">Alternate Layout Product</h3>
                <span class="price">$599.99</span>
                <span class="condition">New</span>
            </a>
        </article>
        """
        soup = BeautifulSoup(html, "html.parser")
        products = adapter._extract_products(soup)
        assert len(products) == 1
        assert products[0].name == "Alternate Layout Product"
        assert products[0].price == 599.99


# ── Price parsing ────────────────────────────────────────────────────────


class TestPriceParsing:
    """Price string normalization."""

    def test_standard_price(self):
        assert ReverbAdapter._parse_price("$1,599.99") == 1599.99

    def test_price_without_cents(self):
        assert ReverbAdapter._parse_price("$2,000") == 2000.0

    def test_price_with_currency_symbol(self):
        assert ReverbAdapter._parse_price("€899") == 899.0

    def test_price_in_text(self):
        assert ReverbAdapter._parse_price("Price: $1,299.00") == 1299.0

    def test_empty_string(self):
        assert ReverbAdapter._parse_price("") == 0.0

    def test_non_numeric_text(self):
        assert ReverbAdapter._parse_price("Contact for price") == 0.0


# ── SKU generation ───────────────────────────────────────────────────────


class TestSkuGeneration:
    """SKU extraction from URLs."""

    def test_sku_from_uuid_url(self):
        sku = ReverbAdapter._extract_sku(
            "/item/a1b2c3d4-e5f6-7890-abcd-ef1234567890-fender-strat",
            "Fender Stratocaster",
        )
        assert sku == "reverb-a1b2c3d4"

    def test_sku_fallback_to_name(self):
        sku = ReverbAdapter._extract_sku(
            "/item/some-product",
            "Fender Stratocaster",
        )
        assert "reverb-" in sku
        assert len(sku) > 7


# ── Pagination detection ─────────────────────────────────────────────────


class TestPagination:
    """Next-page detection."""

    def test_has_next_page_with_link(self):
        html = '<a class="pagination__next" href="?page=2">Next</a>'
        soup = BeautifulSoup(html, "html.parser")
        assert ReverbAdapter._has_next_page(soup) is True

    def test_has_next_page_with_load_more(self):
        html = '<button class="load-more">Load more</button>'
        soup = BeautifulSoup(html, "html.parser")
        assert ReverbAdapter._has_next_page(soup) is True

    def test_no_next_page(self):
        html = "<html><body>Page 1 of 1</body></html>"
        soup = BeautifulSoup(html, "html.parser")
        assert ReverbAdapter._has_next_page(soup) is False

    def test_disabled_next_button(self):
        html = '<a class="pagination__next" aria-disabled="true">Next</a>'
        soup = BeautifulSoup(html, "html.parser")
        assert ReverbAdapter._has_next_page(soup) is False


# ── HTTP error handling ──────────────────────────────────────────────────


class TestHttpErrorHandling:
    """Adapter raises typed errors on HTTP failures."""

    def test_404_raises_fetch_error(self):
        """A 404 should raise FetchError immediately (no retry)."""
        adapter = ReverbAdapter(session=MagicMock())
        mock_response = MagicMock(spec=requests.Response)
        mock_response.status_code = 404
        mock_response.reason = "Not Found"
        adapter.session.get.return_value = mock_response

        with pytest.raises(FetchError, match="404"):
            adapter._fetch("https://reverb.com/missing")

    def test_timeout_raises_fetch_error(self):
        """A timeout should raise FetchError."""
        adapter = ReverbAdapter(session=MagicMock())
        adapter.session.get.side_effect = requests.exceptions.Timeout("timed out")

        with pytest.raises(FetchError, match="timed out"):
            adapter._fetch("https://reverb.com/slow")

    def test_connection_error_raises_fetch_error(self):
        """A connection error should raise FetchError."""
        adapter = ReverbAdapter(session=MagicMock())
        adapter.session.get.side_effect = requests.exceptions.ConnectionError("refused")

        with pytest.raises(FetchError, match="refused"):
            adapter._fetch("https://reverb.com/down")

    def test_400_raises_fetch_error(self):
        """A 400 should raise FetchError (client error, no retry)."""
        adapter = ReverbAdapter(session=MagicMock())
        mock_response = MagicMock(spec=requests.Response)
        mock_response.status_code = 400
        mock_response.reason = "Bad Request"
        adapter.session.get.return_value = mock_response

        with pytest.raises(FetchError, match="400"):
            adapter._fetch("https://reverb.com/bad-request")


# ── Session creation ─────────────────────────────────────────────────────


class TestSession:
    """Adapter session configuration."""

    def test_session_has_user_agent(self):
        """Session includes a descriptive User-Agent header."""
        session = ReverbAdapter._build_session()
        ua = session.headers.get("User-Agent", "")
        assert "GuitarHub-Scraper" in ua
        assert "github.com" in ua

    def test_session_has_retry_adapter(self):
        """Session mounts HTTPAdapter with retry config."""
        session = ReverbAdapter._build_session()
        adapter = session.get_adapter("https://reverb.com")
        assert isinstance(adapter, requests.adapters.HTTPAdapter)
