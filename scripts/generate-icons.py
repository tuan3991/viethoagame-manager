#!/usr/bin/env python3
"""
Generate icon Tauri từ 1 file PNG/SVG đơn giản.

Cài đặt: pip install Pillow

Usage: python generate-icons.py [source.png]
Nếu không có source, sẽ tạo icon mặc định với chữ "VG".
"""

import os
import sys
from pathlib import Path

try:
    from PIL import Image, ImageDraw, ImageFont
except ImportError:
    print("Cần cài Pillow: pip install Pillow")
    sys.exit(1)


def make_default_icon(size: int = 1024) -> Image.Image:
    """Tạo icon mặc định với chữ 'VG' trên nền xanh lá."""
    img = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    draw = ImageDraw.Draw(img)

    # Background gradient (đơn giản)
    bg_color = (22, 163, 74, 255)  # green-600
    bg_color_light = (34, 197, 94, 255)  # green-500

    # Rounded rect
    radius = size // 8
    draw.rounded_rectangle([(0, 0), (size, size)], radius=radius, fill=bg_color)

    # Chữ "VG"
    font_size = size // 2
    try:
        font = ImageFont.truetype("arial.ttf", font_size)
    except OSError:
        font = ImageFont.load_default()

    text = "VG"
    bbox = draw.textbbox((0, 0), text, font=font)
    text_width = bbox[2] - bbox[0]
    text_height = bbox[3] - bbox[1]
    x = (size - text_width) // 2 - bbox[0]
    y = (size - text_height) // 2 - bbox[1] - font_size // 16

    draw.text((x, y), text, fill=(255, 255, 255, 255), font=font)

    return img


def main():
    icons_dir = Path(__file__).parent.parent / "src-tauri" / "icons"
    icons_dir.mkdir(parents=True, exist_ok=True)

    # Source: PNG có sẵn hoặc tạo default
    if len(sys.argv) > 1 and Path(sys.argv[1]).exists():
        print(f"Using source: {sys.argv[1]}")
        source = Image.open(sys.argv[1]).convert("RGBA")
    else:
        print("Creating default icon...")
        source = make_default_icon(1024)

    # Generate các size cần thiết cho Tauri
    sizes = {
        "32x32.png": 32,
        "128x128.png": 128,
        "128x128@2x.png": 256,
        "icon.png": 512,
    }

    for filename, size in sizes.items():
        resized = source.resize((size, size), Image.LANCZOS)
        out_path = icons_dir / filename
        resized.save(out_path)
        print(f"  - {filename} ({size}x{size})")

    # Tạo .ico cho Windows (cần nhiều size)
    ico_sizes = [16, 24, 32, 48, 64, 128, 256]
    ico_imgs = [source.resize((s, s), Image.LANCZOS) for s in ico_sizes]
    ico_imgs[0].save(
        icons_dir / "icon.ico",
        format="ICO",
        sizes=[(s, s) for s in ico_sizes],
        append_images=ico_imgs[1:],
    )
    print(f"  - icon.ico (Windows)")

    # Tạo .icns cho macOS (chỉ lưu 512x512 hoặc 1024x1024)
    source.resize((1024, 1024), Image.LANCZOS).save(
        icons_dir / "icon.icns"
    )
    print(f"  - icon.icns (macOS)")

    print(f"\nDone! Icons created at: {icons_dir}")


if __name__ == "__main__":
    main()