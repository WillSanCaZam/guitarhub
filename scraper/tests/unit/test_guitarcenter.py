# SPDX-License-Identifier: GPL-3.0-or-later

"""Unit tests for GuitarCenterAdapter — Algolia API extraction and mapping."""

import json
import os
from pathlib import Path
from unittest.mock import MagicMock, patch

import pytest
import requests

from scraper.adapters.guitarcenter import GuitarCenterAdapter
from scraper.ports import FetchError, ParseError

FIXTURES_DIR = Path(__file__).parent.parent / "fixtures"


# ── Fixtures ─────────────────────────────────────────────────────────────


@pytest.fixture
def adapter() -> GuitarCenterAdapter:
    """Return a GuitarCenterAdapter with stub credentials and no real HTTP."""
    return GuitarCenterAdapter(
        source_id="guitarcenter",
        session=MagicMock(),
        algolia_app_id="test-app-id",
        algolia_api_key="test-api-key",
    )


@pytest.fixture
def sample_hits() -> list[dict]:
    """Load the sample Guitar Center Algolia fixture."""
    path = FIXTURES_DIR / "guitarcenter-sample.json"
    data = json.loads(path.read_text(encoding="utf-8"))
    return data.get("hits", [])


@pytest.fixture
def sample_json() -> dict:
    """Load the full sample Algolia response."""
    path = FIXTURES_DIR / "guitarcenter-sample.json"
    return json.loads(path.read_text(encoding="utf-8"))


# ── Credential resolution ─────────────────────────────────────────────────


class TestCredentials:
    """Adapter credential resolution from constructor args and env vars."""

    def test_constructor_credentials_used(self) -> None:
        """Explicit constructor args take priority."""
        adapter = GuitarCenterAdapter(
            algolia_app_id="my-app",
            algolia_api_key="my-key",
            session=MagicMock(),
        )
        assert adapter.algolia_app_id == "my-app"
        assert adapter.algolia_api_key == "my-key"

    def test_env_var_credentials_fallback(self) -> None:
        """Env vars are used when constructor args are not provided."""
        with patch.dict(
            os.environ,
            {"GC_ALGOLIA_APP_ID": "env-app", "GC_ALGOLIA_API_KEY": "env-key"},
        ):
            adapter = GuitarCenterAdapter(session=MagicMock())
        assert adapter.algolia_app_id == "env-app"
        assert adapter.algolia_api_key == "env-key"

    def test_missing_credentials_raises_value_error(self) -> None:
        """Missing credentials raise ValueError listing missing vars."""
        with patch.dict(os.environ, {}, clear=True):
            with pytest.raises(ValueError) as exc_info:
                GuitarCenterAdapter(session=MagicMock())
        msg = str(exc_info.value)
        assert "GC_ALGOLIA_APP_ID" in msg
        assert "GC_ALGOLIA_API_KEY" in msg

    def test_missing_app_id_raises_value_error(self) -> None:
        """Missing only GC_ALGOLIA_APP_ID raises ValueError."""
        with patch.dict(
            os.environ,
            {"GC_ALGOLIA_API_KEY": "key-only"},
            clear=True,
        ):
            with pytest.raises(ValueError) as exc_info:
                GuitarCenterAdapter(session=MagicMock())
        assert "GC_ALGOLIA_APP_ID" in str(exc_info.value)

    def test_missing_api_key_raises_value_error(self) -> None:
        """Missing only GC_ALGOLIA_API_KEY raises ValueError."""
        with patch.dict(
            os.environ,
            {"GC_ALGOLIA_APP_ID": "app-only"},
            clear=True,
        ):
            with pytest.raises(ValueError) as exc_info:
                GuitarCenterAdapter(session=MagicMock())
        assert "GC_ALGOLIA_API_KEY" in str(exc_info.value)


# ── Field mapping ─────────────────────────────────────────────────────────


