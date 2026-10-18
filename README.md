# Viethoagame Manager

App quản lý và cài đặt bản việt hóa game cho Windows, xây dựng bằng **Tauri + React + TypeScript**.

## Tính năng

- 📚 Thư viện game từ Google Drive (manifest URL công khai)
- 🎮 Auto-detect game đã cài trên Steam, Epic Games, GOG
- ⚡ Cài đặt 1-click với backup/restore tự động
- 🔄 Cập nhật phiên bản mới với changelog
- ❤️ Yêu thích + tìm kiếm + lọc
- 🛡️ Verify SHA256 để đảm bảo file không lỗi
- 🌙 Dark/light theme
- 📊 Admin Panel: thống kê, báo lỗi, bình luận, yêu cầu game
- 📈 Telemetry opt-in (analytics cho admin)

## Tech Stack

- **Frontend**: React 18, TypeScript, Vite, TailwindCSS, shadcn/ui
- **Backend**: Tauri 2 (Rust), SQLite (rusqlite)
- **State**: Zustand + TanStack Query
- **Plugins**: dialog, fs, http, notification, process, shell, store, updater

## Cấu trúc dự án

```
viethoagame-manager/
├── src/                    # React frontend
│   ├── components/
│   ├── routes/
│   ├── lib/                # API wrapper, store, utils
│   ├── types/
│   └── styles/
├── src-tauri/              # Rust backend
│   ├── src/
│   │   ├── commands/       # Tauri commands
│   │   ├── db/             # SQLite module
│   │   └── models.rs
│   ├── Cargo.toml
│   └── tauri.conf.json
├── package.json
└── vite.config.ts
```

## Tài liệu chi tiết

- [docs/MANIFEST_SCHEMA.md](docs/MANIFEST_SCHEMA.md) - Schema manifest Drive
- [docs/DRIVE_SETUP.md](docs/DRIVE_SETUP.md) - Hướng dẫn setup Drive
- [docs/CONTRIBUTING.md](docs/CONTRIBUTING.md) - Hướng dẫn contribute

## Cài đặt & chạy

### Yêu cầu

- Node.js 18+
- Rust 1.70+
- Visual Studio Build Tools (Windows)
- WebView2 (Windows 10+ đã có sẵn)

### Dev mode

```bash
# Cài dependencies
npm install

# Chạy app ở dev mode
npm run tauri:dev
```

Lần đầu chạy sẽ mất ~5-10 phút để Rust compile.

### Build production

```bash
npm run tauri:build
```

File installer sẽ ở `src-tauri/target/release/bundle/`:
- MSI: `msi/Viethoagame Manager_0.1.0_x64_en-US.msi`
- NSIS: `nsis/Viethoagame Manager_0.1.0_x64-setup.exe`

## Workflow thêm game mới

1. Tạo folder `games/<id>/` trên Drive (xem [docs/DRIVE_SETUP.md](docs/DRIVE_SETUP.md))
2. Upload `vi.zip` vào `v<version>/`, lấy SHA256 bằng `scripts/sha256sum.py`
3. Tạo `manifest.json` theo schema, validate bằng `scripts/validate-manifest.py`
4. Thêm entry vào root `manifest.json`
5. Trong app: **Admin Panel → Drive Config** → nhập URL → Test kết nối

## Cấu trúc dữ liệu

App đọc manifest từ Google Drive public URL. Cấu trúc:

```
/Viethoagame Drive/
├── manifest.json                 ← danh sách tất cả game
└── games/
    └── <game-id>/
        ├── manifest.json         ← metadata + install config
        ├── v1.0/vi.zip          ← bản việt hóa
        ├── v1.1/vi.zip
        ├── screenshots/
        └── cover.jpg
```

Xem chi tiết schema trong `docs/MANIFEST_SCHEMA.md`.

## Local data

App lưu local ở `%APPDATA%/com.viethoagame.manager/`:

```
├── viethoagame.db              # SQLite - installed games, history, settings
├── cache/
│   └── downloads/<game-id>.zip # File đã tải
└── backups/                    # Backup files gốc của game
    └── <game-id>/<version>/
```

## Đóng góp

Xem `docs/CONTRIBUTING.md`.

## License

MIT