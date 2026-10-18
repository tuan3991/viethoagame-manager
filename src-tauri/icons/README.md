# Tauri App Icons

Bạn cần tạo các file icon sau trong thư mục này để build production:

- `32x32.png` - icon 32x32
- `128x128.png` - icon 128x128
- `128x128@2x.png` - icon 256x256 (cho retina)
- `icon.icns` - icon macOS
- `icon.ico` - icon Windows

## Tạo nhanh bằng CLI

```bash
# Có 1 file PNG gốc (1024x1024), dùng:
npx @tauri-apps/cli icon path/to/source-icon.png
```

Tool sẽ tự generate tất cả các size + format cần thiết.

## Yêu cầu

- Background đặc, không trong suốt
- Logo "VG" hoặc tên app "Viethoagame"
- Màu chính: xanh lá (xem `tailwind.config.js`: `hsl(142, 76%, 36%)`)