class TestFieldMapping:
    """Algolia hit → CatalogProduct mapping."""

    def test_full_hit_maps_all_fields(self, adapter, sample_hits) -> None:
        """A complete hit with all fields populates every CatalogProduct field."""
        hit = sample_hits[0]
        product = adapter._map_hit(hit)

        assert product is not None
        assert product.sku == "gc-LIVESTRAT-001"
        assert product.name == "Fender American Professional II Stratocaster"
        assert product.brand == "Fender"
        assert product.price == 1599.99
        assert product.currency == "USD"
        assert "fender-american-professional-ii-stratocaster" in product.url
        assert product.image_url == (
            "https://media.guitarcenter.com/is/image/MMGS7/FENAP2STRAT001"
        )

    def test_missing_brand_defaults_to_unknown(self, adapter, sample_hits) -> None:
        """A hit with null brand defaults to 'Unknown'."""
        hit = sample_hits[9]
        product = adapter._map_hit(hit)
        assert product is not None
        assert product.brand == "Unknown"

    def test_missing_image_defaults_to_empty(self, adapter, sample_hits) -> None:
        """A hit with null imageId gets empty image_url."""
        hit = sample_hits[10]
        product = adapter._map_hit(hit)
        assert product is not None
        assert product.image_url == ""

    def test_sku_prefix_is_gc(self, adapter) -> None:
        """SKU is prefixed with gc-."""
        hit = {
            "display_name": "Test",
            "product_id": "TEST123",
            "identifiers": {"gcItemNumber": "TEST123"},
            "current_price": 100.0,
            "seoUrl": "/test",
        }
        product = adapter._map_hit(hit)
        assert product is not None
        assert product.sku == "gc-TEST123"

    def test_missing_product_id_uses_empty_sku(self, adapter) -> None:
        """When product_id is missing, SKU falls back to empty prefix."""
        hit = {
            "display_name": "Test",
            "current_price": 100.0,
            "seoUrl": "/test",
        }
        product = adapter._map_hit(hit)
        assert product is not None
        assert product.sku == "gc-"

    def test_price_from_current_price(self, adapter) -> None:
        """price field from current_price."""
        hit = {
            "display_name": "Test",
            "product_id": "T1",
            "current_price": 599.99,
            "seoUrl": "/test",
        }
        product = adapter._map_hit(hit)
        assert product is not None
        assert product.price == 599.99

    def test_image_url_constructed_correctly(self, adapter) -> None:
        """image_url is constructed by prepending the base URL to imageId."""
        hit = {
            "display_name": "Test",
            "product_id": "T1",
            "current_price": 100.0,
            "seoUrl": "/test",
            "imageId": "TESTIMAGE001",
        }
        product = adapter._map_hit(hit)
        assert product is not None
        assert product.image_url == (
            "https://media.guitarcenter.com/is/image/MMGS7/TESTIMAGE001"
        )

    def test_category_from_hierarchical_facets(self, adapter) -> None:
        """category and subcategory are derived from categories.* facets."""
        hit = {
            "display_name": "Test",
            "product_id": "T1",
            "current_price": 100.0,
            "seoUrl": "/test",
            "categories": {
                "lvl0": "Guitars",
                "lvl1": "Electric Guitars",
                "lvl2": "Solid Body",
            },
        }
        product = adapter._map_hit(hit)
        assert product is not None
        assert product.category == "Electric Guitars"
        assert product.subcategory == "Solid Body"

    def test_category_single_level_falls_back(self, adapter) -> None:
        """When only lvl0 exists, category = lvl0 and subcategory = ''."""
        hit = {
            "display_name": "Test",
            "product_id": "T1",
            "current_price": 100.0,
            "seoUrl": "/test",
            "categories": {"lvl0": "Guitars"},
        }
        product = adapter._map_hit(hit)
        assert product is not None
        assert product.category == "Guitars"
        assert product.subcategory == ""

    def test_missing_categories_returns_empty(self, adapter) -> None:
        """When categories is missing, category and subcategory are ''."""
        hit = {
            "display_name": "Test",
            "product_id": "T1",
            "current_price": 100.0,
            "seoUrl": "/test",
        }
        product = adapter._map_hit(hit)
        assert product is not None
        assert product.category == ""
        assert product.subcategory == ""

    def test_specs_json_includes_stickers(self, adapter, sample_hits) -> None:
        """specs_json preserves stickers and condition_original."""
        hit = sample_hits[0]
        product = adapter._map_hit(hit)
        assert product is not None
        specs = json.loads(product.specs_json)
        assert "stickers" in specs
        assert "Price Drop" in specs["stickers"]
        assert "condition_original" in specs
        assert specs["condition_original"] == "New"


# ── Condition normalization ───────────────────────────────────────────────


