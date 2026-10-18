//! Database module - SQLite cho local data

use anyhow::Result;
use rusqlite::{params, Connection};
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::State;

pub mod schema;

/// State chứa connection tới DB, share qua Tauri
pub struct DbState {
    pub path: PathBuf,
}

impl DbState {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    /// Mở connection (mỗi thread nên mở riêng)
    pub fn conn(&self) -> Result<Connection> {
        Ok(Connection::open(&self.path)?)
    }
}

/// Khởi tạo DB - chạy schema
pub fn init(db_path: &PathBuf) -> Result<()> {
    let conn = Connection::open(db_path)?;
    conn.execute_batch(schema::SCHEMA)?;
    tracing::info!("Database initialized at {:?}", db_path);
    Ok(())
}

// === Helper functions ===

pub fn get_library(db: &State<'_, DbState>) -> Result<Vec<crate::models::LocalGame>> {
    let conn = db.conn()?;
    let mut stmt = conn.prepare(
        "SELECT id, name, version, installed_version, install_path, installed_at, last_checked, status
         FROM games
         ORDER BY name"
    )?;

    let rows = stmt.query_map([], |row| {
        Ok(crate::models::LocalGame {
            id: row.get(0)?,
            name: row.get(1)?,
            version: row.get(2)?,
            installed_version: row.get(3)?,
            install_path: row.get(4)?,
            installed_at: row.get(5)?,
            last_checked: row.get(6)?,
            status: row.get(7)?,
        })
    })?;

    Ok(rows.collect::<Result<Vec<_>, _>>()?)
}

pub fn get_game_local(
    db: &State<'_, DbState>,
    game_id: &str,
) -> Result<Option<crate::models::LocalGame>> {
    let conn = db.conn()?;
    let mut stmt = conn.prepare(
        "SELECT id, name, version, installed_version, install_path, installed_at, last_checked, status
         FROM games WHERE id = ?1"
    )?;

    let result = stmt.query_row(params![game_id], |row| {
        Ok(crate::models::LocalGame {
            id: row.get(0)?,
            name: row.get(1)?,
            version: row.get(2)?,
            installed_version: row.get(3)?,
            install_path: row.get(4)?,
            installed_at: row.get(5)?,
            last_checked: row.get(6)?,
            status: row.get(7)?,
        })
    });

    match result {
        Ok(g) => Ok(Some(g)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e.into()),
    }
}

pub fn upsert_game(db: &State<'_, DbState>, game: &crate::models::LocalGame) -> Result<()> {
    let conn = db.conn()?;
    conn.execute(
        "INSERT INTO games (id, name, version, installed_version, install_path, installed_at, last_checked, status)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
         ON CONFLICT(id) DO UPDATE SET
           name = excluded.name,
           version = excluded.version,
           last_checked = excluded.last_checked,
           status = CASE WHEN games.status = 'installed' THEN 'installed' ELSE excluded.status END
        ",
        params![
            game.id,
            game.name,
            game.version,
            game.installed_version,
            game.install_path,
            game.installed_at,
            game.last_checked,
            game.status,
        ],
    )?;
    Ok(())
}

pub fn update_install_info(
    db: &State<'_, DbState>,
    game_id: &str,
    installed_version: &str,
    install_path: &str,
    status: &str,
) -> Result<()> {
    let conn = db.conn()?;
    let now = chrono::Utc::now().timestamp();
    conn.execute(
        "UPDATE games SET installed_version = ?1, install_path = ?2, status = ?3,
         installed_at = ?4, last_checked = ?4 WHERE id = ?5",
        params![installed_version, install_path, status, now, game_id],
    )?;
    Ok(())
}

pub fn add_install_history(
    db: &State<'_, DbState>,
    game_id: &str,
    version: &str,
    action: &str,
    success: bool,
    error: Option<&str>,
) -> Result<()> {
    let conn = db.conn()?;
    let now = chrono::Utc::now().timestamp();
    conn.execute(
        "INSERT INTO install_history (game_id, version, action, success, error_message, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![game_id, version, action, success as i32, error, now],
    )?;
    Ok(())
}

pub fn add_backup(
    db: &State<'_, DbState>,
    game_id: &str,
    version: &str,
    file_path: &str,
    original_path: &str,
    file_hash: &str,
) -> Result<()> {
    let conn = db.conn()?;
    let now = chrono::Utc::now().timestamp();
    conn.execute(
        "INSERT INTO backups (game_id, version, file_path, original_path, file_hash, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![game_id, version, file_path, original_path, file_hash, now],
    )?;
    Ok(())
}

