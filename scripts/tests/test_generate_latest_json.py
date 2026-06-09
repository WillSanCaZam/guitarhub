#!/usr/bin/env python3
"""Tests for generate_latest_json.py.

Run: python scripts/tests/test_generate_latest_json.py
"""

import os
import sys
import tempfile
import subprocess


SCRIPT_PATH = os.path.join(
    os.path.dirname(__file__), "..", "generate_latest_json.py"
)


def test_rejects_empty_sig() -> None:
    """Script must exit with error when signature file is empty."""
    with tempfile.TemporaryDirectory() as tmp:
        sig_path = os.path.join(tmp, "empty.sig")
        with open(sig_path, "w") as f:
            f.write("")  # empty file

        result = subprocess.run(
            [sys.executable, SCRIPT_PATH, "v0.1.1", sig_path],
            capture_output=True,
            text=True,
            cwd=tmp,
        )

        assert result.returncode != 0, (
            f"Expected non-zero exit, got {result.returncode}. "
            f"stdout: {result.stdout}, stderr: {result.stderr}"
        )
        assert "empty" in result.stderr.lower(), (
            f"Expected stderr to mention 'empty', got: {result.stderr}"
        )

        # latest.json must NOT be written
        latest_path = os.path.join(tmp, "latest.json")
        assert not os.path.exists(latest_path), (
            "latest.json should NOT exist when sig is empty"
        )


def test_rejects_missing_sig() -> None:
    """Script must exit with error when signature file does not exist."""
    with tempfile.TemporaryDirectory() as tmp:
        result = subprocess.run(
            [sys.executable, SCRIPT_PATH, "v0.1.1", "/nonexistent/file.sig"],
            capture_output=True,
            text=True,
            cwd=tmp,
        )

        assert result.returncode != 0, (
            f"Expected non-zero exit, got {result.returncode}. "
            f"stderr: {result.stderr}"
        )
        assert "not found" in result.stderr.lower(), (
            f"Expected stderr to mention 'not found', got: {result.stderr}"
        )


def test_requires_sig_arg() -> None:
    """Script must exit with error when no signature file argument.""" ""
    result = subprocess.run(
        [sys.executable, SCRIPT_PATH, "v0.1.1"],
        capture_output=True,
        text=True,
    )

    assert result.returncode != 0
    assert "Usage:" in result.stderr


def test_generates_valid_json_with_real_sig() -> None:
    """Script must produce valid latest.json with a real signature."""
    with tempfile.TemporaryDirectory() as tmp:
        sig_path = os.path.join(tmp, "valid.sig")
        with open(sig_path, "w") as f:
            f.write("c2lnbmF0dXJlLWJ5dGVzCg==")  # fake base64 sig

        result = subprocess.run(
            [sys.executable, SCRIPT_PATH, "v0.1.1", sig_path],
            capture_output=True,
            text=True,
            cwd=tmp,
        )

        assert result.returncode == 0, (
            f"Expected zero exit, got {result.returncode}. "
            f"stderr: {result.stderr}"
        )

        latest_path = os.path.join(tmp, "latest.json")
        assert os.path.exists(latest_path), "latest.json must exist"

        import json

        with open(latest_path) as f:
            payload = json.load(f)

        assert payload["version"] == "0.1.1"
        assert "linux-x86_64" in payload["platforms"]
        assert "darwin" not in str(payload["platforms"]).lower()
        assert "windows" not in str(payload["platforms"]).lower()
        assert payload["platforms"]["linux-x86_64"]["signature"] == "c2lnbmF0dXJlLWJ5dGVzCg=="
        assert payload["platforms"]["linux-x86_64"]["url"] != ""
        assert "github.com" in payload["platforms"]["linux-x86_64"]["url"]


def main() -> None:
    tests = [
        ("rejects_empty_sig", test_rejects_empty_sig),
        ("rejects_missing_sig", test_rejects_missing_sig),
        ("requires_sig_arg", test_requires_sig_arg),
        ("generates_valid_json_with_real_sig", test_generates_valid_json_with_real_sig),
    ]

    failures = 0
    for name, func in tests:
        try:
            func()
            print(f"  ✅ {name}")
        except AssertionError as e:
            print(f"  ❌ {name}: {e}")
            failures += 1
        except Exception as e:
            print(f"  💥 {name}: {e}")
            failures += 1

    if failures:
        print(f"\n{failures} test(s) failed")
        sys.exit(1)
    else:
        print(f"\nAll {len(tests)} tests passed")


if __name__ == "__main__":
    main()