class TestConditionNormalization:
    """GC 9-value condition vocabulary → 4-value normalized output."""

    def test_new_condition(self, adapter, sample_hits) -> None:
        """New → condition='new'."""
        hit = sample_hits[0]
        product = adapter._map_hit(hit)
        assert product is not None
        assert product.condition == "new"

    def test_used_excellent(self, adapter, sample_hits) -> None:
        """Used > Excellent → condition='used'."""
        hit = sample_hits[1]
        product = adapter._map_hit(hit)
        assert product is not None
        assert product.condition == "used"

    def test_used_great(self, adapter, sample_hits) -> None:
        """Used > Great → condition='used'."""
        hit = sample_hits[2]
        product = adapter._map_hit(hit)
        assert product is not None
        assert product.condition == "used"

    def test_used_good(self, adapter, sample_hits) -> None:
        """Used > Good → condition='used'."""
        hit = sample_hits[3]
        product = adapter._map_hit(hit)
        assert product is not None
        assert product.condition == "used"

    def test_used_fair(self, adapter, sample_hits) -> None:
        """Used > Fair → condition='used'."""
        hit = sample_hits[4]
        product = adapter._map_hit(hit)
        assert product is not None
        assert product.condition == "used"

    def test_used_poor(self, adapter, sample_hits) -> None:
        """Used > Poor → condition='used'."""
        hit = sample_hits[5]
        product = adapter._map_hit(hit)
        assert product is not None
        assert product.condition == "used"

    def test_open_box_condition(self, adapter, sample_hits) -> None:
        """Open Box (skuCondition=3) → condition='new' + sticker open_box."""
        hit = sample_hits[6]
        product = adapter._map_hit(hit)
        assert product is not None
        assert product.condition == "new"
        specs = json.loads(product.specs_json)
        assert "open_box" in specs.get("stickers", [])

    def test_blemished_condition(self, adapter, sample_hits) -> None:
        """Blemished (skuCondition=11) → condition='new' + sticker blemished."""
        hit = sample_hits[7]
        product = adapter._map_hit(hit)
        assert product is not None
        assert product.condition == "new"
        specs = json.loads(product.specs_json)
        assert "blemished" in specs.get("stickers", [])

    def test_restock_condition(self, adapter, sample_hits) -> None:
        """Restock (skuCondition=2) → condition='refurbished' + sticker restock."""
        hit = sample_hits[8]
        product = adapter._map_hit(hit)
        assert product is not None
        assert product.condition == "refurbished"
        specs = json.loads(product.specs_json)
        assert "restock" in specs.get("stickers", [])

    def test_unknown_condition(self, adapter, sample_hits) -> None:
        """Unknown condition → condition='unknown'."""
        hit = sample_hits[13]
        product = adapter._map_hit(hit)
        assert product is not None
        assert product.condition == "unknown"

    def test_missing_condition_defaults_unknown(self, adapter, sample_hits) -> None:
        """When condition is missing entirely → condition='unknown'."""
        hit = sample_hits[14]
        product = adapter._map_hit(hit)
        assert product is not None
        assert product.condition == "unknown"

    def test_condition_original_preserved_in_specs(self, adapter, sample_hits) -> None:
        """Raw condition value is preserved in specs_json.condition_original."""
        hit = sample_hits[1]
        product = adapter._map_hit(hit)
        assert product is not None
        specs = json.loads(product.specs_json)
        assert specs.get("condition_original") == "Used > Excellent"

    def test_condition_original_for_sku_condition(self, adapter, sample_hits) -> None:
        """skuCondition-based condition preserves a readable original."""
        hit = sample_hits[6]  # Open Box
        product = adapter._map_hit(hit)
        assert product is not None
        specs = json.loads(product.specs_json)
        assert "condition_original" in specs
        assert specs["condition_original"] == "Open Box"


# ── Availability logic ────────────────────────────────────────────────────


