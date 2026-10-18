//! Commands liên quan tới Google Drive
//!
//! - Fetch manifest từ URL công khai
//! - Download file việt hóa
//! - Verify SHA256 checksum

use crate::commands::settings;
use crate::db::DbState;
use crate::models::*;
use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::Path;
use tauri::{AppHandle, Emitter, State};

/// URL mặc định của manifest (GitHub Releases - sẽ được override bằng setting)
///
/// Repo: https://github.com/tuan3991/viethoagame-manager
/// Release: v2 (Latest, tự động tăng khi có bản mới)
///
/// GitHub Releases có ưu điểm: không giới hạn size, không bị virus warning,
/// không cần auth, hỗ trợ atomic updates (CDN cached).
/// Cập nhật manifest chỉ cần tạo release mới (v3, v4...) và upload lại file.
pub const DEFAULT_MANIFEST_URL: &str =
    "https://github.com/tuan3991/viethoagame-manager/releases/latest/download/manifest.json";

/// Lấy URL manifest hiện tại từ settings
async fn get_manifest_url(db: &State<'_, DbState>) -> Result<String> {
    let url_opt = settings::get_setting(db.clone(), "manifest_url".to_string()).await?;
    match url_opt {
        Some(url) => Ok(url),
        None => Ok(DEFAULT_MANIFEST_URL.to_string()),
    }
}

/// Fetch manifest từ Google Drive
pub async fn fetch_manifest(db: State<'_, DbState>) -> Result<Manifest> {
    let url = get_manifest_url(&db).await?;
    tracing::info!("Fetching manifest from: {}", url);

    let response = reqwest::get(&url).await.context("Failed to fetch manifest")?;

    if !response.status().is_success() {
        return Err(anyhow!(
            "Failed to fetch manifest: HTTP {}",
            response.status()
        ));
    }

    let manifest: Manifest = response
        .json()
        .await
        .context("Failed to parse manifest JSON")?;

    tracing::info!("Fetched manifest with {} games", manifest.games.len());
    Ok(manifest)
}

/// Fetch manifest chi tiết của 1 game
pub async fn fetch_game_manifest(db: State<'_, DbState>, game_id: String) -> Result<GameManifest> {
    // Đầu tiên lấy danh sách games để biết URL
    let manifest = fetch_manifest(db.clone()).await?;

    let entry = manifest
        .games
        .iter()
        .find(|g| g.id == game_id)
        .ok_or_else(|| anyhow!("Game '{}' not found in manifest", game_id))?;

    tracing::info!("Fetching game manifest: {}", entry.manifest_url);

    let response = reqwest::get(&entry.manifest_url)
        .await
        .context("Failed to fetch game manifest")?;

    let game: GameManifest = response
        .json()
        .await
        .context("Failed to parse game manifest JSON")?;

    Ok(game)
}

/// Download progress event payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadProgress {
    pub game_id: String,
    pub version: String,
    pub downloaded_bytes: u64,
    pub total_bytes: u64,
    pub speed_bytes_per_sec: u64,
    pub status: String,
    pub message: Option<String>,
}

/// Download file từ URL với progress event
pub async fn download_file_with_progress(
    app: &AppHandle,
    url: &str,
    dest: &Path,
    game_id: &str,
    version: &str,
    expected_sha256: &str,
) -> Result<()> {
    use futures_util::StreamExt;
    use std::io::Write;

    tracing::info!("Downloading {} to {:?}", url, dest);

    // Tạo folder cha nếu cần
    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let client = reqwest::Client::builder()
        .user_agent("ViethoagameManager/0.1")
        .build()?;

    let response = client.get(url).send().await?;
    if !response.status().is_success() {
        return Err(anyhow!("Download failed: HTTP {}", response.status()));
    }

    let total_bytes = response.content_length().unwrap_or(0);

    // Emit starting event
    let _ = app.emit(
        "download-progress",
        DownloadProgress {
            game_id: game_id.to_string(),
            version: version.to_string(),
            downloaded_bytes: 0,
            total_bytes,
            speed_bytes_per_sec: 0,
            status: "downloading".to_string(),
            message: None,
        },
    );

    let mut file = std::fs::File::create(dest)?;
    let mut stream = response.bytes_stream();

    let mut downloaded: u64 = 0;
    let mut last_emit = std::time::Instant::now();
    let mut last_bytes: u64 = 0;

    while let Some(chunk_result) = stream.next().await {
        let chunk = chunk_result?;
        file.write_all(&chunk)?;
        downloaded += chunk.len() as u64;

        // Emit progress mỗi 500ms
        let now = std::time::Instant::now();
        if now.duration_since(last_emit).as_millis() >= 500 {
            let elapsed = now.duration_since(last_emit).as_secs_f64();
            let speed = if elapsed > 0.0 {
                ((downloaded - last_bytes) as f64 / elapsed) as u64
            } else {
                0
            };

            let _ = app.emit(
                "download-progress",
                DownloadProgress {
                    game_id: game_id.to_string(),
                    version: version.to_string(),
                    downloaded_bytes: downloaded,
                    total_bytes,
                    speed_bytes_per_sec: speed,
                    status: "downloading".to_string(),
                    message: None,
                },
            );

            last_emit = now;
            last_bytes = downloaded;
        }
    }

    file.flush()?;
    drop(file);

    // Verify SHA256
    let _ = app.emit(
        "download-progress",
        DownloadProgress {
            game_id: game_id.to_string(),
            version: version.to_string(),
            downloaded_bytes: downloaded,
            total_bytes,
            speed_bytes_per_sec: 0,
            status: "verifying".to_string(),
            message: Some("Đang kiểm tra checksum...".to_string()),
        },
    );

    let actual_sha256 = sha256_file(dest)?;
    if actual_sha256 != expected_sha256 {
        std::fs::remove_file(dest).ok();
        return Err(anyhow!(
            "Checksum mismatch!\nExpected: {}\nActual: {}",
            expected_sha256,
            actual_sha256
        ));
    }

    tracing::info!("Download complete and verified: {:?}", dest);
    Ok(())
}

/// Tính SHA256 của file
pub fn sha256_file(path: &Path) -> Result<String> {
    use std::io::Read;

    let mut file = std::fs::File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 8192];

    loop {
        let bytes_read = file.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }

    Ok(hex::encode(hasher.finalize()))
}

/// SHA256 của bytes (cho testing)
#[allow(dead_code)]
pub fn sha256_bytes(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hex::encode(hasher.finalize())
}

/// Convert Google Drive URL thành direct download link
///
/// Drive URL dạng: https://drive.google.com/uc?export=download&id=FILE_ID
/// Hoặc: https://drive.google.com/file/d/FILE_ID/view?usp=sharing
pub fn normalize_drive_url(url: &str) -> String {
    // Nếu đã là direct link thì trả về
    if url.contains("export=download") {
        return url.to_string();
    }

    // Parse FILE_ID từ URL dạng /file/d/FILE_ID/...
    if let Some(start) = url.find("/file/d/") {
        let after = &url[start + 9..];
        if let Some(end) = after.find('/') {
            let file_id = &after[..end];
            return format!(
                "https://drive.google.com/uc?export=download&id={}",
                file_id
            );
        }
    }

    url.to_string()
}