//! Scanner - phát hiện game đã cài trên Steam, Epic, GOG

use crate::db::DbState;
use crate::models::LocalGame;
use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tauri::State;

#[cfg(windows)]
use winreg::enums::*;
#[cfg(windows)]
use winreg::RegKey;

/// Kết quả detect
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectedGame {
    pub id: String,
    pub name: String,
    pub install_path: String,
    pub launcher: String, // 'steam' | 'epic' | 'gog' | 'manual'
    pub app_id: Option<String>,
}

/// Quét tất cả launcher, trả về danh sách game đã cài
pub async fn scan_installed_games() -> Result<Vec<LocalGame>> {
    let mut all = Vec::new();
    all.extend(scan_steam().unwrap_or_default());
    all.extend(scan_epic().unwrap_or_default());
    all.extend(scan_gog().unwrap_or_default());

    tracing::info!("Total detected games: {}", all.len());
    Ok(all.into_iter().map(|g| g.into()).collect())
}

/// Detect đường dẫn game theo game_id + cách detect trong manifest
pub async fn detect_game_path(db: State<'_, DbState>, game_id: String) -> Result<Option<String>> {
    // Lấy game manifest từ Drive
    let manifest = crate::commands::drive::fetch_game_manifest(db.clone(), game_id.clone()).await?;

    // Thử từng launcher
    for launcher in &manifest.detect.launchers {
        match launcher.as_str() {
            "steam" => {
                if let Some(app_id) = &manifest.detect.steam_app_id {
                    if let Some(path) = find_steam_game(app_id, &manifest.name) {
                        return Ok(Some(path));
                    }
                }
            }
            "epic" => {
                if let Some(app_name) = &manifest.detect.epic_app_name {
                    if let Some(path) = find_epic_game(app_name) {
                        return Ok(Some(path));
                    }
                }
            }
            "gog" => {
                if let Some(app_id) = &manifest.detect.gog_app_id {
                    if let Some(path) = find_gog_game(app_id) {
                        return Ok(Some(path));
                    }
                }
            }
            _ => {}
        }
    }

    Ok(None)
}

/// User set đường dẫn thủ công
pub async fn set_custom_game_path(
    db: State<'_, DbState>,
    game_id: String,
    path: String,
) -> Result<()> {
    crate::db::upsert_game(
        &db,
        &LocalGame {
            id: game_id.clone(),
            name: game_id, // Sẽ được update từ manifest sau
            version: String::new(),
            installed_version: None,
            install_path: Some(path),
            installed_at: None,
            last_checked: chrono::Utc::now().timestamp(),
            status: "manual".to_string(),
        },
    )?;
    Ok(())
}

// ============================================
// Steam Scanner
// ============================================

#[cfg(windows)]
fn scan_steam() -> Result<Vec<DetectedGame>> {
    let steam_path = read_steam_install_path()?;
    let steamapps = steam_path.join("steamapps");

    if !steamapps.exists() {
        return Ok(vec![]);
    }

    // Đọc libraryfolders.vdf để biết tất cả Steam library
    let library_folders_vdf = steamapps.join("libraryfolders.vdf");
    let mut library_paths = vec![steam_path.clone()];

    if library_folders_vdf.exists() {
        let content = std::fs::read_to_string(&library_folders_vdf)?;
        // Parse VDF (đơn giản - tìm "path" keys)
        for line in content.lines() {
            if line.contains("\"path\"") {
                if let Some(path_str) = extract_quoted_value(line) {
                    let path = PathBuf::from(path_str.replace("\\\\", "\\"));
                    if path.exists() {
                        library_paths.push(path);
                    }
                }
            }
        }
    }

    let mut games = Vec::new();

    for lib_path in library_paths {
        let lib_steamapps = lib_path.join("steamapps");
        if !lib_steamapps.exists() {
            continue;
        }

        // Đọc tất cả appmanifest_*.acf
        for entry in std::fs::read_dir(&lib_steamapps)? {
            let entry = entry?;
            let file_name = entry.file_name();
            let name_str = file_name.to_string_lossy();

            if name_str.starts_with("appmanifest_") && name_str.ends_with(".acf") {
                if let Some(game) = parse_appmanifest(&entry.path()) {
                    games.push(DetectedGame {
                        id: format!("steam-{}", game.appid),
                        name: game.name,
                        install_path: game.installdir,
                        launcher: "steam".to_string(),
                        app_id: Some(game.appid),
                    });
                }
            }
        }
    }

    Ok(games)
}