class TestAvailability:
    """Availability from inventoryStatus + stores[]. """

    def test_in_stock_both_signals(self, adapter, sample_hits) -> None:
        """inventoryStatus=1000 AND stores non-empty → in_stock."""
        hit = sample_hits[0]
        product = adapter._map_hit(hit)
        assert product is not None
        assert product.availability == "in_stock"

    def test_in_stock_1003_with_stores(self, adapter, sample_hits) -> None:
        """inventoryStatus=1003 AND stores non-empty → in_stock."""
        hit = sample_hits[2]  # inventoryStatus=1003, stores=["Store3"]
        product = adapter._map_hit(hit)
        assert product is not None
        assert product.availability == "in_stock"

    def test_out_of_stock_empty_stores(self, adapter, sample_hits) -> None:
        """stores empty → out_of_stock (regardless of inventoryStatus)."""
        hit = sample_hits[11]  # inventoryStatus=1000, stores=[]
        product = adapter._map_hit(hit)
        assert product is not None
        assert product.availability == "out_of_stock"

    def test_out_of_stock_no_inventory_status(self, adapter, sample_hits) -> None:
        """Missing inventoryStatus → out_of_stock."""
        hit = sample_hits[12]  # no inventoryStatus, stores=["Store1"]
        product = adapter._map_hit(hit)
        assert product is not None
        assert product.availability == "out_of_stock"

    def test_out_of_stock_inventory_status_not_in_set(self, adapter) -> None:
        """inventoryStatus not in (1000, 1003) → out_of_stock."""
        hit = {
            "display_name": "Test",
            "product_id": "T1",
            "current_price": 100.0,
            "seoUrl": "/test",
            "inventoryStatus": 999,
            "stores": ["Store1"],
        }
        product = adapter._map_hit(hit)
        assert product is not None
        assert product.availability == "out_of_stock"

    def test_out_of_stock_no_stores_field(self, adapter) -> None:
        """Missing stores field → out_of_stock."""
        hit = {
            "display_name": "Test",
            "product_id": "T1",
            "current_price": 100.0,
            "seoUrl": "/test",
            "inventoryStatus": 1000,
        }
        product = adapter._map_hit(hit)
        assert product is not None
        assert product.availability == "out_of_stock"


# ── Pagination ────────────────────────────────────────────────────────────


class TestPagination:
    """Page-stop logic based on Algolia hits array and max_pages."""

    def test_single_page_stops_after_hits(self, adapter, sample_json) -> None:
        """One page of hits returns all products and stops."""
        adapter._fetch_json = MagicMock(return_value=sample_json)
        catalog = adapter.scrape()
        expected_count = len(sample_json.get("hits", []))
        assert len(catalog.products) == expected_count
        assert adapter._fetch_json.call_count == 1

    def test_stops_when_hits_empty(self, adapter) -> None:
        """Empty hits array stops pagination immediately."""
        adapter._fetch_json = MagicMock(return_value={
            "hits": [],
            "page": 0,
            "nbPages": 0,
        })
        catalog = adapter.scrape()
        assert len(catalog.products) == 0
        assert adapter._fetch_json.call_count == 1

    def test_empty_catalog_returns_valid_file(self, adapter) -> None:
        """Empty catalog returns CatalogFile with products=[]."""
        adapter._fetch_json = MagicMock(return_value={
            "hits": [],
            "page": 0,
            "nbPages": 0,
        })
        catalog = adapter.scrape()
        assert catalog.products == []
        assert catalog.source_id == "guitarcenter"

    def test_respects_max_pages(self, adapter) -> None:
        """Stops scraping when max_pages is reached."""
        adapter.max_pages = 2
        adapter._fetch_json = MagicMock(side_effect=[
            {
                "hits": [{"display_name": f"Product {i}", "product_id": str(i),
                          "current_price": float(i), "seoUrl": f"/p/{i}"}
                         for i in range(3)],
                "page": 0,
                "nbPages": 10,
            },
            {
                "hits": [{"display_name": f"Product {i}", "product_id": str(i),
                          "current_price": float(i), "seoUrl": f"/p/{i}"}
                         for i in range(3, 6)],
                "page": 1,
                "nbPages": 10,
            },
            {
                "hits": [{"display_name": f"Product {i}", "product_id": str(i),
                          "current_price": float(i), "seoUrl": f"/p/{i}"}
                         for i in range(6, 9)],
                "page": 2,
                "nbPages": 10,
            },
        ])
        catalog = adapter.scrape()
        assert len(catalog.products) == 6  # 2 pages × 3 products
        assert adapter._fetch_json.call_count == 2

    def test_page_increments_correctly(self, adapter) -> None:
        """Each request uses an incremented page parameter."""
        call_args_list: list = []

        def mock_fetch_json(page: int) -> dict:
            call_args_list.append(page)
            return {
                "hits": [{"display_name": str(page), "product_id": str(page),
                          "current_price": float(page), "seoUrl": f"/{page}"}],
                "page": page,
                "nbPages": 3,
            }

        adapter.max_pages = 3
        adapter._fetch_json = MagicMock(side_effect=mock_fetch_json)
        adapter.scrape()
        assert call_args_list == [0, 1, 2]

    def test_rate_limit_delay_applied(self, adapter) -> None:
        """Delay is applied between pagination requests."""
        adapter.max_pages = 2
        adapter.delay = 0.01
        adapter._fetch_json = MagicMock(return_value={
            "hits": [{"display_name": "Test", "product_id": "1",
                      "current_price": 1.0, "seoUrl": "/t"}],
            "page": 0,
            "nbPages": 2,
        })

        import time
        start = time.monotonic()
        adapter.scrape()
        elapsed = time.monotonic() - start
        assert elapsed >= 0.01  # at least one delay between pages

    def test_continues_across_multiple_pages(self, adapter) -> None:
        """Multiple pages of hits are all collected."""
        adapter.max_pages = 3
        adapter._fetch_json = MagicMock(side_effect=[
            {
                "hits": [{"display_name": f"A{i}", "product_id": f"A{i}",
                          "current_price": float(i), "seoUrl": f"/a{i}"}
                         for i in range(2)],
                "page": 0,
                "nbPages": 3,
            },
            {
                "hits": [{"display_name": f"B{i}", "product_id": f"B{i}",
                          "current_price": float(i) + 10, "seoUrl": f"/b{i}"}
                         for i in range(2)],
                "page": 1,
                "nbPages": 3,
            },
            {
                "hits": [],
                "page": 2,
                "nbPages": 3,
            },
        ])
        catalog = adapter.scrape()
        assert len(catalog.products) == 4  # 2 pages × 2 products; 3rd empty
        assert adapter._fetch_json.call_count == 3


