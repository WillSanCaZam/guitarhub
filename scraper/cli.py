# SPDX-License-Identifier: GPL-3.0-or-later

"""CLI entry point for the GuitarHub catalog scraper.

Usage:
    python -m scraper --adapter reverb --output catalog.json [--validate]
"""

import argparse
import json
import logging
import sys

from scraper.domain import CatalogFile


def main() -> int:
    """Run the scraper CLI. Returns exit code (0 success, 1 failure)."""
    parser = argparse.ArgumentParser(
        description="GuitarHub catalog scraper — Ports & Adapters pattern"
    )
    parser.add_argument(
        "--adapter",
        required=True,
        choices=["reverb"],
        help="Scraper adapter to use",
    )
    parser.add_argument(
        "--output",
        required=True,
        help="Output JSON file path (use '-' for stdout)",
    )
    parser.add_argument(
        "--validate",
        action="store_true",
        help="Re-validate output against schema after writing",
    )
    parser.add_argument(
        "--verbose",
        "-v",
        action="store_true",
        help="Enable debug logging to stderr",
    )
    args = parser.parse_args()

    logging.basicConfig(
        level=logging.DEBUG if args.verbose else logging.INFO,
        format="%(levelname)s | %(name)s | %(message)s",
        stream=sys.stderr,
    )

    # ── Select and instantiate adapter ──────────────────────────────
    logger = logging.getLogger(__name__)

    if args.adapter == "reverb":
        from scraper.adapters.reverb import ReverbAdapter

        adapter = ReverbAdapter()
    else:
        logger.error("Unknown adapter: %s", args.adapter)
        return 1

    # ── Run scraper ─────────────────────────────────────────────────
    logger.info("Running adapter=%s output=%s", args.adapter, args.output)

    try:
        catalog: CatalogFile = adapter.scrape()
    except Exception as exc:
        logger.error("Scraper failed: %s", exc)
        if args.verbose:
            logger.exception("Full traceback:")
        return 1

    # ── Serialize ───────────────────────────────────────────────────
    try:
        json_str = catalog.model_dump_json(indent=2)
    except Exception as exc:
        logger.error("Serialization failed: %s", exc)
        return 1

    # ── Write output ────────────────────────────────────────────────
    try:
        if args.output == "-":
            print(json_str)
        else:
            with open(args.output, "w", encoding="utf-8") as f:
                f.write(json_str)
            logger.info("Wrote %d products to %s", len(catalog.products), args.output)
    except OSError as exc:
        logger.error("Failed to write output: %s", exc)
        return 1

    # ── Validate ────────────────────────────────────────────────────
    if args.validate:
        logger.info("Validating output schema...")
        try:
            with open(args.output, "r", encoding="utf-8") as f:
                raw = f.read()
            CatalogFile.model_validate_json(raw)
            logger.info("Schema validation: PASSED")
        except Exception as exc:
            logger.error("Schema validation: FAILED — %s", exc)
            return 1

    return 0


if __name__ == "__main__":
    sys.exit(main())