pub fn get_backups_for_game(
    db: &State<'_, DbState>,
    game_id: &str,
) -> Result<Vec<(String, String, String)>> {
    let conn = db.conn()?;
    let mut stmt = conn.prepare(
        "SELECT file_path, original_path, file_hash FROM backups WHERE game_id = ?1 ORDER BY created_at DESC"
    )?;
    let rows = stmt.query_map(params![game_id], |row| {
        Ok((row.get(0)?, row.get(1)?, row.get(2)?))
    })?;
    Ok(rows.collect::<Result<Vec<_>, _>>()?)
}

pub fn delete_backups(db: &State<'_, DbState>, game_id: &str) -> Result<()> {
    let conn = db.conn()?;
    conn.execute("DELETE FROM backups WHERE game_id = ?1", params![game_id])?;
    Ok(())
}

pub fn toggle_favorite(db: &State<'_, DbState>, game_id: &str) -> Result<bool> {
    let conn = db.conn()?;
    let exists: bool = conn
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM favorites WHERE game_id = ?1)",
            params![game_id],
            |row| row.get(0),
        )
        .unwrap_or(false);

    if exists {
        conn.execute("DELETE FROM favorites WHERE game_id = ?1", params![game_id])?;
        Ok(false)
    } else {
        conn.execute(
            "INSERT INTO favorites (game_id) VALUES (?1)",
            params![game_id],
        )?;
        Ok(true)
    }
}

pub fn get_favorites(db: &State<'_, DbState>) -> Result<Vec<String>> {
    let conn = db.conn()?;
    let mut stmt = conn.prepare("SELECT game_id FROM favorites")?;
    let rows = stmt.query_map([], |row| row.get::<_, String>(0))?;
    Ok(rows.collect::<Result<Vec<_>, _>>()?)
}

pub fn get_setting(db: &State<'_, DbState>, key: &str) -> Result<Option<String>> {
    let conn = db.conn()?;
    let result = conn.query_row(
        "SELECT value FROM settings WHERE key = ?1",
        params![key],
        |row| row.get::<_, String>(0),
    );
    match result {
        Ok(v) => Ok(Some(v)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e.into()),
    }
}

pub fn set_setting(db: &State<'_, DbState>, key: &str, value: &str) -> Result<()> {
    let conn = db.conn()?;
    conn.execute(
        "INSERT INTO settings (key, value) VALUES (?1, ?2)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        params![key, value],
    )?;
    Ok(())
}

// === Telemetry ===

pub fn log_telemetry(
    db: &State<'_, DbState>,
    event: &str,
    game_id: Option<&str>,
    version: Option<&str>,
    anonymous_id: &str,
    metadata: Option<&str>,
) -> Result<()> {
    let conn = db.conn()?;
    let now = chrono::Utc::now().timestamp();
    conn.execute(
        "INSERT INTO telemetry (event, game_id, version, anonymous_id, metadata, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![event, game_id, version, anonymous_id, metadata, now],
    )?;
    Ok(())
}

pub fn count_telemetry_events(db: &State<'_, DbState>, event: &str, since: i64) -> Result<u64> {
    let conn = db.conn()?;
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM telemetry WHERE event = ?1 AND created_at >= ?2",
        params![event, since],
        |row| row.get(0),
    )?;
    Ok(count as u64)
}

pub fn top_games_by_downloads(db: &State<'_, DbState>, limit: u32) -> Result<Vec<(String, u64)>> {
    let conn = db.conn()?;
    let mut stmt = conn.prepare(
        "SELECT game_id, COUNT(*) as cnt FROM telemetry
         WHERE event = 'install' AND game_id IS NOT NULL
         GROUP BY game_id ORDER BY cnt DESC LIMIT ?1"
    )?;
    let rows = stmt.query_map(params![limit], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)? as u64))
    })?;
    Ok(rows.collect::<Result<Vec<_>, _>>()?)
}

// === Bug reports ===

pub fn create_bug_report(
    db: &State<'_, DbState>,
    report: &crate::models::AdminBugReport,
) -> Result<()> {
    let conn = db.conn()?;
    conn.execute(
        "INSERT INTO bug_reports (id, game_id, version, user_id, title, description, status, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![
            report.id,
            report.game_id,
            report.version,
            report.user_id,
            report.title,
            report.description,
            report.status,
            report.created_at,
        ],
    )?;
    Ok(())
}

