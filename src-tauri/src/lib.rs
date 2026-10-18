//! Viethoagame Manager - Tauri Backend
//!
//! App quản lý và cài đặt các bản việt hóa game cho Windows.

use commands::{
    admin, drive, installer, library, scanner, settings, update,
};
use tauri::Manager;

mod commands;
mod db;
mod models;

pub use models::*;

// =====================================================================
// Wrapper functions: convert anyhow::Error -> String for Tauri commands
// =====================================================================
//
// Tauri yêu cầu command trả về Result<T, E> với E: Serialize. String là
// dạng đơn giản nhất, hiển thị trực tiếp cho user. Mỗi wrapper chỉ là
// một async fn trả về Result<T, String>, bên trong `.await?` rồi
// `.map_err(|e| e.to_string())`.

// ----- Drive -----
#[tauri::command]
async fn fetch_manifest(db: tauri::State<'_, db::DbState>) -> Result<Manifest, String> {
    drive::fetch_manifest(db).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn fetch_game_manifest(
    db: tauri::State<'_, db::DbState>,
    game_id: String,
) -> Result<GameManifest, String> {
    drive::fetch_game_manifest(db, game_id)
        .await
        .map_err(|e| e.to_string())
}

// ----- Scanner -----
#[tauri::command]
async fn scan_installed_games() -> Result<Vec<LocalGame>, String> {
    scanner::scan_installed_games().await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn detect_game_path(
    db: tauri::State<'_, db::DbState>,
    game_id: String,
) -> Result<Option<String>, String> {
    scanner::detect_game_path(db, game_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn set_custom_game_path(
    db: tauri::State<'_, db::DbState>,
    game_id: String,
    path: String,
) -> Result<(), String> {
    scanner::set_custom_game_path(db, game_id, path)
        .await
        .map_err(|e| e.to_string())
}

/// Tự động quét toàn bộ launcher (Steam/Epic/GOG) và tự điền
/// install_path cho mọi game trong manifest. Trả về kết quả chi tiết.
#[tauri::command]
async fn auto_setup_games(
    db: tauri::State<'_, db::DbState>,
) -> Result<scanner::AutoSetupResult, String> {
    scanner::auto_setup_all_games(db)
        .await
        .map_err(|e| e.to_string())
}

// ----- Library -----
#[tauri::command]
async fn get_library(db: tauri::State<'_, db::DbState>) -> Result<Vec<LocalGame>, String> {
    library::get_library(db).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_game_local(
    db: tauri::State<'_, db::DbState>,
    game_id: String,
) -> Result<Option<LocalGame>, String> {
    library::get_game_local(db, game_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn set_game_status(
    db: tauri::State<'_, db::DbState>,
    game_id: String,
    status: String,
) -> Result<(), String> {
    library::set_game_status(db, game_id, status)
        .await
        .map_err(|e| e.to_string())
}

// ----- Install -----
#[tauri::command]
async fn install_game(
    app: tauri::AppHandle,
    db: tauri::State<'_, db::DbState>,
    install_id: String,
    _drive_url: String,
) -> Result<(), String> {
    installer::install_game(app, db, install_id, _drive_url)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn uninstall_game(
    app: tauri::AppHandle,
    db: tauri::State<'_, db::DbState>,
    install_id: String,
) -> Result<(), String> {
    installer::uninstall_game(app, db, install_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn cancel_install(install_id: String) -> Result<(), String> {
    installer::cancel_install(install_id)
        .await
        .map_err(|e| e.to_string())
}

// ----- Favorites -----
#[tauri::command]
async fn toggle_favorite(
    db: tauri::State<'_, db::DbState>,
    game_id: String,
) -> Result<bool, String> {
    library::toggle_favorite(db, game_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_favorites(db: tauri::State<'_, db::DbState>) -> Result<Vec<String>, String> {
    library::get_favorites(db)
        .await
        .map_err(|e| e.to_string())
}

// ----- Settings -----
#[tauri::command]
async fn get_setting(
    db: tauri::State<'_, db::DbState>,
    key: String,
) -> Result<Option<String>, String> {
    settings::get_setting(db, key)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn set_setting(
    db: tauri::State<'_, db::DbState>,
    key: String,
    value: String,
) -> Result<(), String> {
    settings::set_setting(db, key, value)
        .await
        .map_err(|e| e.to_string())
}

// ----- Admin -----
#[tauri::command]
async fn admin_upsert_game(
    app: tauri::AppHandle,
    db: tauri::State<'_, db::DbState>,
    game_id: String,
    name: String,
    manifest_url: String,
) -> Result<(), String> {
    admin::admin_upsert_game(app, db, game_id, name, manifest_url)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn admin_delete_game(
    app: tauri::AppHandle,
    db: tauri::State<'_, db::DbState>,
    game_id: String,
) -> Result<(), String> {
    admin::admin_delete_game(app, db, game_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn admin_add_version(
    db: tauri::State<'_, db::DbState>,
    game_id: String,
    version: serde_json::Value,
) -> Result<(), String> {
    // Parse JSON to GameVersion
    let version: GameVersion = serde_json::from_value(version)
        .map_err(|e| format!("Invalid version payload: {}", e))?;
    admin::admin_add_version(db, game_id, version)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn admin_get_analytics(
    db: tauri::State<'_, db::DbState>,
) -> Result<admin::AdminAnalytics, String> {
    admin::admin_get_analytics(db)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn admin_get_reports(
    db: tauri::State<'_, db::DbState>,
) -> Result<Vec<admin::BugReport>, String> {
    admin::admin_get_reports(db)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn admin_get_comments(
    db: tauri::State<'_, db::DbState>,
) -> Result<Vec<admin::Comment>, String> {
    admin::admin_get_comments(db)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn admin_get_requests(
    db: tauri::State<'_, db::DbState>,
) -> Result<Vec<admin::UserRequest>, String> {
    admin::admin_get_requests(db)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn admin_update_comment_status(
    db: tauri::State<'_, db::DbState>,
    comment_id: i64,
    status: String,
) -> Result<(), String> {
    admin::admin_update_comment_status(db, comment_id.to_string(), status)
        .await
        .map_err(|e| e.to_string())
}

// ----- App -----
#[tauri::command]
async fn get_app_version() -> Result<String, String> {
    // update::get_app_version returns Result<String, anyhow::Error>
    update::get_app_version()
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn open_game_folder(
    _db: tauri::State<'_, db::DbState>,
    path: String,
) -> Result<(), String> {
    update::open_game_folder(_db, path)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn launch_game(
    _db: tauri::State<'_, db::DbState>,
    path: String,
) -> Result<(), String> {
    update::launch_game(_db, path)
        .await
        .map_err(|e| e.to_string())
}

/// Entry point của Tauri app
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Khởi tạo logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    tauri::Builder::default()
        // Plugins
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_http::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_updater::Builder::new().build())
        // Setup hook - khởi tạo DB khi app start
        .setup(|app| {
            // Lấy app data dir
            let app_data_dir = app
                .path()
                .app_data_dir()
                .expect("Failed to get app data dir");

            // Tạo folder nếu chưa có
            std::fs::create_dir_all(&app_data_dir)?;

            // Khởi tạo SQLite DB
            let db_path = app_data_dir.join("viethoagame.db");
            db::init(&db_path).expect("Failed to init database");

            // Lưu db_path vào state
            app.manage(db::DbState::new(db_path));

            tracing::info!("Viethoagame Manager started");
            tracing::info!("App data dir: {:?}", app_data_dir);

            Ok(())
        })
        // Đăng ký Tauri commands (sử dụng wrapper tại đây)
        .invoke_handler(tauri::generate_handler![
            // Drive
            fetch_manifest,
            fetch_game_manifest,
            // Scanner
            scan_installed_games,
            detect_game_path,
            set_custom_game_path,
            auto_setup_games,
            // Library
            get_library,
            get_game_local,
            set_game_status,
            // Install
            install_game,
            uninstall_game,
            cancel_install,
            // Favorites
            toggle_favorite,
            get_favorites,
            // Settings
            get_setting,
            set_setting,
            // Admin
            admin_upsert_game,
            admin_delete_game,
            admin_add_version,
            admin_get_analytics,
            admin_get_reports,
            admin_get_comments,
            admin_get_requests,
            admin_update_comment_status,
            // App
            get_app_version,
            open_game_folder,
            launch_game,
        ])
        .run(tauri::generate_context!())
        .expect("Error while running tauri application");
}
