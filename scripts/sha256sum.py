#!/usr/bin/env python3
"""Tính SHA256 checksum cho file vi.zip."""

import hashlib
import sys
from pathlib import Path


def sha256_file(path: Path) -> str:
    sha = hashlib.sha256()
    with open(path, "rb") as f:
        while chunk := f.read(8192):
            sha.update(chunk)
    return sha.hexdigest()


def main():
    if len(sys.argv) < 2:
        print("Usage: sha256sum.py <file>")
        sys.exit(1)

    path = Path(sys.argv[1])
    if not path.exists():
        print(f"File not found: {path}")
        sys.exit(1)

    digest = sha256_file(path)
    print(digest)
    print(f"# Size: {path.stat().st_size} bytes", file=sys.stderr)


if __name__ == "__main__":
    main()