pub fn list_bug_reports(
    db: &State<'_, DbState>,
    status: Option<&str>,
) -> Result<Vec<crate::models::AdminBugReport>> {
    let conn = db.conn()?;
    let (sql, has_filter) = if status.is_some() {
        ("SELECT id, game_id, version, user_id, title, description, status, created_at
          FROM bug_reports WHERE status = ?1 ORDER BY created_at DESC", true)
    } else {
        ("SELECT id, game_id, version, user_id, title, description, status, created_at
          FROM bug_reports ORDER BY created_at DESC", false)
    };
    let mut stmt = conn.prepare(sql)?;
    let mapper = |row: &rusqlite::Row| -> rusqlite::Result<crate::models::AdminBugReport> {
        Ok(crate::models::AdminBugReport {
            id: row.get(0)?,
            game_id: row.get(1)?,
            version: row.get(2)?,
            user_id: row.get(3)?,
            title: row.get(4)?,
            description: row.get(5)?,
            status: row.get(6)?,
            created_at: row.get(7)?,
        })
    };
    if has_filter {
        let rows = stmt.query_map(params![status.unwrap()], mapper)?;
        Ok(rows.collect::<Result<Vec<_>, _>>()?)
    } else {
        let rows = stmt.query_map([], mapper)?;
        Ok(rows.collect::<Result<Vec<_>, _>>()?)
    }
}

// === Comments ===

pub fn create_comment(db: &State<'_, DbState>, comment: &crate::models::AdminComment) -> Result<()> {
    let conn = db.conn()?;
    conn.execute(
        "INSERT INTO comments (id, game_id, user_id, rating, content, status, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            comment.id,
            comment.game_id,
            comment.user_id,
            comment.rating,
            comment.content,
            comment.status,
            comment.created_at,
        ],
    )?;
    Ok(())
}

pub fn list_comments(
    db: &State<'_, DbState>,
    status: Option<&str>,
) -> Result<Vec<crate::models::AdminComment>> {
    let conn = db.conn()?;
    let mapper = |row: &rusqlite::Row| -> rusqlite::Result<crate::models::AdminComment> {
        Ok(crate::models::AdminComment {
            id: row.get(0)?,
            game_id: row.get(1)?,
            user_id: row.get(2)?,
            rating: row.get(3)?,
            content: row.get(4)?,
            status: row.get(5)?,
            created_at: row.get(6)?,
        })
    };

    if let Some(s) = status {
        let mut stmt = conn.prepare(
            "SELECT id, game_id, user_id, rating, content, status, created_at
             FROM comments WHERE status = ?1 ORDER BY created_at DESC"
        )?;
        let rows = stmt.query_map(params![s], mapper)?;
        Ok(rows.collect::<Result<Vec<_>, _>>()?)
    } else {
        let mut stmt = conn.prepare(
            "SELECT id, game_id, user_id, rating, content, status, created_at
             FROM comments ORDER BY created_at DESC"
        )?;
        let rows = stmt.query_map([], mapper)?;
        Ok(rows.collect::<Result<Vec<_>, _>>()?)
    }
}

pub fn update_comment_status(
    db: &State<'_, DbState>,
    id: &str,
    status: &str,
) -> Result<()> {
    let conn = db.conn()?;
    conn.execute(
        "UPDATE comments SET status = ?1 WHERE id = ?2",
        params![status, id],
    )?;
    Ok(())
}

// === User requests (game mong muốn việt hoá) ===

pub fn create_user_request(
    db: &State<'_, DbState>,
    request: &crate::models::AdminUserRequest,
) -> Result<()> {
    let conn = db.conn()?;
    // Upsert: nếu đã có game_name thì +1 vote
    conn.execute(
        "INSERT INTO user_requests (id, game_name, votes, requested_by, created_at)
         VALUES (?1, ?2, 1, ?3, ?4)
         ON CONFLICT(id) DO UPDATE SET votes = votes + 1",
        params![request.id, request.game_name, request.requested_by, request.created_at],
    )?;
    Ok(())
}

pub fn list_user_requests(db: &State<'_, DbState>) -> Result<Vec<crate::models::AdminUserRequest>> {
    let conn = db.conn()?;
    let mut stmt = conn.prepare(
        "SELECT id, game_name, votes, requested_by, created_at
         FROM user_requests ORDER BY votes DESC, created_at DESC"
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(crate::models::AdminUserRequest {
            id: row.get(0)?,
            game_name: row.get(1)?,
            votes: row.get(2)?,
            requested_by: row.get(3)?,
            created_at: row.get(4)?,
        })
    })?;
    Ok(rows.collect::<Result<Vec<_>, _>>()?)
}

// Bỏ warning unused import
#[allow(unused_imports)]
use tauri::Manager;