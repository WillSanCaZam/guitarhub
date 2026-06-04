"""Unit tests for scraper domain models."""

from datetime import datetime, timezone

import pytest
from pydantic import ValidationError

from scraper.domain import CatalogFile, CatalogProduct


class TestCatalogProduct:
    """CatalogProduct field mapping and validation."""

    def test_create_with_all_fields(self):
        """All fields populated correctly."""
        product = CatalogProduct(
            sku="reverb-abc123",
            name="Fender American Professional II Stratocaster",
            brand="Fender",
            model="American Professional II",
            category="Electric Guitars",
            subcategory="Solid Body",
            price=1599.99,
            currency="USD",
            condition="new",
            availability="in_stock",
            url="https://reverb.com/item/abc123-fender-strat",
            image_url="https://images.reverb.com/test.jpg",
            specs_json='{"body_wood": "alder"}',
            seller="Reverb Bazaar",
            location="Austin, TX",
        )
        assert product.sku == "reverb-abc123"
        assert product.name == "Fender American Professional II Stratocaster"
        assert product.price == 1599.99
        assert product.condition == "new"
        assert product.specs_json == '{"body_wood": "alder"}'

    def test_defaults_for_optional_fields(self):
        """Optional fields get sensible defaults."""
        product = CatalogProduct(
            sku="reverb-test",
            name="Test Guitar",
            price=999.0,
            url="https://reverb.com/item/test",
        )
        assert product.brand == ""
        assert product.model == ""
        assert product.currency == "USD"
        assert product.availability == "in_stock"
        assert product.specs_json == "{}"
        assert product.image_url == ""
        assert product.seller == ""
        assert product.location == ""

    def test_sku_is_required(self):
        """sku is a required field."""
        with pytest.raises(ValidationError):
            CatalogProduct(
                name="No SKU",
                price=100.0,
                url="https://reverb.com/item/no-sku",
            )

    def test_name_is_required(self):
        """name is a required field."""
        with pytest.raises(ValidationError):
            CatalogProduct(
                sku="reverb-test",
                price=100.0,
                url="https://reverb.com/item/test",
            )

    def test_price_is_required(self):
        """price is a required field."""
        with pytest.raises(ValidationError):
            CatalogProduct(
                sku="reverb-test",
                name="No Price",
                url="https://reverb.com/item/no-price",
            )

    def test_url_is_required(self):
        """url is a required field."""
        with pytest.raises(ValidationError):
            CatalogProduct(
                sku="reverb-test",
                name="No URL",
                price=100.0,
            )

    def test_serialization_round_trip(self):
        """Serializes to dict and back without data loss."""
        original = CatalogProduct(
            sku="reverb-xyz",
            name="Gibson Les Paul Standard",
            brand="Gibson",
            model="Les Paul Standard",
            category="Electric Guitars",
            subcategory="Solid Body",
            price=2499.99,
            currency="USD",
            condition="used",
            availability="in_stock",
            url="https://reverb.com/item/xyz-gibson",
            image_url="https://images.reverb.com/gibson.jpg",
            specs_json='{"finish": "cherry burst"}',
            seller="Guitar Center",
            location="Nashville, TN",
        )
        data = original.model_dump()
        restored = CatalogProduct.model_validate(data)
        assert restored == original
        assert restored.price == 2499.99


class TestCatalogFile:
    """CatalogFile envelope creation and validation."""

    def test_create_envelope_with_products(self):
        """Factory produces valid envelope with auto fields."""
        products = [
            CatalogProduct(
                sku="reverb-001",
                name="Product One",
                price=499.0,
                url="https://reverb.com/item/001",
            ),
            CatalogProduct(
                sku="reverb-002",
                name="Product Two",
                price=799.0,
                url="https://reverb.com/item/002",
            ),
        ]
        catalog = CatalogFile.create(source_id="reverb", products=products)

        assert catalog.schema_version == "1.0"
        assert catalog.source_id == "reverb"
        assert len(catalog.products) == 2
        assert catalog.generated_at != ""
        assert catalog.run_id.startswith("reverb-")

    def test_run_id_format(self):
        """run_id follows source_id-YYYYMMDD-HHMMSS format."""
        catalog = CatalogFile.create(source_id="reverb", products=[])
        parts = catalog.run_id.split("-")
        assert parts[0] == "reverb"
        assert len(parts) == 3
        assert len(parts[1]) == 8  # YYYYMMDD
        assert len(parts[2]) == 6  # HHMMSS

    def test_generated_at_is_utc_iso(self):
        """generated_at is valid UTC ISO 8601."""
        catalog = CatalogFile.create(source_id="test", products=[])
        parsed = datetime.fromisoformat(catalog.generated_at.replace("Z", "+00:00"))
        assert parsed.tzinfo is not None
        assert parsed.tzinfo.utcoffset(parsed) == timezone.utc.utcoffset(parsed)

    def test_empty_products_list(self):
        """Empty products list is valid."""
        catalog = CatalogFile.create(source_id="empty-test", products=[])
        assert len(catalog.products) == 0
        assert catalog.source_id == "empty-test"

    def test_serialization_to_json(self):
        """Serializes to JSON and back correctly."""
        products = [
            CatalogProduct(
                sku="reverb-001",
                name="Test Product",
                price=100.0,
                url="https://reverb.com/item/001",
            )
        ]
        catalog = CatalogFile.create(source_id="reverb", products=products)

        json_str = catalog.model_dump_json(indent=2)
        restored = CatalogFile.model_validate_json(json_str)

        assert restored.source_id == "reverb"
        assert len(restored.products) == 1
        assert restored.products[0].sku == "reverb-001"
        assert restored.products[0].name == "Test Product"
        assert restored.products[0].price == 100.0

    def test_json_matches_rust_rawproduct_schema(self):
        """Output JSON fields match Rust's RawProduct struct.

        This test verifies the JSON key names match the Rust
        serde-derived deserialization expectations.
        """
        product = CatalogProduct(
            sku="reverb-match",
            name="Schema Match Guitar",
            brand="TestBrand",
            model="TM-100",
            category="Electric Guitars",
            subcategory="Solid Body",
            price=999.99,
            currency="USD",
            condition="new",
            availability="in_stock",
            url="https://reverb.com/item/match",
            image_url="https://images.reverb.com/match.jpg",
            specs_json="{}",
            seller="Test Seller",
            location="USA",
        )
        data = product.model_dump()

        # These keys MUST match Rust RawProduct field names
        expected_keys = {
            "sku", "name", "brand", "model", "category", "subcategory",
            "price", "currency", "condition", "availability",
            "url", "image_url", "specs_json", "seller", "location",
        }
        assert set(data.keys()) == expected_keys, (
            f"JSON keys {set(data.keys())} do not match "
            f"expected Rust RawProduct fields {expected_keys}"
        )
