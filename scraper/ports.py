"""Ports for the scraper — Ports & Adapters pattern."""

from typing import Protocol, runtime_checkable

from scraper.domain import CatalogFile


class ScraperError(Exception):
    """Base error for scraper operations."""
    pass


class FetchError(ScraperError):
    """Error during HTTP fetch (timeout, connection, HTTP status)."""
    pass


class ParseError(ScraperError):
    """Error during HTML parsing."""
    pass


@runtime_checkable
class ScraperPort(Protocol):
    """Protocol that all scraper adapters must implement.

    Each adapter fetches products from a source and returns a
    complete CatalogFile matching the Rust CatalogFile schema.
    """

    def scrape(self, url: str = "") -> CatalogFile:
        """Fetch and parse products from the source.

        Args:
            url: Optional override URL. Adapters may provide a default.

        Returns:
            CatalogFile with all scraped products.

        Raises:
            FetchError: on HTTP or network failure.
            ParseError: on HTML/data parse failure.
        """
        ...
