/*
    Local Native
    Copyright (C) 2018-2019  Yi Wang

    This program is free software: you can redistribute it and/or modify
    it under the terms of the GNU Affero General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    This program is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU Affero General Public License for more details.

    You should have received a copy of the GNU Affero General Public License
    along with this program.  If not, see <https://www.gnu.org/licenses/>.
*/

//! SQLite storage: where the database lives, how connections are opened, the
//! command dispatcher behind the JSON FFI, and last-write-wins tokens.

pub use crate::error::{DatabaseError, DbError, DbResult, ValidationError};
use models::Cmd;
pub use models::Note;
use rusqlite::{Connection, OptionalExtension};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::RwLock;

#[cfg(feature = "encryption")]
pub mod encryption;
pub mod migrations;
pub mod models;
pub mod peers;
pub mod queries;
pub mod sync;

#[cfg(test)]
mod tests;

/// Type alias for the r2d2 connection pool backed by SQLite.
pub type Pool = r2d2::Pool<r2d2_sqlite::SqliteConnectionManager>;

/// A single connection checked out from a [`Pool`]. Re-exported so front-ends
/// can name the pooled-connection type without depending on `r2d2` directly.
pub type PooledConn = r2d2::PooledConnection<r2d2_sqlite::SqliteConnectionManager>;

// ── Database location ─────────────────────────────────────────────────────

static DB_PATH_OVERRIDE: RwLock<Option<PathBuf>> = RwLock::new(None);

/// Open every later [`init_db`] / [`init_pool`] at `path` instead of the
/// platform default. Front-ends that know their sandbox — Android's
/// `filesDir`, an iOS App Group container, a CLI `--db` flag — call this
/// before anything else.
pub fn set_db_path(path: impl Into<PathBuf>) {
    let mut guard = DB_PATH_OVERRIDE
        .write()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    *guard = Some(path.into());
}

/// The database file the core opens: the path given to [`set_db_path`], else
/// `$LOCALNATIVE_DB`, else the platform default
/// (`~/LocalNative/localnative.sqlite3` on desktop).
pub fn db_path() -> DbResult<PathBuf> {
    let overridden = DB_PATH_OVERRIDE
        .read()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .clone();
    if let Some(path) = overridden {
        return Ok(path);
    }
    if let Some(path) = std::env::var_os("LOCALNATIVE_DB").filter(|p| !p.is_empty()) {
        return Ok(PathBuf::from(path));
    }
    default_db_path()
}

fn default_db_path() -> DbResult<PathBuf> {
    // Android apps pass their own `filesDir` through `set_db_path`; this is
    // only the historical location for callers that don't.
    if cfg!(target_os = "android") {
        return Ok(PathBuf::from(
            "/data/data/app.localnative/files/localnative.sqlite3",
        ));
    }
    let home = dirs::home_dir().ok_or_else(|| {
        DatabaseError::IoError(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Failed to get home directory",
        ))
    })?;
    let dir = if cfg!(target_os = "ios") {
        home.join("Documents")
    } else {
        home.join("LocalNative")
    };
    Ok(dir.join("localnative.sqlite3"))
}

fn create_parent_dir(path: &Path) -> DbResult<()> {
    if let Some(parent) = path.parent()
        && !parent.as_os_str().is_empty()
    {
        std::fs::create_dir_all(parent)?;
    }
    Ok(())
}

/// Applied to every connection. Temporary tables stay in memory, so SQLite
/// never needs a writable temp directory (Android has no `/tmp`).
const CONNECTION_PRAGMAS: &str =
    "PRAGMA journal_mode=WAL; PRAGMA busy_timeout=5000; PRAGMA temp_store=MEMORY;";

/// Open (or create) the database at [`db_path`] and run pending migrations.
pub fn init_db() -> DbResult<Connection> {
    init_db_at(&db_path()?)
}

/// Open (or create) the database at `path` and run pending migrations.
pub fn init_db_at(path: &Path) -> DbResult<Connection> {
    create_parent_dir(path)?;
    tracing::info!(db_path = %path.display(), "opening database");
    let conn = Connection::open(path)?;
    conn.execute_batch(CONNECTION_PRAGMAS)?;
    migrations::upgrade(&conn)?;
    Ok(conn)
}

/// Create a connection pool for the database at [`db_path`].
pub fn init_pool() -> DbResult<Pool> {
    init_pool_at(&db_path()?)
}

/// Create a connection pool for the database at `path`. Migrations run once on
/// a checked-out connection before the pool is returned.
pub fn init_pool_at(path: &Path) -> DbResult<Pool> {
    create_parent_dir(path)?;
    tracing::info!(db_path = %path.display(), "opening database pool");
    let manager = r2d2_sqlite::SqliteConnectionManager::file(path)
        .with_init(|c| c.execute_batch(CONNECTION_PRAGMAS));
    let pool = r2d2::Pool::builder()
        .max_size(4)
        .build(manager)
        .map_err(|e| DatabaseError::IoError(std::io::Error::other(e.to_string())))?;
    let conn = pool
        .get()
        .map_err(|e| DatabaseError::IoError(std::io::Error::other(e.to_string())))?;
    migrations::upgrade(&conn)?;
    Ok(pool)
}

