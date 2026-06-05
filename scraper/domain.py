# SPDX-License-Identifier: GPL-3.0-or-later

"""Domain models for catalog products matching Rust RawProduct schema."""

from datetime import datetime, timezone
from pydantic import BaseModel


class CatalogProduct(BaseModel):
    """A single product listing matching Rust's RawProduct struct."""

    sku: str
    name: str
    brand: str = ""
    model: str = ""
    category: str = ""
    subcategory: str = ""
    price: float
    currency: str = "USD"
    condition: str = ""
    availability: str = "in_stock"
    url: str
    image_url: str = ""
    specs_json: str = "{}"
    seller: str = ""
    location: str = ""


class CatalogFile(BaseModel):
    """Top-level output envelope matching Rust's CatalogFile struct."""

    schema_version: str = "1.0"
    source_id: str
    generated_at: str = ""
    run_id: str = ""
    products: list[CatalogProduct] = []

    @classmethod
    def create(
        cls,
        source_id: str,
        products: list[CatalogProduct],
        schema_version: str = "1.0",
    ) -> "CatalogFile":
        """Factory: auto-populates generated_at and run_id."""
        now = datetime.now(timezone.utc)
        return cls(
            schema_version=schema_version,
            source_id=source_id,
            generated_at=now.strftime("%Y-%m-%dT%H:%M:%SZ"),
            run_id=f"{source_id}-{now.strftime('%Y%m%d-%H%M%S')}",
            products=products,
        )
