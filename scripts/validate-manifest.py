#!/usr/bin/env python3
"""Validate manifest.json trước khi upload lên Drive."""

import json
import sys
from pathlib import Path


def validate_root_manifest(data: dict) -> list[str]:
    errors = []
    if "version" not in data:
        errors.append("Thiếu field 'version'")
    if "lastUpdated" not in data:
        errors.append("Thiếu field 'lastUpdated'")
    if "games" not in data or not isinstance(data["games"], list):
        errors.append("Thiếu field 'games' hoặc không phải list")
        return errors

    for i, game in enumerate(data["games"]):
        if "id" not in game:
            errors.append(f"games[{i}]: thiếu 'id'")
        if "name" not in game:
            errors.append(f"games[{i}]: thiếu 'name'")
        if "manifestUrl" not in game:
            errors.append(f"games[{i}]: thiếu 'manifestUrl'")
    return errors


def validate_game_manifest(data: dict) -> list[str]:
    errors = []
    required = [
        "id", "name", "shortDescription", "longDescription",
        "developer", "publisher", "releaseDate", "genres",
        "coverImage", "latestVersion", "versions", "detect", "install"
    ]
    for field in required:
        if field not in data:
            errors.append(f"Thiếu field bắt buộc: '{field}'")

    if "versions" in data and isinstance(data["versions"], list):
        if len(data["versions"]) == 0:
            errors.append("'versions' không được rỗng")
        for i, v in enumerate(data["versions"]):
            v_required = ["version", "releasedAt", "sizeBytes", "downloadUrl", "checksum"]
            for f in v_required:
                if f not in v:
                    errors.append(f"versions[{i}]: thiếu '{f}'")

            # Check version nào là latest
            if v.get("version") == data.get("latestVersion"):
                # Verify version latest tồn tại đúng
                pass

    if "install" in data and isinstance(data["install"], dict):
        if "files" not in data["install"] or not data["install"]["files"]:
            errors.append("install.files không được rỗng")
        for i, rule in enumerate(data["install"]["files"]):
            rule_required = ["source", "target", "backupOriginal"]
            for f in rule_required:
                if f not in rule:
                    errors.append(f"install.files[{i}]: thiếu '{f}'")

    return errors


def main():
    if len(sys.argv) < 2:
        print("Usage: validate-manifest.py <path-to-manifest.json> [--game]")
        sys.exit(1)

    path = Path(sys.argv[1])
    if not path.exists():
        print(f"File not found: {path}")
        sys.exit(1)

    try:
        with open(path, encoding="utf-8") as f:
            data = json.load(f)
    except json.JSONDecodeError as e:
        print(f"❌ JSON parse error: {e}")
        sys.exit(1)

    is_game = "--game" in sys.argv

    print(f"🔍 Validating: {path}")
    print(f"   Mode: {'game manifest' if is_game else 'root manifest'}")
    print()

    if is_game:
        errors = validate_game_manifest(data)
    else:
        errors = validate_root_manifest(data)

    if errors:
        print(f"❌ {len(errors)} lỗi:")
        for e in errors:
            print(f"   - {e}")
        sys.exit(1)

    print(f"✅ OK!")
    if not is_game:
        print(f"   Games: {len(data.get('games', []))}")


if __name__ == "__main__":
    main()