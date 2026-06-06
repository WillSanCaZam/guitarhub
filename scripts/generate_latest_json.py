#!/usr/bin/env python3
"""Generate a Tauri updater latest.json stub from a version tag."""

import json
import sys
import datetime


def main() -> None:
    if len(sys.argv) < 2:
        print("Usage: generate_latest_json.py <version>", file=sys.stderr)
        sys.exit(1)

    version = sys.argv[1]
    # Strip a leading 'v' if present so the JSON is semver-clean
    if version.startswith("v"):
        version = version[1:]

    payload = {
        "version": version,
        "notes": "",
        "pub_date": datetime.datetime.now(datetime.timezone.utc)
        .replace(microsecond=0)
        .isoformat()
        .replace("+00:00", "Z"),
        "platforms": {
            "linux-x86_64": {"signature": "", "url": ""},
            "windows-x86_64": {"signature": "", "url": ""},
            "darwin-x86_64": {"signature": "", "url": ""},
            "darwin-aarch64": {"signature": "", "url": ""},
        },
    }

    with open("latest.json", "w") as f:
        json.dump(payload, f, indent=2)
        f.write("\n")


if __name__ == "__main__":
    main()
