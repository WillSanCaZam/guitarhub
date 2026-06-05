# SPDX-License-Identifier: GPL-3.0-or-later

"""Contract tests: verify adapter conforms to ScraperPort protocol.

These tests verify at runtime that adapters satisfy the ScraperPort
protocol interface. Static compliance is enforced by mypy --strict.
"""

from scraper.adapters.reverb import ReverbAdapter
from scraper.ports import ScraperPort


class TestProtocolConformance:
    """ReverbAdapter conforms to ScraperPort protocol."""

    def test_reverb_adapter_is_scraper_port(self):
        """ReverbAdapter satisfies ScraperPort via runtime_checkable."""
        adapter = ReverbAdapter()
        assert isinstance(adapter, ScraperPort), (
            "ReverbAdapter must conform to ScraperPort protocol"
        )

    def test_adapter_has_scrape_method(self):
        """All adapters expose a callable scrape method."""
        adapter = ReverbAdapter()
        assert hasattr(adapter, "scrape")
        assert callable(adapter.scrape)

    def test_scrape_returns_catalog_file(self):
        """scrape() signature accepts optional url and returns CatalogFile.

        We verify the return type annotation without making network calls
        by checking the method's type hints.
        """
        import inspect

        sig = inspect.signature(ReverbAdapter.scrape)
        # scrape(self, url: str = "") -> CatalogFile
        params = list(sig.parameters.values())
        assert len(params) >= 2  # self + url
        url_param = params[1]
        assert url_param.name == "url"
        assert url_param.default == ""
        # Return annotation should mention CatalogFile
        return_str = str(sig.return_annotation)
        assert "CatalogFile" in return_str, (
            f"Expected CatalogFile return annotation, got {return_str}"
        )

    def test_adapter_has_required_fields(self):
        """Adapter exposes source_id for provenance."""
        adapter = ReverbAdapter()
        assert hasattr(adapter, "source_id")
        assert adapter.source_id == "reverb"