# ── HTTP error handling ───────────────────────────────────────────────────


class TestHttpErrorHandling:
    """Adapter raises typed errors on HTTP failures."""

    def test_algolia_query_url(self) -> None:
        """Query URL contains the Algolia app ID and index name."""
        adapter = GuitarCenterAdapter(
            session=MagicMock(),
            algolia_app_id="testapp",
            algolia_api_key="testkey",
        )
        assert "testapp" in adapter.query_url
        assert "guitarcenter" in adapter.query_url
        assert "algolia.net" in adapter.query_url

    def test_404_raises_fetch_error(self) -> None:
        """A 404 should raise FetchError."""
        adapter = GuitarCenterAdapter(
            session=MagicMock(),
            algolia_app_id="app",
            algolia_api_key="key",
        )
        mock_response = MagicMock(spec=requests.Response)
        mock_response.status_code = 404
        mock_response.reason = "Not Found"
        adapter.session.post.return_value = mock_response

        with pytest.raises(FetchError, match="404"):
            adapter._fetch(0)

    def test_timeout_raises_fetch_error(self) -> None:
        """A timeout should raise FetchError."""
        adapter = GuitarCenterAdapter(
            session=MagicMock(),
            algolia_app_id="app",
            algolia_api_key="key",
        )
        adapter.session.post.side_effect = requests.exceptions.Timeout("timed out")

        with pytest.raises(FetchError, match="timed out"):
            adapter._fetch(0)

    def test_connection_error_raises_fetch_error(self) -> None:
        """A connection error should raise FetchError."""
        adapter = GuitarCenterAdapter(
            session=MagicMock(),
            algolia_app_id="app",
            algolia_api_key="key",
        )
        adapter.session.post.side_effect = (
            requests.exceptions.ConnectionError("refused")
        )

        with pytest.raises(FetchError, match="refused"):
            adapter._fetch(0)

    def test_400_raises_fetch_error(self) -> None:
        """A 400 should raise FetchError."""
        adapter = GuitarCenterAdapter(
            session=MagicMock(),
            algolia_app_id="app",
            algolia_api_key="key",
        )
        mock_response = MagicMock(spec=requests.Response)
        mock_response.status_code = 400
        mock_response.reason = "Bad Request"
        adapter.session.post.return_value = mock_response

        with pytest.raises(FetchError, match="400"):
            adapter._fetch(0)

    def test_invalid_json_raises_parse_error(self) -> None:
        """Invalid JSON response should raise ParseError."""
        adapter = GuitarCenterAdapter(
            session=MagicMock(),
            algolia_app_id="app",
            algolia_api_key="key",
        )
        mock_response = MagicMock(spec=requests.Response)
        mock_response.status_code = 200
        mock_response.text = "not json"
        adapter.session.post.return_value = mock_response

        with pytest.raises(ParseError, match="Invalid JSON"):
            adapter._fetch_json(0)


