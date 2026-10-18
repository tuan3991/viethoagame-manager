# Hướng dẫn cài đặt & chạy Viethoagame Manager

## Yêu cầu hệ thống

- **OS**: Windows 10/11 (64-bit)
- **Node.js**: 18+ (đã có sẵn - kiểm tra bằng `node --version`)
- **Rust**: 1.70+ (cần cài thêm - xem bên dưới)
- **WebView2 Runtime**: đã có sẵn trên Win11, có thể cần cài trên Win10
- **Visual Studio Build Tools** (chỉ C++ build tools): cần cho compile Rust

---

## Bước 1: Cài Rust

### Cách 1: Dùng rustup-init (khuyến nghị)

1. Tải `rustup-init.exe` từ https://rustup.rs/
2. Chạy file vừa tải
3. Chọn option `1) Proceed with installation (default)`
4. Đợi cài xong (~2-5 phút)
5. **Mở PowerShell MỚI** và kiểm tra:
   ```powershell
   rustc --version
   cargo --version
   ```

### Cách 2: Dùng winget (Windows 11)

```powershell
winget install Rustlang.Rustup
```

### Cách 3: Dùng chocolatey

```powershell
choco install rust
```

## Bước 2: Cài Visual Studio Build Tools (Windows)

Tauri cần MSVC linker để compile. Cài 1 trong 2:

### Cách 1: Dùng Visual Studio Installer (khuyến nghị)

1. Tải **Build Tools for Visual Studio** từ:
   https://visualstudio.microsoft.com/downloads/#build-tools-for-visual-studio-2022
2. Chạy installer
3. Tick **"Desktop development with C++"**
4. Bấm Install (~5-10GB, mất 15-30 phút)

### Cách 2: Dùng winget

```powershell
winget install Microsoft.VisualStudio.2022.BuildTools
```

## Bước 3: Kiểm tra WebView2

### Trên Windows 11
Đã có sẵn, không cần cài.

### Trên Windows 10
- Tải **Evergreen WebView2 Runtime**:
  https://developer.microsoft.com/en-us/microsoft-edge/webview2/
- Hoặc dùng winget:
  ```powershell
  winget install Microsoft.EdgeWebView2Runtime
  ```

## Bước 4: Cài dependencies cho project

Mở PowerShell trong thư mục dự án:

```powershell
cd "c:\Users\tuan3\Documents\cursor Viethoagame\viethoagame-manager"

# Cài npm packages
npm install

# Kiểm tra cargo có fetch crates được không
# (Rust sẽ tự động tải khi build lần đầu)
```

## Bước 5: Chạy app ở dev mode

```powershell
npm run tauri:dev
```

Lần đầu chạy:
- Rust compile: **5-15 phút** (tuỳ máy)
- Sau khi compile xong, cửa sổ app sẽ tự động mở

Các lần sau chỉ cần **10-30 giây** (cache build đã có).

## Bước 6: Test nhanh

App mở ra sẽ thấy:
- Sidebar bên trái: Thư viện, Cài đặt, Admin, Giới thiệu
- Trang Library trống (vì chưa có manifest)

Để test với data thật:
1. Tạo 1 manifest đơn giản (xem `docs/DRIVE_SETUP.md`)
2. Upload lên Google Drive
3. Trong app: **Settings** → nhập manifest URL → Reload

## Bước 7: Build installer (.msi / .exe)

```powershell
npm run tauri:build
```

Output ở `src-tauri/target/release/bundle/`:
- `msi/Viethoagame Manager_0.1.0_x64_en-US.msi` - Installer Windows
- `nsis/Viethoagame Manager_0.1.0_x64-setup.exe` - Portable installer

Build production mất **10-30 phút** lần đầu.

---

## Troubleshooting

### Lỗi "linker not found"

Cài **Visual Studio Build Tools** (xem Bước 2).

### Lỗi "WebView2 not found"

Cài **WebView2 Runtime** (xem Bước 3).

### Lỗi npm install timeout

Tăng timeout:
```powershell
npm config set fetch-timeout 600000
npm install
```

### Lỗi Cargo build chậm

Rust compile lần đầu rất chậm. Các giải pháp:
- Dùng ổ SSD
- Tăng RAM (compile dùng ~4GB)
- Đặt `target/` ở RAM disk (advanced)

### Lỗi "Cannot find module" khi chạy

Chắc chắn đã chạy `npm install`. Nếu vẫn lỗi:
```powershell
Remove-Item -Recurse -Force node_modules
Remove-Item package-lock.json
npm install
```

### Lỗi Windows Defender block Rust

Có thể bị Windows Defender chặn `rustc.exe`. Whitelist folder:
```
C:\Users\<user>\.cargo\
C:\Users\<user>\.rustup\
```

### App mở nhưng crash ngay

Mở PowerShell và chạy:
```powershell
cd "c:\Users\tuan3\Documents\cursor Viethoagame\viethoagame-manager"
npm run tauri:dev
```
Xem log trong terminal - sẽ có thông tin chi tiết.

---

## Checklist chạy thành công

- [ ] Node.js 18+ đã cài
- [ ] Rust 1.70+ đã cài (`rustc --version`)
- [ ] Visual Studio Build Tools đã cài
- [ ] WebView2 Runtime có sẵn (hoặc đã cài)
- [ ] `npm install` chạy thành công
- [ ] `npm run tauri:dev` mở app
- [ ] App hiển thị UI đầy tiên
- [ ] Sidebar có 4 mục: Thư viện / Cài đặt / Admin / Giới thiệu

---

## Lệnh thường dùng

```powershell
# Chạy dev
npm run tauri:dev

# Build installer
npm run tauri:build

# Type-check TypeScript
npx tsc --noEmit

# Lint
npm run lint

# Format code
npm run format

# Validate manifest
python scripts/validate-manifest.py docs/manifest.example.json --game

# Tính SHA256
python scripts/sha256sum.py path/to/vi.zip

# Generate icons (1 lần)
python scripts/generate-icons.py
```