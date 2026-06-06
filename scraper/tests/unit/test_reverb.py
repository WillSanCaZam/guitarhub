# SPDX-License-Identifier: GPL-3.0-or-later

"""Unit tests for ReverbAdapter — JSON API extraction and mapping."""

import json
from pathlib import Path
from unittest.mock import MagicMock, patch

import pytest
import requests

from scraper.adapters.reverb import ReverbAdapter
from scraper.ports import FetchError

FIXTURES_DIR = Path(__file__).parent.parent / "fixtures"


# ── Fixtures ─────────────────────────────────────────────────────────────


@pytest.fixture
def adapter() -> ReverbAdapter:
    """Return a ReverbAdapter with no real HTTP session."""
    return ReverbAdapter(source_id="reverb", session=MagicMock())


@pytest.fixture
def sample_json() -> dict:
    """Load the sample Reverb JSON API fixture."""
    path = FIXTURES_DIR / "reverb-sample.json"
    return json.loads(path.read_text(encoding="utf-8"))


# ── URL generation ───────────────────────────────────────────────────────


class TestUrlGeneration:
    """Adapter constructs correct API URLs."""

    def test_base_url_is_api_endpoint(self):
        """BASE_URL points to the JSON API."""
        adapter = ReverbAdapter()
        assert adapter.BASE_URL == "https://reverb.com/api/listings"

    def test_scrape_uses_api_url(self):
        """scrape() builds API URL with query params."""
        adapter = ReverbAdapter(session=MagicMock())
        adapter._fetch_json = MagicMock(return_value={
            "listings": [],
            "current_page": 1,
            "total_pages": 1,
        })

        adapter.scrape()
        call_url = adapter._fetch_json.call_args[0][0]
        assert "reverb.com/api/listings" in call_url
        assert "product_type=electric-guitars" in call_url
        assert "per_page=24" in call_url
        assert "page=1" in call_url


# ── JSON extraction ──────────────────────────────────────────────────────


class TestExtractProducts:
    """JSON parsing and field mapping from fixture."""

    def test_extract_products_from_fixture(self, adapter, sample_json):
        """Extract all products from the sample JSON."""
        adapter.max_pages = 1
        adapter._fetch_json = MagicMock(return_value=sample_json)
        catalog = adapter.scrape()
        assert len(catalog.products) == 2, "Expected 2 products in fixture"

    def test_field_mapping_first_product(self, adapter, sample_json):
        """First product fields are correctly mapped."""
        listing = sample_json["listings"][0]
        product = adapter._map_listing(listing)

        assert product is not None
        assert product.sku == "reverb-97923167"
        assert product.name == "Fender American Professional II Stratocaster"
        assert product.brand == "Fender"
        assert product.model == "American Professional II Stratocaster"
        assert product.price == 1599.99
        assert product.currency == "USD"
        assert product.condition == "brand_new"
        assert product.availability == "in_stock"
        assert "fender-american-professional-ii-stratocaster" in product.url
        assert product.image_url == "https://rvb-img.reverb.com/fender-strat.jpg"
        assert product.seller == "Reverb Bazaar"
        assert product.location == ""

    def test_field_mapping_second_product(self, adapter, sample_json):
        """Second product fields are correctly mapped."""
        listing = sample_json["listings"][1]
        product = adapter._map_listing(listing)

        assert product is not None
        assert product.sku == "reverb-97923168"
        assert "Gibson Les Paul Standard '50s" in product.name
        assert product.brand == "Gibson"
        assert product.model == "Les Paul Standard '50s"
        assert product.price == 2499.99
        assert product.currency == "USD"
        assert product.condition == "excellent"
        assert product.availability == "in_stock"

    def test_defaults_for_missing_fields(self, adapter):
        """Listings with missing optional fields still produce valid products."""
        partial = {
            "id": 12345,
            "title": "Minimal Product",
            "price": {"amount": "99.99"},
            "listing_currency": "USD",
            "state": "live",
            "_links": {"web": {"href": "https://reverb.com/item/12345"}},
        }
        product = adapter._map_listing(partial)

        assert product is not None
        assert product.name == "Minimal Product"
        assert product.price == 99.99
        assert product.brand == ""
        assert product.model == ""
        assert product.condition == ""
        assert product.seller == ""
        assert product.location == ""
        assert product.currency == "USD"
        assert product.availability == "in_stock"
        assert product.image_url == ""

    def test_skip_listing_without_title(self, adapter):
        """Listings missing a title are skipped."""
        listing = {
            "id": 99999,
            "title": "",
            "price": {"amount": "99.99"},
            "_links": {"web": {"href": "https://reverb.com/item/99999"}},
        }
        product = adapter._map_listing(listing)
        assert product is None

    def test_empty_page_returns_empty_list(self, adapter):
        """A page with empty listings array returns empty catalog."""
        adapter._fetch_json = MagicMock(return_value={
            "listings": [],
            "current_page": 1,
            "total_pages": 5,
        })
        catalog = adapter.scrape()
        assert len(catalog.products) == 0


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

    def test_json_price_amount(self):
        """Direct JSON price amount string without currency symbol."""
        assert ReverbAdapter._parse_price("1599.99") == 1599.99


