//! Settings commands

use crate::db::{self, DbState};
use anyhow::Result;
use tauri::State;
pub async fn get_setting(db: State<'_, DbState>, key: String) -> Result<Option<String>> {
    db::get_setting(&db, &key)
}
pub async fn set_setting(db: State<'_, DbState>, key: String, value: String) -> Result<()> {
    db::set_setting(&db, &key, &value)
}