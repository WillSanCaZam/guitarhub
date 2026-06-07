#!/usr/bin/env python3
"""Generate a Tauri updater latest.json with real URL and signature.

Usage: generate_latest_json.py <version_tag> <signature_file>

The signature file is a .sig file produced by `tauri signer sign`.
Only linux-x86_64 platform is emitted.
"""

import json
import sys
import datetime
import os


def main() -> None:
    if len(sys.argv) < 3:
        print("Usage: generate_latest_json.py <version_tag> <signature_file>",
              file=sys.stderr)
        sys.exit(1)

    version_tag = sys.argv[1]
    sig_path = sys.argv[2]

    # Strip a leading 'v' for the semver version
    version = version_tag[1:] if version_tag.startswith("v") else version_tag

    # Read and validate the signature file
    if not os.path.isfile(sig_path):
        print(f"Error: signature file '{sig_path}' not found", file=sys.stderr)
        sys.exit(1)

    with open(sig_path, "r") as f:
        signature = f.read().strip()

    if not signature:
        print("Error: signature file is empty", file=sys.stderr)
        sys.exit(1)

    # Build the download URL for the .deb release asset.
    # GitHub release asset URL format:
    #   https://github.com/{owner}/{repo}/releases/download/{tag}/{filename}
    url = (
        f"https://github.com/willsancazam/guitarhub/releases/download/"
        f"{version_tag}/guitarhub_{version}_amd64.deb"
    )

    payload = {
        "version": version,
        "notes": "",
        "pub_date": datetime.datetime.now(datetime.timezone.utc)
        .replace(microsecond=0)
        .isoformat()
        .replace("+00:00", "Z"),
        "platforms": {
            "linux-x86_64": {
                "signature": signature,
                "url": url,
            },
        },
    }

    with open("latest.json", "w") as f:
        json.dump(payload, f, indent=2)
        f.write("\n")


if __name__ == "__main__":
    main()
