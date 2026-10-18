//! Commands cho installer - download, extract, install việt hóa

use crate::commands::drive::download_file_with_progress;
use crate::commands::scanner::detect_game_path;
use crate::db::{self, DbState};
use crate::models::*;
use anyhow::{anyhow, Context, Result};
use std::path::{Path, PathBuf};
use tauri::{AppHandle, Manager, State};

/// Cancel flag cho download hiện tại
static CANCEL_FLAG: once_cell::sync::Lazy<std::sync::Mutex<std::collections::HashSet<String>>> =
    once_cell::sync::Lazy::new(|| std::sync::Mutex::new(Default::default()));
pub async fn cancel_install(game_id: String) -> Result<()> {
    CANCEL_FLAG.lock().unwrap().insert(game_id);
    Ok(())
}

/// Cài việt hóa cho game
pub async fn install_game(
    app: AppHandle,
    db: State<'_, DbState>,
    game_id: String,
    version: String,
) -> Result<()> {
    tracing::info!("Installing {} version {}", game_id, version);

    // 1. Lấy manifest
    let manifest = crate::commands::drive::fetch_game_manifest(db.clone(), game_id.clone()).await?;

    let game_version = manifest
        .versions
        .iter()
        .find(|v| v.version == version)
        .ok_or_else(|| anyhow!("Version {} not found", version))?;

    // 2. Detect đường dẫn game
    let install_path = if let Some(p) = detect_game_path(db.clone(), game_id.clone()).await? {
        p
    } else {
        // Fallback: lấy từ local DB
        let local = db::get_game_local(&db, &game_id)?
            .ok_or_else(|| anyhow!("Game path not found. Please set manually."))?;
        local
            .install_path
            .ok_or_else(|| anyhow!("No install path"))?
    };

    let game_path = PathBuf::from(&install_path);
    if !game_path.exists() {
        return Err(anyhow!("Game path does not exist: {:?}", game_path));
    }

    // 3. Download file việt hóa
    let cache_dir = app
        .path()
        .app_cache_dir()
        .context("Failed to get cache dir")?
        .join("downloads");
    std::fs::create_dir_all(&cache_dir)?;

    let download_url = crate::commands::drive::normalize_drive_url(&game_version.download_url);
    let archive_path = cache_dir.join(format!("{}-{}.zip", game_id, version));

    download_file_with_progress(
        &app,
        &download_url,
        &archive_path,
        &game_id,
        &version,
        &game_version.checksum,
    )
    .await
    .context("Download failed")?;

    // Check cancel
    if CANCEL_FLAG.lock().unwrap().remove(&game_id) {
        std::fs::remove_file(&archive_path).ok();
        return Err(anyhow!("Installation cancelled"));
    }

    // 4. Extract và copy files
    install_from_archive(&app, &db, &manifest, game_version, &archive_path, &game_path, &game_id)
        .await?;

    // 5. Update DB
    db::update_install_info(
        &db,
        &game_id,
        &version,
        &install_path,
        "installed",
    )?;

    db::add_install_history(&db, &game_id, &version, "install", true, None)?;

    // 6. Log telemetry (opt-in)
    log_telemetry_event(&app, &db, "install", &game_id, &version).await;

    tracing::info!("Installation complete: {} v{}", game_id, version);

    // Notification
    use tauri_plugin_notification::NotificationExt;
    let _ = app
        .notification()
        .builder()
        .title("Cài đặt thành công")
        .body(format!("Đã cài {} phiên bản {}", manifest.name, version))
        .show();

    Ok(())
}

