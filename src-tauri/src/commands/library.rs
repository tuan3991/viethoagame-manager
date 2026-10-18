//! Library commands - thao tác với local DB

use crate::db::{self, DbState};
use crate::models::LocalGame;
use anyhow::Result;
use tauri::State;
pub async fn get_library(db: State<'_, DbState>) -> Result<Vec<LocalGame>> {
    db::get_library(&db)
}
pub async fn get_game_local(
    db: State<'_, DbState>,
    game_id: String,
) -> Result<Option<LocalGame>> {
    db::get_game_local(&db, &game_id)
}
pub async fn set_game_status(
    db: State<'_, DbState>,
    game_id: String,
    status: String,
) -> Result<()> {
    let conn = db.conn()?;
    conn.execute(
        "UPDATE games SET status = ?1, last_checked = ?2 WHERE id = ?3",
        rusqlite::params![status, chrono::Utc::now().timestamp(), game_id],
    )?;
    Ok(())
}
pub async fn toggle_favorite(db: State<'_, DbState>, game_id: String) -> Result<bool> {
    db::toggle_favorite(&db, &game_id)
}
pub async fn get_favorites(db: State<'_, DbState>) -> Result<Vec<String>> {
    db::get_favorites(&db)
}