#[cfg(not(windows))]
fn scan_steam() -> Result<Vec<DetectedGame>> {
    Ok(vec![])
}

#[cfg(windows)]
fn read_steam_install_path() -> Result<PathBuf> {
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let key = hklm
        .open_subkey("SOFTWARE\\WOW6432Node\\Valve\\Steam")
        .or_else(|_| hklm.open_subkey("SOFTWARE\\Valve\\Steam"))
        .context("Steam registry key not found")?;

    let install_path: String = key.get_value("InstallPath")?;
    Ok(PathBuf::from(install_path))
}

#[cfg(not(windows))]
fn read_steam_install_path() -> Result<PathBuf> {
    Err(anyhow!("Steam not supported on this platform"))
}

/// Parse file appmanifest_*.acf (VDF format đơn giản)
struct SteamAppManifest {
    appid: String,
    name: String,
    installdir: String,
}

fn parse_appmanifest(path: &std::path::Path) -> Option<SteamAppManifest> {
    let content = std::fs::read_to_string(path).ok()?;

    let mut appid = None;
    let mut name = None;
    let mut installdir = None;

    for line in content.lines() {
        if line.contains("\"appid\"") {
            appid = extract_quoted_value(line);
        } else if line.contains("\"name\"") {
            name = extract_quoted_value(line);
        } else if line.contains("\"installdir\"") {
            installdir = extract_quoted_value(line);
        }
    }

    Some(SteamAppManifest {
        appid: appid?,
        name: name?,
        installdir: installdir?,
    })
}

fn find_steam_game(app_id: &str, _name: &str) -> Option<String> {
    // Tìm trong tất cả library
    let detected = scan_steam().ok()?;
    detected
        .into_iter()
        .find(|g| g.app_id.as_deref() == Some(app_id))
        .map(|g| g.install_path)
}

// ============================================
// Epic Scanner
// ============================================

#[cfg(windows)]
fn scan_epic() -> Result<Vec<DetectedGame>> {
    // Epic Games thường ở:
    // - %PROGRAMFILES%\Epic Games\
    // - C:\Program Files\Epic Games\
    // - User có thể set custom trong launcher

    let program_files = std::env::var("ProgramFiles").unwrap_or_else(|_| "C:\\Program Files".into());
    let epic_base = PathBuf::from(&program_files).join("Epic Games");

    if !epic_base.exists() {
        return Ok(vec![]);
    }

    let mut games = Vec::new();

    for entry in std::fs::read_dir(&epic_base)? {
        let entry = entry?;
        if !entry.file_type()?.is_dir() {
            continue;
        }

        let game_dir = entry.path();
        let game_name = entry.file_name().to_string_lossy().to_string();

        // Mỗi game có folder *.egstore với file *.info
        let egstore = game_dir.join(format!("{}.egstore", game_name.replace(' ', "")));
        if egstore.exists() {
            // Tìm file .info
            if let Ok(entries) = std::fs::read_dir(&egstore) {
                for e in entries.flatten() {
                    if e.path().extension().map(|x| x == "info").unwrap_or(false) {
                        games.push(DetectedGame {
                            id: format!("epic-{}", slugify(&game_name)),
                            name: game_name.clone(),
                            install_path: game_dir.to_string_lossy().to_string(),
                            launcher: "epic".to_string(),
                            app_id: None,
                        });
                        break;
                    }
                }
            }
        }
    }

    Ok(games)
}

