//! App-level commands

use crate::db::DbState;
use crate::models::LocalGame;
use anyhow::Result;
use std::path::PathBuf;
use tauri::State;

const APP_VERSION: &str = env!("CARGO_PKG_VERSION");
pub async fn get_app_version() -> Result<String> {
    Ok(APP_VERSION.to_string())
}
pub async fn open_game_folder(db: State<'_, DbState>, game_id: String) -> Result<()> {
    let local = crate::db::get_game_local(&db, &game_id)?
        .ok_or_else(|| anyhow::anyhow!("Game not found"))?;

    let path = local
        .install_path
        .ok_or_else(|| anyhow::anyhow!("No install path"))?;

    open_in_explorer(&PathBuf::from(path))
}
pub async fn launch_game(db: State<'_, DbState>, game_id: String) -> Result<()> {
    let local = crate::db::get_game_local(&db, &game_id)?
        .ok_or_else(|| anyhow::anyhow!("Game not found"))?;

    let path = local
        .install_path
        .ok_or_else(|| anyhow::anyhow!("No install path"))?;

    // Mở folder game (user sẽ tự chạy exe)
    // Hoặc sau này: detect executable và chạy trực tiếp
    open_in_explorer(&PathBuf::from(path))
}

#[cfg(windows)]
fn open_in_explorer(path: &PathBuf) -> Result<()> {
    use std::process::Command;
    Command::new("explorer").arg(path).spawn()?;
    Ok(())
}

#[cfg(target_os = "macos")]
fn open_in_explorer(path: &PathBuf) -> Result<()> {
    use std::process::Command;
    Command::new("open").arg(path).spawn()?;
    Ok(())
}

#[cfg(all(unix, not(target_os = "macos")))]
fn open_in_explorer(path: &PathBuf) -> Result<()> {
    use std::process::Command;
    Command::new("xdg-open").arg(path).spawn()?;
    Ok(())
}

// Bỏ warning
#[allow(dead_code)]
fn _unused(_: LocalGame) {}