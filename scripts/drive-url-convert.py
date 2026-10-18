#!/usr/bin/env python3
"""Convert Google Drive share URL sang direct download URL."""

import re
import sys
from urllib.parse import parse_qs, urlparse


def convert(url: str) -> str:
    """Convert nhiều dạng URL Drive về direct download URL."""
    # Đã là direct
    if "export=download" in url:
        return url

    # Dạng: https://drive.google.com/file/d/FILE_ID/view?usp=sharing
    m = re.search(r"/file/d/([a-zA-Z0-9_-]+)", url)
    if m:
        return f"https://drive.google.com/uc?export=download&id={m.group(1)}"

    # Dạng: https://drive.google.com/open?id=FILE_ID
    parsed = urlparse(url)
    qs = parse_qs(parsed.query)
    if "id" in qs:
        return f"https://drive.google.com/uc?export=download&id={qs['id'][0]}"

    # Không match
    return url


def main():
    if len(sys.argv) < 2:
        print("Usage: drive-url-convert.py <share-url>")
        print()
        print("Ví dụ:")
        print('  drive-url-convert.py "https://drive.google.com/file/d/1abc.../view?usp=sharing"')
        sys.exit(1)

    url = sys.argv[1]
    result = convert(url)
    print(result)


if __name__ == "__main__":
    main()