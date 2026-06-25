# SPDX-License-Identifier: GPL-3.0-or-later

"""Contract tests: verify adapter conforms to ScraperPort protocol.

These tests verify at runtime that adapters satisfy the ScraperPort
protocol interface. Static compliance is enforced by mypy --strict.
"""

from scraper.adapters.guitarcenter import GuitarCenterAdapter
from scraper.adapters.reverb import ReverbAdapter
from scraper.ports import ScraperPort


class TestProtocolConformance:
    """Adapters conform to ScraperPort protocol."""

    def test_reverb_adapter_is_scraper_port(self) -> None:
        """ReverbAdapter satisfies ScraperPort via runtime_checkable."""
        adapter = ReverbAdapter()
        assert isinstance(adapter, ScraperPort), (
            "ReverbAdapter must conform to ScraperPort protocol"
        )

    def test_guitarcenter_adapter_is_scraper_port(self) -> None:
        """GuitarCenterAdapter satisfies ScraperPort via runtime_checkable."""
        adapter = GuitarCenterAdapter(
            algolia_app_id="test",
            algolia_api_key="test",
        )
        assert isinstance(adapter, ScraperPort), (
            "GuitarCenterAdapter must conform to ScraperPort protocol"
        )

    def test_adapter_has_scrape_method(self) -> None:
        """All adapters expose a callable scrape method."""
        for adapter_cls in [ReverbAdapter, GuitarCenterAdapter]:
            kwargs = {}
            if adapter_cls is GuitarCenterAdapter:
                kwargs = {"algolia_app_id": "test", "algolia_api_key": "test"}
            adapter = adapter_cls(**kwargs)
            assert hasattr(adapter, "scrape")
            assert callable(adapter.scrape)

    def test_scrape_returns_catalog_file(self) -> None:
        """scrape() signature accepts optional url and returns CatalogFile.

        We verify the return type annotation without making network calls
        by checking the method's type hints.
        """
        import inspect

        for adapter_cls in [ReverbAdapter, GuitarCenterAdapter]:
            sig = inspect.signature(adapter_cls.scrape)
            params = list(sig.parameters.values())
            assert len(params) >= 2  # self + url
            url_param = params[1]
            assert url_param.name == "url"
            assert url_param.default == ""
            return_str = str(sig.return_annotation)
            assert "CatalogFile" in return_str, (
                f"Expected CatalogFile return annotation, got {return_str}"
            )

    def test_adapter_has_required_fields(self) -> None:
        """Adapter exposes source_id for provenance."""
        for adapter_cls in [ReverbAdapter, GuitarCenterAdapter]:
            kwargs = {}
            if adapter_cls is GuitarCenterAdapter:
                kwargs = {"algolia_app_id": "test", "algolia_api_key": "test"}
            adapter = adapter_cls(**kwargs)
            assert hasattr(adapter, "source_id")
            assert isinstance(adapter.source_id, str)