/// Extract archive và apply install rules
async fn install_from_archive(
    _app: &AppHandle,
    db: &State<'_, DbState>,
    manifest: &GameManifest,
    game_version: &GameVersion,
    archive_path: &Path,
    game_path: &Path,
    game_id: &str,
) -> Result<()> {
    // 1. Pre-install hooks
    for hook in manifest.install.pre_install.clone().unwrap_or_default() {
        match hook {
            InstallHook::KillProcess { process_names } => {
                kill_processes(&process_names)?;
            }
            InstallHook::Verify { check_files: _ } => {}
            InstallHook::OpenUrl { url: _ } => {}
        }
    }

    // 2. Extract tới temp folder
    let extract_dir = std::env::temp_dir().join(format!("viethoagame-{}", game_id));
    if extract_dir.exists() {
        std::fs::remove_dir_all(&extract_dir).ok();
    }
    std::fs::create_dir_all(&extract_dir)?;

    extract_zip(archive_path, &extract_dir).context("Failed to extract archive")?;

    // 3. Apply install rules
    for rule in &manifest.install.files {
        apply_install_rule(db, game_id, game_version, rule, &extract_dir, game_path)?;
    }

    // 4. Post-install hooks
    for hook in manifest.install.post_install.clone().unwrap_or_default() {
        match hook {
            InstallHook::Verify { check_files } => {
                for file in check_files {
                    let resolved = resolve_placeholders(&file, game_path);
                    if !resolved.exists() {
                        return Err(anyhow!(
                            "Post-install verification failed: file not found: {:?}",
                            resolved
                        ));
                    }
                }
            }
            InstallHook::KillProcess { process_names: _ } => {}
            InstallHook::OpenUrl { url: _ } => {}
        }
    }

    // 5. Cleanup
    std::fs::remove_dir_all(&extract_dir).ok();

    Ok(())
}

/// Apply 1 install rule
fn apply_install_rule(
    db: &State<'_, DbState>,
    game_id: &str,
    game_version: &GameVersion,
    rule: &InstallRule,
    extract_dir: &Path,
    game_path: &Path,
) -> Result<()> {
    let source = extract_dir.join(&rule.source.replace('*', ""));
    let target = resolve_placeholders(&rule.target, game_path);

    tracing::info!(
        "Install rule: {} -> {} (backup: {})",
        source.display(),
        target.display(),
        rule.backup_original
    );

    if rule.backup_original {
        backup_files(db, game_id, game_version, &target)?;
    }

    std::fs::create_dir_all(&target)?;
    copy_recursive(&source, &target, rule.overwrite.unwrap_or(true))?;

    Ok(())
}

/// Backup files từ target
fn backup_files(
    db: &State<'_, DbState>,
    game_id: &str,
    game_version: &GameVersion,
    target: &Path,
) -> Result<()> {
    if !target.exists() {
        return Ok(());
    }

    let backup_base = backup_dir(game_id, &game_version.version)?;
    std::fs::create_dir_all(&backup_base)?;

    for entry in walkdir::WalkDir::new(target) {
        let entry = entry?;
        if entry.file_type().is_file() {
            let file = entry.path();
            let relative = file.strip_prefix(target).unwrap_or(file);
            let backup_path = backup_base.join(relative);

            if let Some(parent) = backup_path.parent() {
                std::fs::create_dir_all(parent)?;
            }

            let hash = crate::commands::drive::sha256_file(file).unwrap_or_default();
            std::fs::copy(file, &backup_path)?;

            // Record vào DB
            let _ = db::add_backup(
                db,
                game_id,
                &game_version.version,
                &backup_path.to_string_lossy(),
                &file.to_string_lossy(),
                &hash,
            );
        }
    }

    Ok(())
}

/// Gỡ cài đặt - restore backup
pub async fn uninstall_game(
    app: AppHandle,
    db: State<'_, DbState>,
    game_id: String,
) -> Result<()> {
    tracing::info!("Uninstalling {}", game_id);

    let local = db::get_game_local(&db, &game_id)?
        .ok_or_else(|| anyhow!("Game not found in local DB"))?;

    let install_path = local
        .install_path
        .ok_or_else(|| anyhow!("No install path recorded"))?;

    // Restore từ backup
    let backups = db::get_backups_for_game(&db, &game_id)?;
    for (backup_path, original_path, _hash) in backups {
        let backup_p = PathBuf::from(&backup_path);
        let original_p = PathBuf::from(&original_path);

        if backup_p.exists() {
            if let Some(parent) = original_p.parent() {
                std::fs::create_dir_all(parent).ok();
            }
            std::fs::copy(&backup_p, &original_p).ok();
        }
    }

    // Xoá backup records
    db::delete_backups(&db, &game_id)?;

    // Update DB
    db::update_install_info(
        &db,
        &game_id,
        "",
        &install_path,
        "not_installed",
    )?;

    db::add_install_history(&db, &game_id, &local.version, "uninstall", true, None)?;

    // Notification
    use tauri_plugin_notification::NotificationExt;
    let _ = app
        .notification()
        .builder()
        .title("Đã gỡ cài đặt")
        .body(format!("Đã gỡ bản việt hóa của {}", local.name))
        .show();

    Ok(())
}

// === Utility functions ===