#[cfg(not(windows))]
fn scan_epic() -> Result<Vec<DetectedGame>> {
    Ok(vec![])
}

fn find_epic_game(app_name: &str) -> Option<String> {
    let detected = scan_epic().ok()?;
    detected
    .into_iter()
    .find(|g| g.name.to_lowercase().contains(&app_name.to_lowercase()))
    .map(|g| g.install_path)
}

// ============================================
// GOG Scanner
// ============================================

fn scan_gog() -> Result<Vec<DetectedGame>> {
    // GOG Galaxy ở:
    // - %PROGRAMFILES(x86)%\GOG Galaxy\Games\
    // - Mỗi game có file goggame-*.info

    let program_files_x86 =
        std::env::var("ProgramFiles(x86)").unwrap_or_else(|_| "C:\\Program Files (x86)".into());
    let gog_base = PathBuf::from(&program_files_x86)
        .join("GOG Galaxy")
        .join("Games");

    if !gog_base.exists() {
        return Ok(vec![]);
    }

    let mut games = Vec::new();

    for entry in std::fs::read_dir(&gog_base)? {
        let entry = entry?;
        if !entry.file_type()?.is_dir() {
            continue;
        }

        let game_dir = entry.path();
        let game_name = entry.file_name().to_string_lossy().to_string();

        // Tìm file goggame-*.info
        for e in std::fs::read_dir(&game_dir)?.flatten() {
            let name = e.file_name().to_string_lossy().to_string();
            if name.starts_with("goggame-") && name.ends_with(".info") {
                let app_id = name
                    .trim_start_matches("goggame-")
                    .trim_end_matches(".info")
                    .to_string();

                games.push(DetectedGame {
                    id: format!("gog-{}", app_id),
                    name: parse_gog_info_name(&e.path()).unwrap_or_else(|| game_name.clone()),
                    install_path: game_dir.to_string_lossy().to_string(),
                    launcher: "gog".to_string(),
                    app_id: Some(app_id),
                });
                break;
            }
        }
    }

    Ok(games)
}

fn parse_gog_info_name(path: &std::path::Path) -> Option<String> {
    #[derive(Deserialize)]
    struct GogInfo {
        name: String,
    }
    let content = std::fs::read_to_string(path).ok()?;
    serde_json::from_str::<GogInfo>(&content)
        .ok()
        .map(|i| i.name)
}

fn find_gog_game(app_id: &str) -> Option<String> {
    let detected = scan_gog().ok()?;
    detected
        .into_iter()
        .find(|g| g.app_id.as_deref() == Some(app_id))
        .map(|g| g.install_path)
}

// ============================================
// Helpers
// ============================================

fn extract_quoted_value(line: &str) -> Option<String> {
    let first_quote = line.find('"')?;
    let after_first = &line[first_quote + 1..];
    let second_quote = after_first.find('"')?;
    let key = &after_first[..second_quote];
    let after_key = &after_first[second_quote + 1..];

    // Tìm value quote tiếp theo
    let val_first = after_key.find('"')?;
    let after_val_first = &after_key[val_first + 1..];
    let val_second = after_val_first.find('"')?;
    let value = &after_val_first[..val_second];

    // Đây là value, không phải key
    if key == "path" || key == "name" || key == "appid" || key == "installdir" {
        Some(value.to_string())
    } else {
        None
    }
}

fn slugify(s: &str) -> String {
    s.to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .trim_matches('-')
        .to_string()
}

// Convert DetectedGame -> LocalGame
impl From<DetectedGame> for LocalGame {
    fn from(d: DetectedGame) -> Self {
        LocalGame {
            id: d.id.clone(),
            name: d.name,
            version: String::new(),
            installed_version: None,
            install_path: Some(d.install_path),
            installed_at: None,
            last_checked: chrono::Utc::now().timestamp(),
            status: "not_installed".to_string(),
        }
    }
}