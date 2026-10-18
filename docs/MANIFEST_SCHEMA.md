# Schema cho Manifest Drive

## File `/manifest.json` (root)

```json
{
  "version": 1,
  "lastUpdated": "2026-10-18T00:00:00Z",
  "games": [
    {
      "id": "witcher-3",
      "name": "The Witcher 3: Wild Hunt",
      "manifestUrl": "https://drive.google.com/uc?export=download&id=FILE_ID"
    }
  ]
}
```

## File `/games/<id>/manifest.json`

Xem chi tiết tại `manifest.example.json` trong thư mục này.

## Cách upload lên Google Drive

1. Tạo folder `Viethoagame` trên Google Drive
2. Trong folder đó, tạo:
   - `manifest.json` (file root)
   - `games/<game-id>/manifest.json`
   - `games/<game-id>/v<version>/vi.zip`
   - `games/<game-id>/screenshots/*.jpg`
3. Share từng file với quyền "Anyone with the link"
4. Lấy URL share, convert thành direct link:

```
https://drive.google.com/file/d/FILE_ID/view
              ↓
https://drive.google.com/uc?export=download&id=FILE_ID
```

5. Paste URL vào manifest.json, paste manifest.json URL vào app Settings

## Validate manifest

Dùng script `scripts/validate-manifest.py`:

```bash
python scripts/validate-manifest.py manifest.json
```