/// Dispatch a [`Cmd`] against an open database connection and return the result serialized as JSON.
pub fn process_cmd(cmd: Cmd, conn: &Connection) -> DbResult<String> {
    match cmd {
        Cmd::Insert(ref insert) => {
            insert.process(conn)?;
            let select_result = queries::do_select(conn, insert.limit, insert.offset)?;
            Ok(serde_json::to_string(&select_result)?)
        }
        Cmd::InsertImage(ref insert) => {
            insert.process_image(conn)?;
            let select_result = queries::do_select(conn, insert.limit, insert.offset)?;
            Ok(serde_json::to_string(&select_result)?)
        }
        Cmd::Delete(ref delete) => {
            delete.process(conn)?;
            let search_result =
                queries::do_search(conn, &delete.query, delete.limit, delete.offset)?;
            Ok(serde_json::to_string(&search_result)?)
        }
        Cmd::Update(ref update) => {
            update.process(conn)?;
            let search_result =
                queries::do_search(conn, &update.query, update.limit, update.offset)?;
            Ok(serde_json::to_string(&search_result)?)
        }
        Cmd::Select(ref select) => {
            let select_result = select.process(conn)?;
            Ok(serde_json::to_string(&select_result)?)
        }
        Cmd::Search(ref search) => {
            let search_result = search.process(conn)?;
            Ok(serde_json::to_string(&search_result)?)
        }
        Cmd::Filter(ref filter) => {
            let filter_result = filter.process(conn)?;
            Ok(serde_json::to_string(&filter_result)?)
        }
        Cmd::Upgrade => {
            migrations::upgrade(conn)?;
            Ok(serde_json::to_string(&"Upgrade completed")?)
        }
        Cmd::SyncViaAttach(ref sync) => {
            sync.process(conn)?;
            Ok(serde_json::to_string(&"Sync via attach completed")?)
        }
        Cmd::ExportDb(ref export) => {
            export.process(conn)?;
            Ok(serde_json::to_string(
                &serde_json::json!({ "export-db": export.dest }),
            )?)
        }
        Cmd::ImportDb(ref import) => {
            let merged = import.process(conn)?;
            Ok(serde_json::to_string(
                &serde_json::json!({ "import-db": { "src": import.src, "merged": merged } }),
            )?)
        }
    }
}

// ── Last-write-wins tokens ────────────────────────────────────────────────
//
// Every version of a note carries an `updated_at` token: 20 zero-padded digits
// of milliseconds, a dash, and the writing database's `node_id` (the
// tiebreaker). Tokens compare as strings.
//
// Tokens come from a hybrid logical clock. The `hlc` row in `meta` holds the
// highest millisecond value this database has issued or seen — from its own
// writes, from notes received from peers, from merged files. A new local token
// is `max(now, hlc + 1, superseded + 1)`, so a local write always outranks the
// version it replaces and everything this device has observed, whatever the
// wall clocks say. The row lives in the database, so every process writing to
// it (GUI, CLI, browser host) shares one monotonic clock.

/// How far into the future a token received from a peer may claim to be.
pub const MAX_CLOCK_SKEW_MS: u64 = 7 * 24 * 60 * 60 * 1000;

/// Longest node part accepted in a token (node ids are UUIDs; rows back-filled
/// by migrations use `legacy`).
const TOKEN_NODE_MAX_LEN: usize = 64;

/// Columns emptied when a note becomes a tombstone: a deleted note keeps only
/// its identity, its token and `created_at`.
pub(crate) const TOMBSTONE_CONTENT: &str = "title = '', url = '', tags = '', description = '', \
     comments = '', annotations = '', metadata = '{}'";

/// This database's stable node identifier, created on first use.
pub(crate) fn node_id(conn: &Connection) -> DbResult<String> {
    if let Some(id) = meta_get(conn, "node_id")? {
        return Ok(id);
    }
    let id = uuid::Uuid::new_v4().to_string();
    conn.execute(
        "INSERT OR IGNORE INTO meta (meta_key, meta_value) VALUES ('node_id', ?1)",
        rusqlite::params![id],
    )?;
    // Re-read in case a concurrent writer won the INSERT OR IGNORE race.
    Ok(meta_get(conn, "node_id")?.unwrap_or(id))
}

pub(crate) fn meta_get(conn: &Connection, key: &str) -> DbResult<Option<String>> {
    Ok(conn
        .query_row(
            "SELECT meta_value FROM meta WHERE meta_key = ?1",
            rusqlite::params![key],
            |row| row.get(0),
        )
        .optional()?)
}

