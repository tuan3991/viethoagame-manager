# Contributing

## Development workflow

1. Fork repo
2. Tạo branch mới: `git checkout -b feature/ten-tinh-nang`
3. Commit: `git commit -m 'feat: them tinh nang X'`
4. Push: `git push origin feature/ten-tinh-nang`
5. Tạo Pull Request

## Conventions

### Code style

- **Rust**: `cargo fmt` + `cargo clippy`
- **TypeScript**: ESLint + Prettier (config trong repo)
- **Commit**: Conventional Commits (`feat:`, `fix:`, `docs:`, `refactor:`...)

### File structure

Mỗi tính năng mới cần:
1. **Tauri command** (Rust) trong `src-tauri/src/commands/`
2. **API wrapper** (TypeScript) trong `src/lib/api.ts`
3. **UI component** trong `src/components/` hoặc `src/routes/`

### Manifest schema

Khi thêm field mới vào manifest, update:
- `src/types/index.ts` (frontend types)
- `src-tauri/src/models.rs` (Rust types)
- `docs/MANIFEST_SCHEMA.md`
- `docs/manifest.example.json`

## Testing

### Manual testing checklist

Trước khi submit PR:

- [ ] `npm run tauri:dev` chạy thành công
- [ ] App khởi động, hiển thị thư viện
- [ ] Search/filter hoạt động
- [ ] Click vào game hiển thị detail
- [ ] Install/uninstall flow chạy đúng (test với 1 game)
- [ ] Verify backup/restore
- [ ] Verify SHA256 mismatch xử lý đúng
- [ ] Check cancel install
- [ ] Settings persist sau khi restart app

## Adding a new game to Drive

1. Tạo folder `games/<game-id>/` trên Drive
2. Upload `vi.zip` vào folder con `v<version>/`
3. Tạo `manifest.json` theo schema (dùng `scripts/validate-manifest.py` để check)
4. Compute SHA256: `python scripts/sha256sum.py vi.zip`
5. Add entry vào root `manifest.json`
6. Verify URL trong app Settings