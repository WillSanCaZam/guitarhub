# SPDX-License-Identifier: GPL-3.0-or-later

"""CLI entry point for the GuitarHub catalog scraper.

Usage:
    python -m scraper --adapter reverb --output catalog.json [--validate]
"""

import argparse
import json
import logging
import sys
from pathlib import Path

from scraper.domain import CatalogFile


def main() -> int:
    """Run the scraper CLI. Returns exit code (0 success, 1 failure)."""
    parser = argparse.ArgumentParser(
        description="GuitarHub catalog scraper — Ports & Adapters pattern"
    )
    parser.add_argument(
        "--adapter",
        choices=["reverb"],
        help="Scraper adapter to use",
    )
    parser.add_argument(
        "--output",
        help="Output JSON file path (use '-' for stdout)",
    )
    parser.add_argument(
        "--validate",
        action="store_true",
        help="Re-validate output against schema after writing",
    )
    parser.add_argument(
        "--validate-input",
        action="store_true",
        dest="validate_input",
        help="Validate existing JSON files in --input-dir against CatalogFile schema",
    )
    parser.add_argument(
        "--input-dir",
        default=".",
        dest="input_dir",
        help="Directory to scan for JSON files when --validate-input is used (default: current directory)",
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

    logger = logging.getLogger(__name__)

    # ── Validate-input mode (idempotent, read-only) ─────────────────
    if args.validate_input:
        return _validate_input_dir(args.input_dir, logger)

    # ── Normal scrape mode requires adapter and output ────────────────
    if not args.adapter or not args.output:
        parser.error("--adapter and --output are required unless --validate-input is used")

    # ── Select and instantiate adapter ──────────────────────────────
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


def _validate_input_dir(input_dir: str, logger: logging.Logger) -> int:
    """Validate all JSON files in input_dir against CatalogFile schema.

    Idempotent and read-only: never modifies files.
    Returns 0 if all files are valid, 1 if any are invalid.
    """
    path = Path(input_dir)
    if not path.exists():
        logger.error("Input directory does not exist: %s", input_dir)
        return 1
    if not path.is_dir():
        logger.error("Input path is not a directory: %s", input_dir)
        return 1

    json_files = sorted(path.glob("*.json"))
    if not json_files:
        logger.info("No .json files found in %s", input_dir)
        return 0

    logger.info("Validating %d JSON file(s) in %s", len(json_files), input_dir)
    all_valid = True

    for file_path in json_files:
        try:
            raw = file_path.read_text(encoding="utf-8")
            CatalogFile.model_validate_json(raw)
            logger.info("VALID: %s", file_path.name)
        except Exception as exc:
            logger.error("INVALID: %s — %s", file_path.name, exc)
            all_valid = False

    if all_valid:
        logger.info("All %d file(s) passed schema validation", len(json_files))
        return 0
    else:
        logger.error("Schema validation failed for one or more files")
        return 1


if __name__ == "__main__":
    sys.exit(main())
