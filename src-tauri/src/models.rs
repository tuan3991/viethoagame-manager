//! Data models cho game manifest và local DB

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Manifest {
    pub version: u32,
    pub last_updated: String,
    pub games: Vec<ManifestEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManifestEntry {
    pub id: String,
    pub name: String,
    pub manifest_url: String,
    /// Ảnh bìa hiển thị ở card (3:2 banner), optional
    #[serde(default)]
    pub cover_image: Option<String>,
    /// Ảnh đại diện vuông/icon (1:1), optional
    #[serde(default)]
    pub avatar_image: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameManifest {
    pub id: String,
    pub name: String,
    pub short_description: String,
    pub long_description: String,
    pub developer: String,
    pub publisher: String,
    pub release_date: String,
    pub genres: Vec<String>,
    pub tags: Vec<String>,
    pub cover_image: String,
    pub screenshots: Vec<String>,
    pub video_url: Option<String>,
    pub system_requirements: SystemRequirements,
    pub supported_engines: Vec<String>,
    pub latest_version: String,
    pub versions: Vec<GameVersion>,
    pub detect: GameDetection,
    pub install: InstallConfig,
    pub uninstall: Option<UninstallConfig>,
    pub stats: Option<GameStats>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemRequirements {
    pub min: String,
    pub rec: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameVersion {
    pub version: String,
    pub released_at: String,
    pub size_bytes: u64,
    pub download_url: String,
    pub checksum: String,
    pub changelog: Option<String>,
    pub is_beta: Option<bool>,
    pub is_broken: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameDetection {
    pub launchers: Vec<String>,
    pub steam_app_id: Option<String>,
    pub epic_app_name: Option<String>,
    pub gog_app_id: Option<String>,
    pub executable_relative: Option<String>,
    pub custom_folders: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallConfig {
    pub files: Vec<InstallRule>,
    pub pre_install: Option<Vec<InstallHook>>,
    pub post_install: Option<Vec<InstallHook>>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallRule {
    pub source: String,
    pub target: String,
    pub backup_original: bool,
    pub overwrite: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum InstallHook {
    #[serde(rename = "kill_process")]
    KillProcess { process_names: Vec<String> },
    #[serde(rename = "verify")]
    Verify { check_files: Vec<String> },
    #[serde(rename = "open_url")]
    OpenUrl { url: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UninstallConfig {
    pub restore_backup: bool,
    pub remove_added_files: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameStats {
    pub downloads: u64,
    pub active_installs: u64,
    pub average_rating: f32,
    pub rating_count: u32,
}

// === Local DB models ===

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalGame {
    pub id: String,
    pub name: String,
    pub version: String,
    pub installed_version: Option<String>,
    pub install_path: Option<String>,
    pub installed_at: Option<i64>,
    pub last_checked: i64,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadProgressPayload {
    pub game_id: String,
    pub version: String,
    pub downloaded_bytes: u64,
    pub total_bytes: u64,
    pub speed_bytes_per_sec: u64,
    pub status: String,
    pub message: Option<String>,
}

// === Admin models ===

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminBugReport {
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
pub struct AdminComment {
    pub id: String,
    pub game_id: String,
    pub user_id: String,
    pub rating: u8,
    pub content: String,
    pub status: String,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminUserRequest {
    pub id: String,
    pub game_name: String,
    pub votes: u32,
    pub requested_by: String,
    pub created_at: i64,
}