# ── Constructor ───────────────────────────────────────────────────────────


class TestConstructor:
    """Adapter construction and configuration."""

    def test_default_source_id(self) -> None:
        """Default source_id is 'guitarcenter'."""
        adapter = GuitarCenterAdapter(
            session=MagicMock(),
            algolia_app_id="app",
            algolia_api_key="key",
        )
        assert adapter.source_id == "guitarcenter"

    def test_custom_source_id(self) -> None:
        """source_id can be overridden."""
        adapter = GuitarCenterAdapter(
            source_id="gc-custom",
            session=MagicMock(),
            algolia_app_id="app",
            algolia_api_key="key",
        )
        assert adapter.source_id == "gc-custom"

    def test_default_max_pages(self) -> None:
        """Default max_pages is 50."""
        adapter = GuitarCenterAdapter(
            session=MagicMock(),
            algolia_app_id="app",
            algolia_api_key="key",
        )
        assert adapter.max_pages == 50

    def test_custom_max_pages(self) -> None:
        """max_pages can be overridden."""
        adapter = GuitarCenterAdapter(
            max_pages=10,
            session=MagicMock(),
            algolia_app_id="app",
            algolia_api_key="key",
        )
        assert adapter.max_pages == 10


# ── Specs_json stickers ───────────────────────────────────────────────────


class TestSpecsJson:
    """specs_json field construction with stickers."""

    def test_preserves_original_stickers_from_hit(self, adapter, sample_hits) -> None:
        """Original sticker array from hit is preserved in specs_json."""
        hit = sample_hits[0]  # Has "Price Drop" sticker
        product = adapter._map_hit(hit)
        assert product is not None
        specs = json.loads(product.specs_json)
        assert "Price Drop" in specs.get("stickers", [])

    def test_vintage_sticker_preserved(self, adapter, sample_hits) -> None:
        """Vintage sticker from hit is preserved."""
        hit = sample_hits[3]  # Has "Vintage" sticker
        product = adapter._map_hit(hit)
        assert product is not None
        specs = json.loads(product.specs_json)
        assert "Vintage" in specs.get("stickers", [])

    def test_empty_stickers(self, adapter, sample_hits) -> None:
        """When no sticker data, stickers list is empty."""
        hit = sample_hits[5]
        product = adapter._map_hit(hit)
        assert product is not None
        specs = json.loads(product.specs_json)
        assert specs.get("stickers") == []

    def test_condition_stickers_combined_with_original(
        self, adapter, sample_hits
    ) -> None:
        """Condition stickers merge with original stickers."""
        hit = sample_hits[6]  # Open Box with "Price Drop" sticker
        product = adapter._map_hit(hit)
        assert product is not None
        specs = json.loads(product.specs_json)
        stickers = specs.get("stickers", [])
        assert "open_box" in stickers
        assert "Price Drop" in stickers

    def test_multiple_stickers(self, adapter) -> None:
        """Multiple original stickers are all preserved."""
        hit = {
            "display_name": "Multi Sticker",
            "product_id": "MS1",
            "current_price": 100.0,
            "seoUrl": "/ms1",
            "sticker": ["Price Drop", "Vintage", "Limited Edition"],
            "inventoryStatus": 1000,
            "stores": ["Store1"],
        }
        product = adapter._map_hit(hit)
        assert product is not None
        specs = json.loads(product.specs_json)
        stickers = specs.get("stickers", [])
        assert "Price Drop" in stickers
        assert "Vintage" in stickers
        assert "Limited Edition" in stickers

    def test_sku_condition_sticker_for_restock(self, adapter, sample_hits) -> None:
        """Restock adds 'restock' sticker and preserves other stickers."""
        hit = sample_hits[8]  # Restock item
        product = adapter._map_hit(hit)
        assert product is not None
        specs = json.loads(product.specs_json)
        stickers = specs.get("stickers", [])
        assert "restock" in stickers

    def test_open_box_from_sku_condition(self, adapter, sample_hits) -> None:
        """B-Stock Open Box adds 'open_box' sticker."""
        hit = sample_hits[15]  # Second Open Box item
        product = adapter._map_hit(hit)
        assert product is not None
        specs = json.loads(product.specs_json)
        stickers = specs.get("stickers", [])
        assert "open_box" in stickers
        assert "Price Drop" in stickers  # original sticker preserved
