# Hướng dẫn setup Manifest trên Google Drive

Bước 1-3 dưới đây chỉ cần làm 1 lần. Bước 4+ mỗi khi thêm game/bản việt hóa mới.

## Bước 1: Tạo folder Drive

Tạo folder `Viethoagame` trên Google Drive của bạn.

## Bước 2: Upload manifest root

Tạo file `manifest.json` ở **root folder `Viethoagame/`**:

```json
{
  "version": 1,
  "lastUpdated": "2026-10-18T00:00:00Z",
  "games": [
    {
      "id": "witcher-3",
      "name": "The Witcher 3: Wild Hunt",
      "manifestUrl": "https://drive.google.com/uc?export=download&id=WITCHER_MANIFEST_FILE_ID"
    }
  ]
}
```

Lưu file này vào Google Drive.

## Bước 3: Tạo folder cho mỗi game

Trong `Viethoagame/`, tạo folder `games/<game-id>/`:

```
Viethoagame/
├── manifest.json              ← root
└── games/
    └── witcher-3/
        ├── manifest.json      ← metadata + install config
        ├── cover.jpg
        ├── screenshots/
        │   ├── 01.jpg
        │   └── 02.jpg
        └── v1.32/
            └── vi.zip          ← bản việt hóa
```

## Bước 4: Share folder/file công khai

Với MỖI file/folder:
1. Click chuột phải → "Share" → "General access"
2. Chọn **"Anyone with the link"**
3. Role: **"Viewer"**
4. Copy link

## Bước 5: Convert link thành direct URL

Dùng script `scripts/drive-url-convert.py`:

```bash
# Link share dạng:
# https://drive.google.com/file/d/1AbCdEfG.../view?usp=sharing

python scripts/drive-url-convert.py "https://drive.google.com/file/d/1AbCdEfG.../view?usp=sharing"

# Output:
# https://drive.google.com/uc?export=download&id=1AbCdEfG...
```

Lưu ý: Với file >100MB, Drive cần xác nhận. URL cuối cùng sẽ có dạng:
```
https://drive.usercontent.google.com/download?id=FILE_ID&export=download&confirm=t
```
Script sẽ tự xử lý nếu paste đúng link share.

## Bước 6: Paste URL vào app

1. Mở app Viethoagame Manager
2. Vào **Settings** → nhập URL của root manifest vào
3. App sẽ fetch và hiển thị danh sách game

## Bước 7: Validate manifest trước khi upload

```bash
# Validate manifest root
python scripts/validate-manifest.py manifest.json

# Validate manifest của game
python scripts/validate-manifest.py games/witcher-3/manifest.json --game
```

## Bước 8: Tính SHA256 cho file vi.zip

```bash
python scripts/sha256sum.py vi.zip
# Output: abc123def456...
```

Paste giá trị này vào field `checksum` của `manifest.json` game.

## File Drive nên tổ chức thế nào

```
Viethoagame Drive/
│
├── manifest.json              ← ALWAYS cập nhật khi thêm/xoá game
│
├── games/
│   ├── witcher-3/
│   │   ├── manifest.json      ← Mỗi game có 1 manifest
│   │   ├── cover.jpg
│   │   ├── screenshots/
│   │   │   ├── 01.jpg
│   │   │   └── 02.jpg
│   │   ├── v1.30/
│   │   │   └── vi.zip         ← Mỗi phiên bản 1 folder riêng
│   │   ├── v1.31/
│   │   │   └── vi.zip
│   │   └── v1.32/
│   │       └── vi.zip
│   │
│   └── cyberpunk2077/
│       └── ...
│
└── assets/                    ← Optional: assets chung
    ├── logo.png
    └── banner.jpg
```

## Mẹo quan trọng

### Kích thước file

- File vi.zip thường 20-100MB. Drive miễn phí 15GB → đủ cho ~150 game.
- Nếu vượt, dùng GitHub Releases thay thế (cũng support public link).

### Caching

Drive có cache. Sau khi update `manifest.json`, có thể mất ~5 phút để app thấy thay đổi.

Workaround: thêm query string `?v=2` vào URL manifest để bust cache:
```
https://drive.google.com/uc?export=download&id=FILE_ID&v=2
```

### Backup

Drive của bạn là single point of failure. Nên:
- Sync folder `Viethoagame` với 1 cloud khác (Dropbox, OneDrive)
- Hoặc lưu local backup trên NAS

### Multi-admin

Nếu có nhiều người cùng quản trị:
- Tạo shared folder trong Drive
- Add các admin làm "Editor"
- Ai cũng có thể sửa manifest + upload file