pub(crate) fn meta_set(conn: &Connection, key: &str, value: &str) -> DbResult<()> {
    conn.execute(
        "INSERT INTO meta (meta_key, meta_value) VALUES (?1, ?2)
         ON CONFLICT(meta_key) DO UPDATE SET meta_value = excluded.meta_value",
        rusqlite::params![key, value],
    )?;
    Ok(())
}

pub(crate) fn now_millis() -> u64 {
    u64::try_from(chrono::Utc::now().timestamp_millis()).unwrap_or(0)
}

/// The millisecond component of a token, if it is well-formed.
pub fn token_millis(token: &str) -> Option<u64> {
    let (millis, _) = token.split_once('-')?;
    if millis.len() != 20 || !millis.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    millis.parse().ok()
}

/// Whether a token received from a peer is well-formed and not implausibly far
/// in the future. A token that claims the far future would outrank every
/// later edit of that note forever.
pub fn is_valid_token(token: &str, now_ms: u64) -> bool {
    let Some((_, node)) = token.split_once('-') else {
        return false;
    };
    let node_ok = !node.is_empty()
        && node.len() <= TOKEN_NODE_MAX_LEN
        && node.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-');
    node_ok && token_millis(token).is_some_and(|ms| ms <= now_ms.saturating_add(MAX_CLOCK_SKEW_MS))
}

fn millis_as_sql(ms: u64) -> i64 {
    i64::try_from(ms).unwrap_or(i64::MAX)
}

/// Advance the hybrid logical clock to at least `observed_ms`.
pub(crate) fn observe_millis(conn: &Connection, observed_ms: u64) -> DbResult<()> {
    conn.execute(
        "INSERT INTO meta (meta_key, meta_value) VALUES ('hlc', ?1)
         ON CONFLICT(meta_key) DO UPDATE SET meta_value =
             CAST(max(CAST(meta_value AS INTEGER), CAST(excluded.meta_value AS INTEGER)) AS TEXT)",
        rusqlite::params![millis_as_sql(observed_ms).to_string()],
    )?;
    Ok(())
}

/// Advance the clock past every token already stored — used after bulk merges
/// that bypass [`sync::apply_notes`].
pub(crate) fn observe_stored_tokens(conn: &Connection) -> DbResult<()> {
    let max: Option<String> = conn.query_row(
        "SELECT max(updated_at) FROM note WHERE updated_at GLOB '[0-9]*-*'",
        [],
        |row| row.get(0),
    )?;
    if let Some(ms) = max.as_deref().and_then(token_millis) {
        observe_millis(conn, ms)?;
    }
    Ok(())
}

/// A token for a local write that supersedes `previous` (the row's current
/// token, when updating an existing note).
pub(crate) fn next_update_token(conn: &Connection, previous: Option<&str>) -> DbResult<String> {
    let floor = previous
        .and_then(token_millis)
        .map_or(0, |ms| ms.saturating_add(1));
    let candidate = now_millis().max(floor);
    let stamp: i64 = conn.query_row(
        "INSERT INTO meta (meta_key, meta_value) VALUES ('hlc', ?1)
         ON CONFLICT(meta_key) DO UPDATE SET meta_value = CAST(
             max(CAST(meta_value AS INTEGER) + 1, CAST(excluded.meta_value AS INTEGER)) AS TEXT)
         RETURNING CAST(meta_value AS INTEGER)",
        rusqlite::params![millis_as_sql(candidate).to_string()],
        |row| row.get(0),
    )?;
    Ok(format!("{:020}-{}", stamp.max(0), node_id(conn)?))
}

// ── Tags ──────────────────────────────────────────────────────────────────

/// The canonical form of a tag list: comma-separated, trimmed, empty entries
/// dropped, duplicates removed case-insensitively (the first spelling wins).
pub fn normalize_tags(tags: &str) -> String {
    let mut seen = HashSet::new();
    tags.split(',')
        .map(str::trim)
        .filter(|tag| !tag.is_empty() && seen.insert(tag.to_lowercase()))
        .collect::<Vec<_>>()
        .join(",")
}

/// Rewrite every stored tag list into canonical form. Deterministic, so peers
/// that run it converge without new tokens.
pub(crate) fn normalize_stored_tags(conn: &Connection) -> DbResult<usize> {
    let rows: Vec<(i64, String)> = {
        let mut stmt = conn.prepare("SELECT rowid, tags FROM note WHERE tags <> ''")?;
        stmt.query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?
            .collect::<Result<_, _>>()?
    };
    let mut changed = 0;
    for (rowid, tags) in rows {
        let normalized = normalize_tags(&tags);
        if normalized != tags {
            conn.execute(
                "UPDATE note SET tags = ?1 WHERE rowid = ?2",
                rusqlite::params![normalized, rowid],
            )?;
            changed += 1;
        }
    }
    Ok(changed)
}
