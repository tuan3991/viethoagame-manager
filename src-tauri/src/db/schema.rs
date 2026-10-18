//! Schema SQL cho database

pub const SCHEMA: &str = r#"
CREATE TABLE IF NOT EXISTS games (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    version TEXT NOT NULL,
    installed_version TEXT,
    install_path TEXT,
    installed_at INTEGER,
    last_checked INTEGER NOT NULL,
    status TEXT NOT NULL DEFAULT 'not_installed'
);

CREATE TABLE IF NOT EXISTS install_history (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    game_id TEXT NOT NULL,
    version TEXT NOT NULL,
    action TEXT NOT NULL,
    success INTEGER NOT NULL,
    error_message TEXT,
    created_at INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS backups (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    game_id TEXT NOT NULL,
    version TEXT NOT NULL,
    file_path TEXT NOT NULL,
    original_path TEXT NOT NULL,
    file_hash TEXT NOT NULL,
    created_at INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS favorites (
    game_id TEXT PRIMARY KEY
);

CREATE TABLE IF NOT EXISTS settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);

-- Telemetry: ghi lại mỗi install/uninstall/update để admin xem
CREATE TABLE IF NOT EXISTS telemetry (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    event TEXT NOT NULL,                -- 'download', 'install', 'uninstall', 'update', 'open'
    game_id TEXT,
    version TEXT,
    anonymous_id TEXT NOT NULL,
    metadata TEXT,                       -- JSON
    created_at INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS bug_reports (
    id TEXT PRIMARY KEY,
    game_id TEXT NOT NULL,
    version TEXT NOT NULL,
    user_id TEXT NOT NULL,
    title TEXT NOT NULL,
    description TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'open', -- 'open'|'investigating'|'resolved'|'closed'
    created_at INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS comments (
    id TEXT PRIMARY KEY,
    game_id TEXT NOT NULL,
    user_id TEXT NOT NULL,
    rating INTEGER NOT NULL,
    content TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'pending', -- 'pending'|'approved'|'rejected'
    created_at INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS user_requests (
    id TEXT PRIMARY KEY,
    game_name TEXT NOT NULL,
    votes INTEGER NOT NULL DEFAULT 1,
    requested_by TEXT NOT NULL,
    created_at INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_games_status ON games(status);
CREATE INDEX IF NOT EXISTS idx_backups_game ON backups(game_id);
CREATE INDEX IF NOT EXISTS idx_history_game ON install_history(game_id);
CREATE INDEX IF NOT EXISTS idx_telemetry_event ON telemetry(event, created_at);
CREATE INDEX IF NOT EXISTS idx_reports_status ON bug_reports(status);
CREATE INDEX IF NOT EXISTS idx_comments_status ON comments(status);
CREATE INDEX IF NOT EXISTS idx_requests_votes ON user_requests(votes DESC);
"#;