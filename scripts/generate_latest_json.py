#!/usr/bin/env python3
"""Generate a Tauri updater latest.json with real URLs and signatures.

Usage: generate_latest_json.py <version_tag> <sig1> [<sig2> ...]

Each signature file is a .sig file produced by `tauri signer sign`.
The filename determines which platform it belongs to:

  *sig-linux-x86_64* or *x86_64-unknown-linux-gnu*  → linux-x86_64
  *darwin-aarch64* or *aarch64-apple-darwin*          → darwin-aarch64
  *darwin-x86_64*  or *x86_64-apple-darwin*           → darwin-x86_64
"""

import json
import sys
import datetime
import os
import glob


OWNER_REPO = "WillSanCaZam/guitarhub"

PLATFORM_MAP = {
    "linux-x86_64": {
        "suffixes": ["linux-x86_64", "x86_64-unknown-linux-gnu"],
        "asset_template": "guitarhub_{version}_amd64.deb",
    },
    "darwin-aarch64": {
        "suffixes": ["darwin-aarch64", "aarch64-apple-darwin"],
        "asset_template": "guitarhub_{version}_aarch64.dmg",
    },
    "darwin-x86_64": {
        "suffixes": ["darwin-x86_64", "x86_64-apple-darwin"],
        "asset_template": "guitarhub_{version}_x64.dmg",
    },
}


def detect_platform(sig_path: str) -> str | None:
    """Map a .sig filename to a Tauri platform key."""
    lower = sig_path.lower()
    for platform, config in PLATFORM_MAP.items():
        for suffix in config["suffixes"]:
            if suffix in lower:
                return platform
    return None


def build_url(version_tag: str, version: str, platform: str) -> str:
    """Build the download URL for a platform's release asset."""
    config = PLATFORM_MAP[platform]
    asset_name = config["asset_template"].format(version=version)
    return (
        f"https://github.com/{OWNER_REPO}/releases/download/"
        f"{version_tag}/{asset_name}"
    )


def main() -> None:
    if len(sys.argv) < 2:
        print(
            "Usage: generate_latest_json.py <version_tag> [sig1 sig2 ...]",
            file=sys.stderr,
        )
        sys.exit(1)

    version_tag = sys.argv[1]
    sig_paths = sys.argv[2:]
    version = version_tag[1:] if version_tag.startswith("v") else version_tag

    platforms: dict[str, dict[str, str]] = {}

    for sig_path in sig_paths:
        # Support glob patterns (the workflow passes sig-artifacts/*.sig)
        expanded = glob.glob(sig_path) if ("*" in sig_path or "?" in sig_path) else [sig_path]

        for path in expanded:
            if not os.path.isfile(path):
                print(f"Warning: '{path}' not found, skipping", file=sys.stderr)
                continue

            platform = detect_platform(os.path.basename(path))
            if platform is None:
                print(
                    f"Warning: could not detect platform from '{path}', skipping",
                    file=sys.stderr,
                )
                continue

            with open(path) as f:
                signature = f.read().strip()

            if not signature:
                print(f"Warning: empty signature in '{path}', skipping", file=sys.stderr)
                continue

            url = build_url(version_tag, version, platform)
            platforms[platform] = {"signature": signature, "url": url}

    # If no signatures found, emit unsigned entries for all known platforms.
    # This lets the auto-updater know about new versions even when
    # TAURI_PRIVATE_KEY is not configured in CI.
    if not platforms:
        print("Warning: no signatures found — generating unsigned latest.json", file=sys.stderr)
        for platform in PLATFORM_MAP:
            url = build_url(version_tag, version, platform)
            platforms[platform] = {"signature": "", "url": url}

    payload = {
        "version": version,
        "notes": "",
        "pub_date": datetime.datetime.now(datetime.timezone.utc)
        .replace(microsecond=0)
        .isoformat()
        .replace("+00:00", "Z"),
        "platforms": platforms,
    }

    with open("latest.json", "w") as f:
        json.dump(payload, f, indent=2)
        f.write("\n")


if __name__ == "__main__":
    main()