fn backup_dir(game_id: &str, version: &str) -> Result<PathBuf> {
    let app_data = dirs::data_local_dir()
        .ok_or_else(|| anyhow!("Cannot get data_local_dir"))?
        .join("ViethoagameManager")
        .join("backups")
        .join(game_id)
        .join(version);
    Ok(app_data)
}

/// Resolve placeholders trong target path
/// {gamePath} -> install path
fn resolve_placeholders(template: &str, game_path: &Path) -> PathBuf {
    let mut result = template.to_string();
    if result.contains("{gamePath}") {
        result = result.replace("{gamePath}", &game_path.to_string_lossy());
    }
    // Thêm các placeholder khác sau nếu cần
    PathBuf::from(result)
}

/// Extract zip với progress (cơ bản)
fn extract_zip(zip_path: &Path, dest: &Path) -> Result<()> {
    let file = std::fs::File::open(zip_path)?;
    let mut archive = zip::ZipArchive::new(file)?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let outpath = dest.join(file.name());

        if file.name().ends_with('/') {
            std::fs::create_dir_all(&outpath)?;
        } else {
            if let Some(p) = outpath.parent() {
                std::fs::create_dir_all(p)?;
            }
            let mut outfile = std::fs::File::create(&outpath)?;
            std::io::copy(&mut file, &mut outfile)?;
        }
    }

    Ok(())
}

/// Copy recursive từ src -> dest
fn copy_recursive(src: &Path, dest: &Path, overwrite: bool) -> Result<()> {
    if !src.exists() {
        return Err(anyhow!("Source does not exist: {:?}", src));
    }

    if src.is_file() {
        if dest.exists() && !overwrite {
            return Ok(());
        }
        if let Some(parent) = dest.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::copy(src, dest)?;
        return Ok(());
    }

    // Directory
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let src_path = entry.path();
        let file_name = entry.file_name();
        let dest_path = dest.join(&file_name);

        if file_type.is_dir() {
            std::fs::create_dir_all(&dest_path)?;
            copy_recursive(&src_path, &dest_path, overwrite)?;
        } else if file_type.is_file() {
            if dest_path.exists() && !overwrite {
                continue;
            }
            std::fs::copy(&src_path, &dest_path)?;
        }
    }

    Ok(())
}

/// Kill processes theo tên
fn kill_processes(names: &[String]) -> Result<()> {
    use sysinfo::{ProcessRefreshKind, RefreshKind, System};

    let mut sys = System::new_with_specifics(
        RefreshKind::new().with_processes(ProcessRefreshKind::everything()),
    );

    for name in names {
        let name_lower = name.to_lowercase();
        for (pid, process) in sys.processes() {
            if process.name().to_string_lossy().to_lowercase() == name_lower {
                tracing::info!("Killing process: {} ({})", name, pid);
                process.kill();
            }
        }
    }

    Ok(())
}

/// Log telemetry event (nếu user opt-in)
async fn log_telemetry_event(
    app: &AppHandle,
    db: &State<'_, DbState>,
    event: &str,
    game_id: &str,
    version: &str,
) {
    // Check opt-in
    let enabled = db::get_setting(db, "telemetry_enabled")
        .ok()
        .flatten()
        .map(|v| v == "true")
        .unwrap_or(false);

    if !enabled {
        return;
    }

    // Lấy anonymous ID (rotate mỗi 30 ngày)
    let anon_id = db::get_setting(db, "anonymous_id")
        .ok()
        .flatten()
        .unwrap_or_else(|| {
            let new_id = uuid::Uuid::new_v4().to_string();
            let _ = db::set_setting(db, "anonymous_id", &new_id);
            new_id
        });

    // Log local
    let _ = db::log_telemetry(db, event, Some(game_id), Some(version), &anon_id, None);

    // Gửi lên server (best-effort, không block)
    let payload = serde_json::json!({
        "event": event,
        "gameId": game_id,
        "version": version,
        "anonymousId": anon_id,
        "appVersion": env!("CARGO_PKG_VERSION"),
        "timestamp": chrono::Utc::now().timestamp(),
    });

    let endpoint = db::get_setting(db, "telemetry_endpoint")
        .ok()
        .flatten();

    if let Some(url) = endpoint {
        let _ = app.run_on_main_thread(move || {
            // Spawn background task
            tauri::async_runtime::spawn(async move {
                let client = reqwest::Client::new();
                let _ = client
                    .post(&url)
                    .json(&payload)
                    .send()
                    .await;
            });
        });
    }
}