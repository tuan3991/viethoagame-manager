//! Admin commands - CRUD cho games, analytics, reports

use crate::commands::settings;
use crate::db::DbState;
use crate::models::*;
use anyhow::{anyhow, Context, Result};
use reqwest;
use rusqlite::params;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::Path;
use tauri::{AppHandle, Manager, State};

// === CONFIG ===
// Trong production, nên lưu admin password hash + dùng keyring
// Tạm thời: hardcode password đơn giản
const ADMIN_PASSWORD_HASH: &str = "5e884898da28047151d0e56f8dc6292773603d0d6aabbdd62a11ef721d1542d8"; // "password"

// === Admin auth ===

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminAnalytics {
    pub total_downloads: u64,
    pub active_installs: u64,
    pub unique_users: u64,
    pub average_rating: f32,
    pub downloads_this_week: u64,
    pub downloads_trend: String,
    pub top_games: Vec<TopGame>,
    pub downloads_by_day: Vec<DailyDownload>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopGame {
    pub id: String,
    pub name: String,
    pub downloads: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyDownload {
    pub date: String,
    pub downloads: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BugReport {
    pub id: String,
    pub game_id: String,
    pub version: String,
    pub user_id: String,
    pub title: String,
    pub description: String,
    pub status: String,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Comment {
    pub id: String,
    pub game_id: String,
    pub user_id: String,
    pub rating: u8,
    pub content: String,
    pub status: String,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserRequest {
    pub id: String,
    pub game_name: String,
    pub votes: u32,
    pub requested_by: String,
    pub created_at: i64,
}

// ================================================================
// GAME MANAGEMENT
// ================================================================

/// Thêm/sửa game trong manifest
pub async fn admin_upsert_game(
    app: AppHandle,
    db: State<'_, DbState>,
    game_id: String,
    name: String,
    manifest_url: String,
) -> Result<()> {
    validate_admin_password(&db)?;
    tracing::info!("Admin: upserting game {} ({})", game_id, name);

    // 1. Download manifest hiện tại từ Drive (nếu có)
    let manifest_url_setting = settings::get_setting(db, "manifest_url".to_string()).await?;
    let mut manifest: Manifest = if let Some(url) = manifest_url_setting {
        match download_json::<Manifest>(&url).await {
            Ok(m) => m,
            Err(_) => Manifest {
                version: 1,
                last_updated: chrono::Utc::now().to_rfc3339(),
                games: vec![],
            },
        }
    } else {
        Manifest {
            version: 1,
            last_updated: chrono::Utc::now().to_rfc3339(),
            games: vec![],
        }
    };

    // 2. Update hoặc thêm entry
    let now = chrono::Utc::now().to_rfc3339();
    if let Some(entry) = manifest.games.iter_mut().find(|g| g.id == game_id) {
        entry.name = name;
        entry.manifest_url = manifest_url;
    } else {
        manifest.games.push(ManifestEntry {
            id: game_id.clone(),
            name,
            manifest_url,
        });
    }
    manifest.last_updated = now;

    // 3. Upload manifest mới lên Drive
    //    NOTE: Đây là chỗ phức tạp vì Drive cần OAuth hoặc API key để ghi.
    //    Cách đơn giản: lưu manifest locally rồi user upload tay
    let local_manifest_path = app
        .path()
        .app_data_dir()
        .context("Failed to get app data dir")?
        .join("manifest.json");

    std::fs::create_dir_all(local_manifest_path.parent().unwrap())?;
    let json = serde_json::to_string_pretty(&manifest)?;
    std::fs::write(&local_manifest_path, json)?;

    tracing::info!(
        "Manifest saved locally at: {:?}. User needs to upload this file to Drive.",
        local_manifest_path
    );

    Ok(())
}

/// Xoá game khỏi manifest
pub async fn admin_delete_game(
    app: AppHandle,
    db: State<'_, DbState>,
    game_id: String,
) -> Result<()> {
    validate_admin_password(&db)?;

    let manifest_url = settings::get_setting(db, "manifest_url".to_string()).await?
        .ok_or_else(|| anyhow!("Manifest URL not configured"))?;

    let mut manifest: Manifest = download_json(&manifest_url)
        .await
        .context("Failed to download manifest")?;

    manifest.games.retain(|g| g.id != game_id);
    manifest.last_updated = chrono::Utc::now().to_rfc3339();

    let local_manifest_path = app
        .path()
        .app_data_dir()
        .context("Failed to get app data dir")?
        .join("manifest.json");
    std::fs::write(&local_manifest_path, serde_json::to_string_pretty(&manifest)?)?;

    Ok(())
}

/// Thêm/sửa version cho game
pub async fn admin_add_version(
    db: State<'_, DbState>,
    game_id: String,
    version: GameVersion,
) -> Result<()> {
    validate_admin_password(&db)?;
    tracing::info!("Admin: adding version {} to {}", version.version, game_id);

    // Lấy game manifest URL
    let manifest_url = settings::get_setting(db, "manifest_url".to_string()).await?;
    let root_manifest: Manifest = if let Some(url) = manifest_url {
        download_json(&url).await?
    } else {
        return Err(anyhow!("Manifest URL not configured"));
    };

    let game_entry = root_manifest
        .games
        .iter()
        .find(|g| g.id == game_id)
        .ok_or_else(|| anyhow!("Game not found in manifest"))?;

    let game_url = game_entry.manifest_url.clone();
    let mut game_manifest: GameManifest = download_json(&game_url).await?;

    // Replace hoặc thêm version
    if let Some(v) = game_manifest.versions.iter_mut().find(|v| v.version == version.version) {
        *v = version;
    } else {
        game_manifest.versions.push(version.clone());
        // Sort giảm dần theo released_at
        game_manifest
            .versions
            .sort_by(|a, b| b.released_at.cmp(&a.released_at));
        game_manifest.latest_version = version.version;
    }

    // TODO: Upload game_manifest.json mới lên Drive
    // Hiện tại: lưu local
    let local_path = std::env::temp_dir().join(format!("{}-manifest.json", game_id));
    std::fs::write(&local_path, serde_json::to_string_pretty(&game_manifest)?)?;

    tracing::info!("Game manifest saved at: {:?}", local_path);

    Ok(())
}

// ================================================================
// ANALYTICS
// ================================================================

/// Lấy analytics - tổng hợp từ DB local + endpoint cloud
pub async fn admin_get_analytics(db: State<'_, DbState>) -> Result<AdminAnalytics> {
    validate_admin_password(&db)?;

    let now = chrono::Utc::now().timestamp();
    let one_week_ago = now - (7 * 24 * 3600);

    // Tổng lượt install
    let total_downloads = crate::db::count_telemetry_events(&db, "install", 0)?;

    // Lượt install tuần này
    let downloads_this_week = crate::db::count_telemetry_events(&db, "install", one_week_ago)?;

    // Lượt install tuần trước để tính trend
    let two_weeks_ago = now - (14 * 24 * 3600);
    let downloads_last_week = crate::db::count_telemetry_events(&db, "install", two_weeks_ago)?
        - downloads_this_week;

    let trend = if downloads_last_week > 0 {
        let pct = ((downloads_this_week as f64 - downloads_last_week as f64)
            / downloads_last_week as f64)
            * 100.0;
        format!("{:+.1}%", pct)
    } else {
        "N/A".to_string()
    };

    // Top games
    let top_games_raw = crate::db::top_games_by_downloads(&db, 5)?;
    let top_games = top_games_raw
        .into_iter()
        .map(|(id, downloads)| TopGame {
            id: id.clone(),
            // Lấy tên từ local DB hoặc manifest
            name: crate::db::get_game_local(&db, &id)
                .ok()
                .flatten()
                .map(|g| g.name)
                .unwrap_or(id),
            downloads,
        })
        .collect();

    // Active installs (đếm game có status = 'installed')
    let conn = db.conn()?;
    let active_installs: u64 = conn
        .query_row(
            "SELECT COUNT(*) FROM games WHERE status = 'installed'",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    // Unique users (đếm distinct anonymous_id trong telemetry)
    let unique_users: u64 = conn
        .query_row(
            "SELECT COUNT(DISTINCT anonymous_id) FROM telemetry",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    // Downloads 7 ngày qua
    let mut downloads_by_day = Vec::new();
    for i in (0..7).rev() {
        let day_start = now - (i * 24 * 3600);
        let day_end = day_start + (24 * 3600);
        let count: u64 = conn
            .query_row(
                "SELECT COUNT(*) FROM telemetry WHERE event = 'install'
                 AND created_at >= ?1 AND created_at < ?2",
                params![day_start, day_end],
                |row| row.get(0),
            )
            .unwrap_or(0);
        let date = chrono::DateTime::from_timestamp(day_start, 0)
            .map(|d| d.format("%d/%m").to_string())
            .unwrap_or_default();
        downloads_by_day.push(DailyDownload {
            date,
            downloads: count,
        });
    }

    // Average rating
    let average_rating: f32 = conn
        .query_row(
            "SELECT COALESCE(AVG(rating), 0) FROM comments WHERE status = 'approved'",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0.0);

    Ok(AdminAnalytics {
        total_downloads,
        active_installs,
        unique_users,
        average_rating,
        downloads_this_week,
        downloads_trend: trend,
        top_games,
        downloads_by_day,
    })
}

// ================================================================
// REPORTS / COMMENTS / REQUESTS
// ================================================================
pub async fn admin_get_reports(db: State<'_, DbState>) -> Result<Vec<BugReport>> {
    validate_admin_password(&db)?;
    let reports = crate::db::list_bug_reports(&db, None)?;

    // Convert models.rs -> admin.rs types
    Ok(reports
        .into_iter()
        .map(|r| BugReport {
            id: r.id,
            game_id: r.game_id,
            version: r.version,
            user_id: r.user_id,
            title: r.title,
            description: r.description,
            status: r.status,
            created_at: r.created_at,
        })
        .collect())
}
pub async fn admin_get_comments(db: State<'_, DbState>) -> Result<Vec<Comment>> {
    validate_admin_password(&db)?;
    let comments = crate::db::list_comments(&db, None)?;
    Ok(comments
        .into_iter()
        .map(|c| Comment {
            id: c.id,
            game_id: c.game_id,
            user_id: c.user_id,
            rating: c.rating,
            content: c.content,
            status: c.status,
            created_at: c.created_at,
        })
        .collect())
}
pub async fn admin_get_requests(db: State<'_, DbState>) -> Result<Vec<UserRequest>> {
    validate_admin_password(&db)?;
    let requests = crate::db::list_user_requests(&db)?;
    Ok(requests
        .into_iter()
        .map(|r| UserRequest {
            id: r.id,
            game_name: r.game_name,
            votes: r.votes,
            requested_by: r.requested_by,
            created_at: r.created_at,
        })
        .collect())
}

/// Duyệt/từ chối comment
pub async fn admin_update_comment_status(
    db: State<'_, DbState>,
    comment_id: String,
    status: String,
) -> Result<()> {
    validate_admin_password(&db)?;
    crate::db::update_comment_status(&db, &comment_id, &status)?;
    Ok(())
}

// ================================================================
// HELPERS
// ================================================================

fn validate_admin_password(db: &State<'_, DbState>) -> Result<()> {
    // Check session token (đã set qua setAdminUnlocked ở frontend)
    // Trong production: dùng keyring + Argon2
    let conn = db.conn()?;
    let token: Option<String> = conn
        .query_row(
            "SELECT value FROM settings WHERE key = 'admin_session'",
            [],
            |row| row.get(0),
        )
        .ok();

    match token {
        Some(t) if t == "valid" => Ok(()),
        _ => Err(anyhow!("Admin not authenticated. Set admin_session setting first.")),
    }
}

/// Download JSON từ URL
pub async fn download_json<T: for<'de> Deserialize<'de>>(url: &str) -> Result<T> {
    let url = crate::commands::drive::normalize_drive_url(url);
    let client = reqwest::Client::builder()
        .user_agent("ViethoagameManager/0.1")
        .build()?;
    let response = client.get(&url).send().await?;
    if !response.status().is_success() {
        return Err(anyhow!("HTTP {}", response.status()));
    }
    let json = response.json::<T>().await?;
    Ok(json)
}

/// SHA256 helper
#[allow(dead_code)]
fn sha256_hex(s: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(s.as_bytes());
    hex::encode(hasher.finalize())
}

// Stub để tránh warning unused
#[allow(dead_code)]
fn _unused(_path: &Path) {}