# ── SKU generation ───────────────────────────────────────────────────────


class TestSkuGeneration:
    """SKU extraction from listing id."""

    def test_sku_from_id(self):
        sku = ReverbAdapter._extract_sku("97923167", "Fender Stratocaster")
        assert sku == "reverb-97923167"

    def test_sku_with_numeric_id(self):
        sku = ReverbAdapter._extract_sku("42", "Gibson Les Paul")
        assert sku == "reverb-42"


# ── Pagination ─────────────────────────────────────────────────────────────


class TestPagination:
    """Page-stop logic based on current_page and total_pages."""

    def test_stops_when_current_page_equals_total_pages(self, adapter):
        adapter._fetch_json = MagicMock(return_value={
            "listings": [{"id": 1, "title": "Test", "price": {"amount": "1"}, "_links": {"web": {"href": "https://example.com"}}}],
            "current_page": 1,
            "total_pages": 1,
        })
        catalog = adapter.scrape()
        assert len(catalog.products) == 1
        assert adapter._fetch_json.call_count == 1

    def test_stops_when_current_page_gte_total_pages(self, adapter):
        adapter._fetch_json = MagicMock(return_value={
            "listings": [{"id": 1, "title": "Test", "price": {"amount": "1"}, "_links": {"web": {"href": "https://example.com"}}}],
            "current_page": 3,
            "total_pages": 2,
        })
        catalog = adapter.scrape()
        assert len(catalog.products) == 1
        assert adapter._fetch_json.call_count == 1

    def test_continues_to_next_page(self, adapter):
        adapter._fetch_json = MagicMock(side_effect=[
            {
                "listings": [{"id": 1, "title": "Page1", "price": {"amount": "1"}, "_links": {"web": {"href": "https://example.com"}}}],
                "current_page": 1,
                "total_pages": 2,
            },
            {
                "listings": [{"id": 2, "title": "Page2", "price": {"amount": "2"}, "_links": {"web": {"href": "https://example.com"}}}],
                "current_page": 2,
                "total_pages": 2,
            },
        ])
        catalog = adapter.scrape()
        assert len(catalog.products) == 2
        assert adapter._fetch_json.call_count == 2

    def test_respects_max_pages(self, adapter):
        adapter.max_pages = 2
        adapter._fetch_json = MagicMock(side_effect=[
            {
                "listings": [{"id": i, "title": f"Page{i}", "price": {"amount": str(i)}, "_links": {"web": {"href": "https://example.com"}}}],
                "current_page": i,
                "total_pages": 5,
            }
            for i in range(1, 6)
        ])
        catalog = adapter.scrape()
        assert len(catalog.products) == 2
        assert adapter._fetch_json.call_count == 2


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

    def test_invalid_json_raises_parse_error(self):
        """Invalid JSON response should raise ParseError."""
        adapter = ReverbAdapter(session=MagicMock())
        mock_response = MagicMock(spec=requests.Response)
        mock_response.status_code = 200
        mock_response.text = "not json"
        adapter.session.get.return_value = mock_response

        with pytest.raises(Exception) as exc_info:
            adapter._fetch_json("https://reverb.com/api/listings")
        # json.JSONDecodeError is wrapped in ParseError
        assert "Invalid JSON" in str(exc_info.value)


# ── Session creation ─────────────────────────────────────────────────────


class TestSession:
    """Adapter session configuration."""

    def test_session_has_user_agent(self):
        """Session includes a descriptive User-Agent header."""
        session = ReverbAdapter._build_session()
        ua = session.headers.get("User-Agent", "")
        assert "GuitarHub-Scraper" in ua
        assert "github.com" in ua

    def test_session_accepts_json(self):
        """Session Accept header prefers JSON."""
        session = ReverbAdapter._build_session()
        accept = session.headers.get("Accept", "")
        assert "application/json" in accept

    def test_session_has_retry_adapter(self):
        """Session mounts HTTPAdapter with retry config, or uses curl_cffi."""
        session = ReverbAdapter._build_session()
        ua = session.headers.get("User-Agent", "")
        assert "GuitarHub-Scraper" in ua
        assert "github.com" in ua
        try:
            import curl_cffi.requests as curl_requests

            assert isinstance(session, curl_requests.Session)
        except ImportError:
            adapter = session.get_adapter("https://reverb.com")
            assert isinstance(adapter, requests.adapters.HTTPAdapter)
