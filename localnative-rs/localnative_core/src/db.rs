// db.rs
pub use crate::error::{DatabaseError, DbError, DbResult, ValidationError};
use models::Cmd;
pub use models::{Note, SearchResult};
use rusqlite::Connection;

/// Type alias for the r2d2 connection pool backed by SQLite.
pub type Pool = r2d2::Pool<r2d2_sqlite::SqliteConnectionManager>;

/// A single connection checked out from a [`Pool`]. Re-exported so front-ends
/// can name the pooled-connection type without depending on `r2d2` directly.
pub type PooledConn = r2d2::PooledConnection<r2d2_sqlite::SqliteConnectionManager>;

// ── Last-write-wins sync helpers ───────────────────────────────────────────
//
// Sync uses a per-row logical timestamp (`updated_at`) to decide which version
// of a note wins (last-write-wins). The token is a fixed-width, lexicographically
// sortable string: zero-padded epoch-millis (kept monotonic within this process)
// followed by the database's stable `node_id` as a deterministic tiebreaker.
//
// NOTE: this is a monotonic *physical* clock, which is adequate when peer clocks
// are roughly in sync. A full Hybrid Logical Clock — which also advances the
// local clock past timestamps observed from peers, hardening against clock skew
// and tombstone resurrection — is the planned follow-up (see TODO.md).

/// Return this database's stable node identifier, creating one if absent.
fn node_id(conn: &Connection) -> DbResult<String> {
    if let Ok(id) = conn.query_row(
        "SELECT meta_value FROM meta WHERE meta_key = 'node_id'",
        [],
        |row| row.get::<_, String>(0),
    ) {
        return Ok(id);
    }
    let id = uuid::Uuid::new_v4().to_string();
    conn.execute(
        "INSERT OR IGNORE INTO meta (meta_key, meta_value) VALUES ('node_id', ?1)",
        rusqlite::params![id],
    )?;
    // Re-read in case a concurrent writer won the INSERT OR IGNORE race.
    let id = conn.query_row(
        "SELECT meta_value FROM meta WHERE meta_key = 'node_id'",
        [],
        |row| row.get(0),
    )?;
    Ok(id)
}

/// Generate a sortable last-write-wins token for a local write to a note.
fn next_update_token(conn: &Connection) -> DbResult<String> {
    use std::sync::atomic::{AtomicU64, Ordering};
    static LAST_MILLIS: AtomicU64 = AtomicU64::new(0);

    let phys = chrono::Utc::now().timestamp_millis().max(0) as u64;
    let mut stamp = phys;
    LAST_MILLIS
        .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |last| {
            stamp = phys.max(last.saturating_add(1));
            Some(stamp)
        })
        .ok();
    Ok(format!("{:020}-{}", stamp, node_id(conn)?))
}

/// Whether `table` has a column named `column` (used to keep migrations
/// idempotent against schemas already created by `init_db_schema`).
fn column_exists(conn: &Connection, table: &str, column: &str) -> DbResult<bool> {
    let mut stmt = conn.prepare(&format!("PRAGMA table_info({table})"))?;
    let mut rows = stmt.query([])?;
    while let Some(row) = rows.next()? {
        let name: String = row.get(1)?;
        if name == column {
            return Ok(true);
        }
    }
    Ok(false)
}

/// Open (or create) the SQLite database at the platform-appropriate location and run any
/// pending schema migrations.  Enables WAL mode and sets a busy timeout for
/// better concurrent-read performance.
pub fn init_db() -> DbResult<Connection> {
    let db_path = utils::sqlite3_db_location()?;
    tracing::info!(db_path, "opening database");
    let conn = Connection::open(&db_path)?;

    conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA busy_timeout=5000;")?;

    #[cfg(target_os = "android")]
    conn.execute_batch("PRAGMA temp_store_directory = '/data/data/app.localnative/cache'")?;

    migrations::upgrade(&conn)?;

    Ok(conn)
}

/// Create an r2d2 connection pool for the SQLite database.  Every connection
/// obtained from the pool automatically enables WAL mode and a 5-second busy
/// timeout.  Schema migrations are run once on a temporary connection before
/// the pool is returned.
pub fn init_pool() -> DbResult<Pool> {
    let db_path = utils::sqlite3_db_location()?;
    tracing::info!(db_path, "opening database pool");

    let manager = r2d2_sqlite::SqliteConnectionManager::file(&db_path).with_init(|c| {
        c.execute_batch("PRAGMA journal_mode=WAL; PRAGMA busy_timeout=5000;")?;

        #[cfg(target_os = "android")]
        c.execute_batch("PRAGMA temp_store_directory = '/data/data/app.localnative/cache'")?;

        Ok(())
    });

    let pool = r2d2::Pool::builder()
        .max_size(4)
        .build(manager)
        .map_err(|e| DatabaseError::IoError(std::io::Error::other(e.to_string())))?;

    // Run migrations once on a fresh connection.
    {
        let conn = pool
            .get()
            .map_err(|e| DatabaseError::IoError(std::io::Error::other(e.to_string())))?;
        migrations::upgrade(&conn)?;
    }

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
    }
}

pub mod models {
    use chrono::NaiveDate;
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize, Debug, Clone, Default)]
    pub struct Day {
        pub date: NaiveDate,
        pub count: i64,
    }

    #[derive(Debug, Default, Deserialize, Serialize, Clone)]
    pub struct Tags {
        pub tag: String,
        pub count: i64,
    }

    #[derive(Serialize, Deserialize, Debug, Default, Clone)]
    pub struct Note {
        pub rowid: i64,
        pub uuid4: String,
        pub title: String,
        pub url: String,
        pub tags: String,
        pub description: String,
        pub comments: String,
        pub annotations: String,
        pub created_at: String,
        pub is_public: bool,
        pub metadata: String,
        /// Last-write-wins logical timestamp (sortable token). Used by sync to
        /// decide which version of a note wins. Empty for legacy rows until the
        /// 0.10.0 migration backfills it. See [`crate::db::sync`].
        #[serde(default)]
        pub updated_at: String,
        /// Soft-delete tombstone flag. `true` means the note is deleted; the row
        /// is retained so the deletion can propagate to peers during sync.
        #[serde(default)]
        pub deleted: bool,
    }

    /// A search result wrapping a [`Note`] with optional FTS5 snippet highlights.
    #[derive(Serialize, Deserialize, Debug, Default, Clone)]
    pub struct SearchResult {
        pub note: Note,
        pub title_snippet: Option<String>,
        pub description_snippet: Option<String>,
    }

    #[derive(Serialize, Deserialize, Debug)]
    #[serde(tag = "action", rename_all = "kebab-case")]
    pub enum Cmd {
        Insert(CmdInsert),
        InsertImage(CmdInsert),
        Delete(CmdDelete),
        Select(CmdSelect),
        Search(CmdSearch),
        Filter(CmdFilter),
        Upgrade,
        SyncViaAttach(CmdSyncViaAttach),
        ExportDb(CmdExportDb),
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub struct CmdInsert {
        pub title: String,
        pub url: String,
        pub tags: String,
        pub description: String,
        pub comments: String,
        pub annotations: String,
        pub limit: u32,
        pub offset: u32,
        pub is_public: bool,
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub struct CmdFilter {
        pub query: String,
        pub limit: u32,
        pub offset: u32,
        pub from: String,
        pub to: String,
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub struct CmdSearch {
        pub query: String,
        pub limit: u32,
        pub offset: u32,
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub struct CmdDelete {
        pub query: String,
        pub rowid: i64,
        pub limit: u32,
        pub offset: u32,
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub struct CmdSelect {
        pub limit: u32,
        pub offset: u32,
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub struct CmdSyncViaAttach {
        pub uri: String,
    }

    /// Export a clean, single-file copy of the database to `dest`.
    #[derive(Serialize, Deserialize, Debug)]
    pub struct CmdExportDb {
        pub dest: String,
    }

    #[derive(Debug, Default, Deserialize, Serialize, Clone)]
    pub struct QueryResult {
        pub count: u32,
        pub notes: Vec<Note>,
        pub days: Vec<Day>,
        pub tags: Vec<Tags>,
    }
}

// Error types are now defined in crate::error and re-exported above.

mod utils {
    use super::*;

    pub fn sqlite3_db_location() -> DbResult<String> {
        let dir_name = if cfg!(target_os = "android") {
            "/data/data/app.localnative/files"
        } else if cfg!(target_os = "ios") {
            "Documents"
        } else {
            "LocalNative"
        };

        let home_dir = dirs::home_dir().ok_or(DatabaseError::IoError(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Failed to get home directory",
        )))?;
        let dir = format!(
            "{}/{}",
            home_dir
                .to_str()
                .ok_or(DatabaseError::IoError(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "Invalid home directory path",
                )))?,
            dir_name
        );
        tracing::debug!(dir, "database directory location");
        std::fs::create_dir_all(&dir)?;
        Ok(format!("{}/localnative.sqlite3", dir))
    }
}

mod commands {
    use super::*;
    use base64::Engine as _;
    use base64::engine::general_purpose::STANDARD;
    use models::{
        CmdDelete, CmdExportDb, CmdFilter, CmdInsert, CmdSearch, CmdSelect, CmdSyncViaAttach, Note,
        QueryResult,
    };
    use rusqlite::Connection;

    impl CmdFilter {
        pub fn process(&self, conn: &Connection) -> DbResult<QueryResult> {
            queries::do_filter(
                conn,
                &self.query,
                self.limit,
                self.offset,
                &self.from,
                &self.to,
            )
        }
    }

    impl CmdInsert {
        pub fn process(&self, conn: &Connection) -> DbResult<Note> {
            queries::insert_note(
                conn,
                &self.title,
                &self.url,
                &self.tags,
                &self.description,
                &self.comments,
                self.annotations.as_bytes(),
                self.is_public,
            )
        }

        pub fn process_image(&self, conn: &Connection) -> DbResult<Note> {
            let data64 = self.annotations.replace("data:image/png;base64,", "");
            let decoded = STANDARD.decode(&data64)?;
            queries::insert_note(
                conn,
                &self.title,
                &self.url,
                &self.tags,
                &self.description,
                &self.comments,
                &decoded,
                self.is_public,
            )
        }
    }

    impl CmdDelete {
        pub fn process(&self, conn: &Connection) -> DbResult<()> {
            queries::delete_note(conn, self.rowid)
        }
    }

    impl CmdSyncViaAttach {
        pub fn process(&self, conn: &Connection) -> DbResult<()> {
            queries::sync_via_attach(conn, &self.uri)
        }
    }

    impl CmdExportDb {
        pub fn process(&self, conn: &Connection) -> DbResult<()> {
            queries::export_db(conn, &self.dest)
        }
    }

    impl CmdSelect {
        pub fn process(&self, conn: &Connection) -> DbResult<QueryResult> {
            queries::do_select(conn, self.limit, self.offset)
        }
    }

    impl CmdSearch {
        pub fn process(&self, conn: &Connection) -> DbResult<QueryResult> {
            queries::do_search(conn, &self.query, self.limit, self.offset)
        }
    }
}

pub mod queries {
    use super::*;
    use chrono::NaiveDate;
    use models::{Day, Note, QueryResult, Tags};
    use regex::Regex;
    use rusqlite::Connection;
    use std::collections::HashMap;
    use std::path::Path;
    use std::sync::OnceLock;
    use uuid::Uuid;

    fn map_note(row: &rusqlite::Row) -> rusqlite::Result<Note> {
        Ok(Note {
            rowid: row.get("rowid")?,
            uuid4: row.get("uuid4")?,
            title: row.get("title")?,
            url: row.get("url")?,
            tags: row.get("tags")?,
            description: row.get("description")?,
            comments: row.get("comments")?,
            annotations: row.get("annotations")?,
            created_at: row.get("created_at")?,
            is_public: row.get("is_public")?,
            metadata: row.get("metadata")?,
            // Tolerant of SELECTs that do not project these columns (UI read
            // paths don't need them); the sync paths select them explicitly.
            updated_at: row.get("updated_at").unwrap_or_default(),
            deleted: row.get("deleted").unwrap_or(false),
        })
    }

    fn map_day(row: &rusqlite::Row) -> rusqlite::Result<Day> {
        let date_str: String = row.get("date")?;
        Ok(Day {
            date: NaiveDate::parse_from_str(&date_str, "%Y-%m-%d").unwrap_or_else(|e| {
                tracing::warn!(date_str, %e, "failed to parse date from database");
                NaiveDate::default()
            }),
            count: row.get("count")?,
        })
    }

    #[allow(clippy::too_many_arguments)] // note fields are all distinct; grouping into a struct would require a separate type
    pub fn insert_note(
        conn: &Connection,
        title: &str,
        url: &str,
        tags: &str,
        description: &str,
        comments: &str,
        annotations: &[u8],
        is_public: bool,
    ) -> DbResult<Note> {
        let uuid4 = Uuid::new_v4().to_string();
        let created_at = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
        let updated_at = next_update_token(conn)?;

        conn.execute(
            "INSERT INTO note (uuid4, title, url, tags, description, comments, annotations, created_at, is_public, metadata, updated_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, '{}', ?10)",
            rusqlite::params![uuid4, title, url, tags, description, comments, annotations, created_at, is_public, updated_at],
        )?;

        let note = conn.query_row(
            "SELECT rowid, uuid4, title, url, tags, description, comments, hex(annotations) as annotations, created_at, is_public, metadata FROM note WHERE uuid4 = ?1",
            rusqlite::params![uuid4],
            map_note,
        )?;

        Ok(note)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn insert_note_with_timestamp(
        conn: &Connection,
        title: &str,
        url: &str,
        tags: &str,
        description: &str,
        comments: &str,
        annotations: &[u8],
        is_public: bool,
        created_at: &str,
    ) -> DbResult<Note> {
        let uuid4 = Uuid::new_v4().to_string();
        let updated_at = next_update_token(conn)?;

        conn.execute(
            "INSERT INTO note (uuid4, title, url, tags, description, comments, annotations, created_at, is_public, metadata, updated_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, '{}', ?10)",
            rusqlite::params![uuid4, title, url, tags, description, comments, annotations, created_at, is_public, updated_at],
        )?;

        let note = conn.query_row(
            "SELECT rowid, uuid4, title, url, tags, description, comments, hex(annotations) as annotations, created_at, is_public, metadata FROM note WHERE uuid4 = ?1",
            rusqlite::params![uuid4],
            map_note,
        )?;

        Ok(note)
    }

    /// Soft-delete a note: mark it as a tombstone and bump its last-write-wins
    /// timestamp so the deletion wins over the original insert and propagates to
    /// peers during sync. The row is retained (not physically removed) so the
    /// tombstone can be replicated; read queries filter out `deleted = 1` rows.
    pub fn delete_note(conn: &Connection, rowid: i64) -> DbResult<()> {
        let updated_at = next_update_token(conn)?;
        conn.execute(
            "UPDATE note SET deleted = 1, updated_at = ?2 WHERE rowid = ?1",
            rusqlite::params![rowid, updated_at],
        )?;
        Ok(())
    }

    fn validate_sync_file_path(uri: &str) -> DbResult<()> {
        let path = Path::new(uri);

        // Ensure the path is absolute to prevent relative path traversal
        if !path.is_absolute() {
            return Err(ValidationError::InvalidPath(
                "Sync file path must be absolute".to_string(),
            )
            .into());
        }

        // Verify the file exists and is a regular file
        if !path.is_file() {
            return Err(ValidationError::InvalidPath(
                "Sync file does not exist or is not a regular file".to_string(),
            )
            .into());
        }

        // Validate file extension
        match path.extension().and_then(|e| e.to_str()) {
            Some("sqlite3") | Some("db") | Some("sqlite") => {}
            _ => {
                return Err(ValidationError::InvalidPath(
                    "Sync file must have a .sqlite3, .sqlite, or .db extension".to_string(),
                )
                .into());
            }
        }

        Ok(())
    }

    /// Whether `schema.note` (e.g. `main` or an attached `other`) has `column`.
    /// `schema` is a fixed internal identifier, never user input.
    fn schema_column_exists(conn: &Connection, schema: &str, column: &str) -> DbResult<bool> {
        let mut stmt = conn.prepare(&format!("PRAGMA {schema}.table_info(note)"))?;
        let mut rows = stmt.query([])?;
        while let Some(row) = rows.next()? {
            let name: String = row.get(1)?;
            if name == column {
                return Ok(true);
            }
        }
        Ok(false)
    }

    /// Bring `schema.note` up to the last-write-wins schema, adding and
    /// backfilling the columns when an attached peer database predates 0.10.0.
    fn ensure_lww_columns(conn: &Connection, schema: &str) -> DbResult<()> {
        if !schema_column_exists(conn, schema, "updated_at")? {
            conn.execute_batch(&format!(
                "ALTER TABLE {schema}.note ADD COLUMN updated_at TEXT NOT NULL DEFAULT '';"
            ))?;
        }
        if !schema_column_exists(conn, schema, "deleted")? {
            conn.execute_batch(&format!(
                "ALTER TABLE {schema}.note ADD COLUMN deleted INTEGER NOT NULL DEFAULT 0;"
            ))?;
        }
        conn.execute_batch(&format!(
            "UPDATE {schema}.note
             SET updated_at = printf('%020d-legacy', CAST(strftime('%s', created_at) AS INTEGER) * 1000)
             WHERE updated_at IS NULL OR updated_at = '';"
        ))?;
        Ok(())
    }

    /// Offline sync against an attached SQLite file. Bidirectional, conflict-
    /// resolving via last-write-wins on `updated_at`, and tombstone-aware so
    /// deletions replicate. Mirrors the live RPC merge so the two paths agree.
    pub fn sync_via_attach(conn: &Connection, uri: &str) -> DbResult<()> {
        validate_sync_file_path(uri)?;
        conn.execute("ATTACH ?1 AS other", rusqlite::params![uri])?;
        let result = sync_attached(conn);
        // Always detach, even on failure, so a later sync can re-attach.
        let _ = conn.execute_batch("DETACH DATABASE other;");
        result
    }

    /// Export a clean, single-file copy of the database to `dest` using
    /// `VACUUM INTO`. The copy is compacted and contains all committed data —
    /// including transactions still resident in the WAL — with no `-wal`/`-shm`
    /// sidecars, so it is safe to read or share as a standalone file. Any
    /// existing file at `dest` is replaced (VACUUM INTO refuses to overwrite).
    ///
    /// Note: with the optional SQLCipher `encryption` feature, the exported
    /// copy is written **unencrypted** (VACUUM INTO does not carry the key);
    /// the default build stores the database in plaintext, so the copy matches
    /// the source.
    pub fn export_db(conn: &Connection, dest: &str) -> DbResult<()> {
        if dest.trim().is_empty() {
            return Err(ValidationError::InvalidPath(
                "Export destination path is empty".to_string(),
            )
            .into());
        }
        let dest_path = Path::new(dest);
        // VACUUM INTO errors if the target exists; make re-export idempotent.
        if dest_path.exists() {
            std::fs::remove_file(dest_path)?;
        }
        if let Some(parent) = dest_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        conn.execute("VACUUM INTO ?1", rusqlite::params![dest])?;
        Ok(())
    }

    fn sync_attached(conn: &Connection) -> DbResult<()> {
        ensure_lww_columns(conn, "main")?;
        ensure_lww_columns(conn, "other")?;
        // `WHERE true` is required for SQLite to parse ON CONFLICT after a SELECT.
        conn.execute_batch(
            "INSERT INTO main.note AS m
                 (uuid4, title, url, tags, description, comments, annotations, created_at, is_public, metadata, updated_at, deleted)
             SELECT uuid4, title, url, tags, description, comments, annotations, created_at, is_public, metadata, updated_at, deleted
             FROM other.note WHERE true
             ON CONFLICT(uuid4) DO UPDATE SET
                 title = excluded.title, url = excluded.url, tags = excluded.tags,
                 description = excluded.description, comments = excluded.comments,
                 annotations = excluded.annotations, created_at = excluded.created_at,
                 is_public = excluded.is_public, metadata = excluded.metadata,
                 updated_at = excluded.updated_at, deleted = excluded.deleted
             WHERE excluded.updated_at > m.updated_at;

             INSERT INTO other.note AS o
                 (uuid4, title, url, tags, description, comments, annotations, created_at, is_public, metadata, updated_at, deleted)
             SELECT uuid4, title, url, tags, description, comments, annotations, created_at, is_public, metadata, updated_at, deleted
             FROM main.note WHERE true
             ON CONFLICT(uuid4) DO UPDATE SET
                 title = excluded.title, url = excluded.url, tags = excluded.tags,
                 description = excluded.description, comments = excluded.comments,
                 annotations = excluded.annotations, created_at = excluded.created_at,
                 is_public = excluded.is_public, metadata = excluded.metadata,
                 updated_at = excluded.updated_at, deleted = excluded.deleted
             WHERE excluded.updated_at > o.updated_at;",
        )?;
        Ok(())
    }

    pub fn do_select(conn: &Connection, limit: u32, offset: u32) -> DbResult<QueryResult> {
        let count = select_count(conn)?;
        let notes = select(conn, limit, offset)?;
        let days = select_by_day(conn)?;
        let tags = select_by_tag(conn)?;

        Ok(QueryResult {
            count,
            notes,
            days,
            tags,
        })
    }

    pub fn do_search(
        conn: &Connection,
        query: &str,
        limit: u32,
        offset: u32,
    ) -> DbResult<QueryResult> {
        let count = search_count(conn, query)?;
        let notes = search(conn, query, limit, offset)?;
        let days = search_by_day(conn, query)?;
        let tags = search_by_tag(conn, query)?;

        Ok(QueryResult {
            count,
            notes,
            days,
            tags,
        })
    }

    pub fn do_filter(
        conn: &Connection,
        query: &str,
        limit: u32,
        offset: u32,
        from: &str,
        to: &str,
    ) -> DbResult<QueryResult> {
        let count = filter_count(conn, query, from, to)?;
        let notes = filter(conn, query, from, to, limit, offset)?;
        let days = search_by_day(conn, query)?;
        let tags = filter_by_tag(conn, query, from, to)?;

        Ok(QueryResult {
            count,
            notes,
            days,
            tags,
        })
    }

    fn select_count(conn: &Connection) -> DbResult<u32> {
        let count: i64 =
            conn.query_row("SELECT COUNT(1) FROM note WHERE deleted = 0", [], |row| {
                row.get(0)
            })?;
        Ok(u32::try_from(count).unwrap_or(u32::MAX))
    }

    fn select(conn: &Connection, limit: u32, offset: u32) -> DbResult<Vec<Note>> {
        let mut stmt = conn.prepare(
            "SELECT rowid, uuid4, title, url, tags, description, comments,
             hex(annotations) as annotations, created_at, is_public, metadata
             FROM note
             WHERE deleted = 0
             ORDER BY created_at DESC
             LIMIT ?1 OFFSET ?2",
        )?;
        let notes = stmt
            .query_map(rusqlite::params![limit, offset], map_note)?
            .collect::<Result<Vec<_>, rusqlite::Error>>()?;
        Ok(notes)
    }

    fn select_by_day(conn: &Connection) -> DbResult<Vec<Day>> {
        let mut stmt = conn.prepare(
            "SELECT DATE(substr(created_at, 1, 10)) as date, COUNT(1) as count
            FROM note
            WHERE deleted = 0
            GROUP BY date
            ORDER BY date",
        )?;
        let days = stmt
            .query_map([], map_day)?
            .collect::<Result<Vec<_>, rusqlite::Error>>()?;
        Ok(days)
    }

    fn select_by_tag(conn: &Connection) -> DbResult<Vec<Tags>> {
        let mut tag_count_map = HashMap::new();
        let mut stmt = conn.prepare("SELECT tags FROM note WHERE deleted = 0")?;
        let tags_iter = stmt.query_map([], |row| row.get::<_, String>(0))?;

        for tag_result in tags_iter {
            let tags_str = tag_result?;
            for tag in tags_str.split(',').map(|s| s.to_lowercase()) {
                *tag_count_map.entry(tag).or_insert(0i64) += 1;
            }
        }

        let tags = tag_count_map
            .into_iter()
            .map(|(tag, count)| Tags { tag, count })
            .collect();
        Ok(tags)
    }

    fn search_count(conn: &Connection, query: &str) -> DbResult<u32> {
        if query.is_empty() {
            return select_count(conn);
        }

        let fts_query = make_fts_query(query);
        let sql = format!(
            "SELECT COUNT(1)
            FROM note
            WHERE deleted = 0 AND {}",
            fts_where_clause(1)
        );

        let count: i64 = conn.query_row(&sql, rusqlite::params![fts_query], |row| row.get(0))?;
        Ok(u32::try_from(count).unwrap_or(u32::MAX))
    }

    fn search(conn: &Connection, query: &str, limit: u32, offset: u32) -> DbResult<Vec<Note>> {
        if query.is_empty() {
            return select(conn, limit, offset);
        }

        let fts_query = make_fts_query(query);
        let sql = "SELECT note.rowid, note.uuid4, note.title, note.url, note.tags,
             note.description, note.comments,
             hex(note.annotations) as annotations, note.created_at, note.is_public, note.metadata
             FROM note
             JOIN note_fts ON note.rowid = note_fts.rowid
             WHERE note_fts MATCH ?3
             AND note.deleted = 0
             ORDER BY bm25(note_fts, 10.0, 5.0, 3.0, 1.0)
             LIMIT ?1 OFFSET ?2";

        let mut stmt = conn.prepare(sql)?;
        let notes = stmt
            .query_map(rusqlite::params![limit, offset, fts_query], map_note)?
            .collect::<Result<Vec<_>, rusqlite::Error>>()?;
        Ok(notes)
    }

    fn search_by_day(conn: &Connection, query: &str) -> DbResult<Vec<Day>> {
        if query.is_empty() {
            return select_by_day(conn);
        }

        let fts_query = make_fts_query(query);
        let sql = format!(
            "SELECT DATE(substr(created_at, 1, 10)) as date, COUNT(1) as count
            FROM note
            WHERE deleted = 0 AND {}
            GROUP BY date
            ORDER BY date",
            fts_where_clause(1)
        );

        let mut stmt = conn.prepare(&sql)?;
        let days = stmt
            .query_map(rusqlite::params![fts_query], map_day)?
            .collect::<Result<Vec<_>, rusqlite::Error>>()?;
        Ok(days)
    }

    fn search_by_tag(conn: &Connection, query: &str) -> DbResult<Vec<Tags>> {
        if query.is_empty() {
            return select_by_tag(conn);
        }

        let fts_query = make_fts_query(query);
        let sql = format!(
            "SELECT tags
            FROM note
            WHERE deleted = 0 AND {}",
            fts_where_clause(1)
        );

        let mut tag_count_map = HashMap::new();
        let mut stmt = conn.prepare(&sql)?;
        let tags_iter =
            stmt.query_map(rusqlite::params![fts_query], |row| row.get::<_, String>(0))?;

        for tag_result in tags_iter {
            let tags_str = tag_result?;
            for tag in tags_str.split(',').map(|s| s.to_lowercase()) {
                *tag_count_map.entry(tag).or_insert(0i64) += 1;
            }
        }

        let tags = tag_count_map
            .into_iter()
            .map(|(tag, count)| Tags { tag, count })
            .collect();
        Ok(tags)
    }

    fn filter_count(conn: &Connection, query: &str, from: &str, to: &str) -> DbResult<u32> {
        let fts_query = make_fts_query(query);
        if fts_query.is_empty() {
            // No search term, but the date range must still apply (the FTS path
            // below is skipped, so we cannot fall back to the unbounded count).
            let count: i64 = conn.query_row(
                "SELECT COUNT(1)
                FROM note
                WHERE deleted = 0
                AND substr(created_at, 1, 10) >= ?1
                AND substr(created_at, 1, 10) <= ?2",
                rusqlite::params![from, to],
                |row| row.get(0),
            )?;
            return Ok(u32::try_from(count).unwrap_or(u32::MAX));
        }

        let sql = format!(
            "SELECT COUNT(1)
            FROM note
            WHERE deleted = 0
            AND substr(created_at, 1, 10) >= ?1
            AND substr(created_at, 1, 10) <= ?2
            AND {}",
            fts_where_clause(3)
        );

        let count: i64 = conn.query_row(&sql, rusqlite::params![from, to, fts_query], |row| {
            row.get(0)
        })?;
        Ok(u32::try_from(count).unwrap_or(u32::MAX))
    }

    fn filter(
        conn: &Connection,
        query: &str,
        from: &str,
        to: &str,
        limit: u32,
        offset: u32,
    ) -> DbResult<Vec<Note>> {
        let fts_query = make_fts_query(query);
        if fts_query.is_empty() {
            // No search term: date-bounded select, newest first (mirrors the
            // "empty query = show all" convention but keeps the date range).
            let mut stmt = conn.prepare(
                "SELECT rowid, uuid4, title, url, tags, description, comments,
                 hex(annotations) as annotations, created_at, is_public, metadata
                 FROM note
                 WHERE deleted = 0
                 AND substr(created_at, 1, 10) >= ?1
                 AND substr(created_at, 1, 10) <= ?2
                 ORDER BY created_at DESC
                 LIMIT ?3 OFFSET ?4",
            )?;
            let notes = stmt
                .query_map(rusqlite::params![from, to, limit, offset], map_note)?
                .collect::<Result<Vec<_>, rusqlite::Error>>()?;
            return Ok(notes);
        }

        let sql = "SELECT note.rowid, note.uuid4, note.title, note.url, note.tags,
             note.description, note.comments,
             hex(note.annotations) as annotations, note.created_at, note.is_public, note.metadata
             FROM note
             JOIN note_fts ON note.rowid = note_fts.rowid
             WHERE substr(note.created_at, 1, 10) >= ?1
             AND substr(note.created_at, 1, 10) <= ?2
             AND note_fts MATCH ?5
             AND note.deleted = 0
             ORDER BY bm25(note_fts, 10.0, 5.0, 3.0, 1.0)
             LIMIT ?3 OFFSET ?4";

        let mut stmt = conn.prepare(sql)?;
        let notes = stmt
            .query_map(
                rusqlite::params![from, to, limit, offset, fts_query],
                map_note,
            )?
            .collect::<Result<Vec<_>, rusqlite::Error>>()?;
        Ok(notes)
    }

    fn filter_by_tag(conn: &Connection, query: &str, from: &str, to: &str) -> DbResult<Vec<Tags>> {
        let fts_query = make_fts_query(query);

        if fts_query.is_empty() {
            // No search term: aggregate tags over the date-bounded set only.
            let mut tag_count_map = HashMap::new();
            let mut stmt = conn.prepare(
                "SELECT tags
                FROM note
                WHERE deleted = 0
                AND substr(created_at, 1, 10) >= ?1
                AND substr(created_at, 1, 10) <= ?2",
            )?;
            let tags_iter =
                stmt.query_map(rusqlite::params![from, to], |row| row.get::<_, String>(0))?;
            for tag_result in tags_iter {
                let tags_str = tag_result?;
                for tag in tags_str.split(',').map(|s| s.to_lowercase()) {
                    *tag_count_map.entry(tag).or_insert(0i64) += 1;
                }
            }
            let tags = tag_count_map
                .into_iter()
                .map(|(tag, count)| Tags { tag, count })
                .collect();
            return Ok(tags);
        }

        let sql = format!(
            "SELECT tags
            FROM note
            WHERE deleted = 0
            AND substr(created_at, 1, 10) >= ?1
            AND substr(created_at, 1, 10) <= ?2
            AND {}",
            fts_where_clause(3)
        );

        let mut tag_count_map = HashMap::new();
        let mut stmt = conn.prepare(&sql)?;
        let tags_iter = stmt.query_map(rusqlite::params![from, to, fts_query], |row| {
            row.get::<_, String>(0)
        })?;

        for tag_result in tags_iter {
            let tags_str = tag_result?;
            for tag in tags_str.split(',').map(|s| s.to_lowercase()) {
                *tag_count_map.entry(tag).or_insert(0i64) += 1;
            }
        }

        let tags = tag_count_map
            .into_iter()
            .map(|(tag, count)| Tags { tag, count })
            .collect();
        Ok(tags)
    }

    /// Fetch all notes from the database (no pagination).
    pub fn select_all(conn: &Connection) -> DbResult<Vec<Note>> {
        let mut stmt = conn.prepare(
            "SELECT rowid, uuid4, title, url, tags, description, comments,
             hex(annotations) as annotations, created_at, is_public, metadata
             FROM note
             WHERE deleted = 0
             ORDER BY created_at DESC",
        )?;
        let notes = stmt
            .query_map([], map_note)?
            .collect::<Result<Vec<_>, rusqlite::Error>>()?;
        Ok(notes)
    }

    /// Fetch notes matching a search query (no pagination).
    pub fn search_all(conn: &Connection, query: &str) -> DbResult<Vec<Note>> {
        if query.is_empty() {
            return select_all(conn);
        }

        let fts_query = make_fts_query(query);
        let sql = "SELECT note.rowid, note.uuid4, note.title, note.url, note.tags,
             note.description, note.comments,
             hex(note.annotations) as annotations, note.created_at, note.is_public, note.metadata
             FROM note
             JOIN note_fts ON note.rowid = note_fts.rowid
             WHERE note_fts MATCH ?1
             AND note.deleted = 0
             ORDER BY bm25(note_fts, 10.0, 5.0, 3.0, 1.0)";

        let mut stmt = conn.prepare(sql)?;
        let notes = stmt
            .query_map(rusqlite::params![fts_query], map_note)?
            .collect::<Result<Vec<_>, rusqlite::Error>>()?;
        Ok(notes)
    }

    /// Search with FTS5 snippet highlights for matched terms.
    ///
    /// Returns [`SearchResult`] with `<b>`/`</b>` markers around matched
    /// terms in title and description snippets. Falls back to plain select
    /// with `None` snippets when the query is empty.
    pub fn search_with_snippets(
        conn: &Connection,
        query: &str,
        limit: u32,
        offset: u32,
    ) -> DbResult<Vec<models::SearchResult>> {
        if query.is_empty() {
            let notes = select(conn, limit, offset)?;
            return Ok(notes
                .into_iter()
                .map(|note| models::SearchResult {
                    note,
                    title_snippet: None,
                    description_snippet: None,
                })
                .collect());
        }

        let fts_query = make_fts_query(query);
        let sql = "SELECT note.rowid, note.uuid4, note.title, note.url, note.tags,
             note.description, note.comments,
             hex(note.annotations) as annotations, note.created_at, note.is_public, note.metadata,
             snippet(note_fts, 0, '<b>', '</b>', '...', 32) as title_snippet,
             snippet(note_fts, 3, '<b>', '</b>', '...', 32) as description_snippet
             FROM note
             JOIN note_fts ON note.rowid = note_fts.rowid
             WHERE note_fts MATCH ?1
             AND note.deleted = 0
             ORDER BY bm25(note_fts, 10.0, 5.0, 3.0, 1.0)
             LIMIT ?2 OFFSET ?3";

        let mut stmt = conn.prepare(sql)?;
        let results = stmt
            .query_map(rusqlite::params![fts_query, limit, offset], |row| {
                let note = map_note(row)?;
                let title_snippet: Option<String> = row.get("title_snippet")?;
                let description_snippet: Option<String> = row.get("description_snippet")?;
                Ok(models::SearchResult {
                    note,
                    title_snippet: title_snippet.filter(|s| !s.is_empty()),
                    description_snippet: description_snippet.filter(|s| !s.is_empty()),
                })
            })?
            .collect::<Result<Vec<_>, rusqlite::Error>>()?;
        Ok(results)
    }

    /// Build an FTS5 match expression from a user query string.
    ///
    /// Each whitespace-separated word is quoted (to escape FTS5 operators)
    /// and combined with AND so all terms must appear. A trailing `*` is
    /// added for prefix matching, giving behaviour similar to the old
    /// LIKE `%word%` approach.
    pub(crate) fn make_fts_query(query: &str) -> String {
        static WHITESPACE_RE: OnceLock<Regex> = OnceLock::new();
        let re = WHITESPACE_RE.get_or_init(|| Regex::new(r"\s+").expect("static regex"));
        re.replace_all(query.trim(), " ")
            .split(' ')
            .filter(|w| !w.is_empty())
            .map(|w| {
                // Escape double-quotes inside the term, then wrap in quotes
                // and add prefix wildcard for substring-like matching.
                let escaped = w.replace('"', "\"\"");
                format!("\"{escaped}\" *")
            })
            .collect::<Vec<_>>()
            .join(" AND ")
    }

    /// Return a WHERE clause that joins the note table with the FTS index.
    ///
    /// `fts_param` is the SQL parameter index (1-based) that will hold the
    /// FTS5 match expression.
    pub(crate) fn fts_where_clause(fts_param: usize) -> String {
        format!("rowid IN (SELECT rowid FROM note_fts WHERE note_fts MATCH ?{fts_param})")
    }

    // ── Metadata helpers ───────────────────────────────────────────────

    /// Extract a single value from the JSON metadata column of a note.
    ///
    /// Uses SQLite's `json_extract()` function. Returns `None` when the key
    /// does not exist or the value is JSON null.
    pub fn get_metadata_value(
        conn: &Connection,
        rowid: i64,
        key: &str,
    ) -> DbResult<Option<String>> {
        let path = format!("$.{key}");
        let value: Option<String> = conn
            .query_row(
                "SELECT json_extract(metadata, ?1) FROM note WHERE rowid = ?2",
                rusqlite::params![path, rowid],
                |row| row.get(0),
            )
            .map_err(DbError::from)?;
        Ok(value)
    }

    /// Set (or overwrite) a single key in the JSON metadata column of a note.
    ///
    /// Uses SQLite's `json_set()` function so the rest of the object is
    /// preserved.
    pub fn set_metadata_value(
        conn: &Connection,
        rowid: i64,
        key: &str,
        value: &str,
    ) -> DbResult<()> {
        let path = format!("$.{key}");
        conn.execute(
            "UPDATE note SET metadata = json_set(metadata, ?1, ?2) WHERE rowid = ?3",
            rusqlite::params![path, value, rowid],
        )?;
        Ok(())
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_validate_sync_file_path_relative() {
            assert!(validate_sync_file_path("relative/path.sqlite3").is_err());
        }

        #[test]
        fn test_validate_sync_file_path_wrong_extension() {
            let tmp = std::env::temp_dir().join("test_wrong_ext.txt");
            std::fs::write(&tmp, "test").unwrap();
            let result = validate_sync_file_path(tmp.to_str().unwrap());
            std::fs::remove_file(&tmp).ok();
            assert!(result.is_err());
        }

        #[test]
        fn test_validate_sync_file_path_nonexistent() {
            assert!(validate_sync_file_path("/tmp/nonexistent_file.sqlite3").is_err());
        }

        #[test]
        fn test_validate_sync_file_path_valid() {
            let tmp = std::env::temp_dir().join("test_valid.sqlite3");
            std::fs::write(&tmp, "test").unwrap();
            let result = validate_sync_file_path(tmp.to_str().unwrap());
            std::fs::remove_file(&tmp).ok();
            assert!(result.is_ok());
        }
    }
}

pub mod migrations {
    use super::*;
    use models::Note;
    use rusqlite::Connection;
    use semver::Version;
    use uuid::Uuid;

    type MigrationFn = fn(&Connection) -> DbResult<()>;

    const MIGRATIONS: &[(Version, MigrationFn)] = &[
        (Version::new(0, 4, 0), migrate_schema),
        (Version::new(0, 4, 1), migrate_note),
        (Version::new(0, 5, 0), drop_ssb_table),
        (Version::new(0, 6, 0), migrate_created_at),
        (Version::new(0, 7, 0), migrate_fts5),
        (Version::new(0, 8, 0), migrate_metadata),
        (Version::new(0, 9, 0), migrate_fts5_trigram),
        (Version::new(0, 10, 0), migrate_lww),
    ];

    pub fn upgrade(conn: &Connection) -> DbResult<()> {
        if !table_exists(conn)? {
            return init_db_schema(conn);
        }

        let current_version = Version::parse(&get_meta_version(conn)?)?;

        for (version, migrate) in MIGRATIONS {
            if current_version < *version {
                migrate(conn)?;
                set_meta_version(conn, &version.to_string())?;
            }
        }

        Ok(())
    }

    fn table_exists(conn: &Connection) -> DbResult<bool> {
        let exists: bool = conn.query_row(
            "SELECT EXISTS (SELECT 1 FROM sqlite_master WHERE type='table' AND name='note')",
            [],
            |row| row.get(0),
        )?;
        Ok(exists)
    }

    pub fn get_meta_version(conn: &Connection) -> DbResult<String> {
        let row: Option<String> = conn
            .query_row(
                "SELECT meta_value FROM meta WHERE meta_key = 'version'",
                [],
                |row| row.get(0),
            )
            .ok();
        Ok(row.unwrap_or_else(|| "0.3.10".to_string()))
    }

    fn set_meta_version(conn: &Connection, version: &str) -> DbResult<()> {
        conn.execute(
            "INSERT OR REPLACE INTO meta (meta_key, meta_value) VALUES ('version', ?1)",
            rusqlite::params![version],
        )?;
        Ok(())
    }

    fn init_db_schema(conn: &Connection) -> DbResult<()> {
        conn.execute_batch(
            "CREATE TABLE note (
                 rowid INTEGER PRIMARY KEY AUTOINCREMENT,
                 uuid4 TEXT NOT NULL UNIQUE,
                 title TEXT NOT NULL,
                 url TEXT NOT NULL,
                 tags TEXT NOT NULL,
                 description TEXT NOT NULL,
                 comments TEXT NOT NULL,
                 annotations TEXT NOT NULL,
                 created_at TEXT NOT NULL,
                 is_public BOOLEAN NOT NULL DEFAULT 0,
                 metadata TEXT NOT NULL DEFAULT '{}',
                 updated_at TEXT NOT NULL DEFAULT '',
                 deleted INTEGER NOT NULL DEFAULT 0
             );
             CREATE TABLE IF NOT EXISTS meta (
                 meta_key TEXT PRIMARY KEY,
                 meta_value TEXT NOT NULL
             );
             CREATE VIRTUAL TABLE IF NOT EXISTS note_fts USING fts5(
                 title, url, tags, description,
                 content=note, content_rowid=rowid
             );
             CREATE TRIGGER IF NOT EXISTS note_fts_insert AFTER INSERT ON note BEGIN
                 INSERT INTO note_fts(rowid, title, url, tags, description)
                 VALUES (new.rowid, new.title, new.url, new.tags, new.description);
             END;
             CREATE TRIGGER IF NOT EXISTS note_fts_delete AFTER DELETE ON note BEGIN
                 INSERT INTO note_fts(note_fts, rowid, title, url, tags, description)
                 VALUES ('delete', old.rowid, old.title, old.url, old.tags, old.description);
             END;
             CREATE TRIGGER IF NOT EXISTS note_fts_update AFTER UPDATE ON note BEGIN
                 INSERT INTO note_fts(note_fts, rowid, title, url, tags, description)
                 VALUES ('delete', old.rowid, old.title, old.url, old.tags, old.description);
                 INSERT INTO note_fts(rowid, title, url, tags, description)
                 VALUES (new.rowid, new.title, new.url, new.tags, new.description);
             END;
             INSERT INTO meta (meta_key, meta_value) VALUES ('version', '0.8.0');",
        )?;
        // Seed a stable node id used as the last-write-wins tiebreaker.
        conn.execute(
            "INSERT OR IGNORE INTO meta (meta_key, meta_value) VALUES ('node_id', ?1)",
            rusqlite::params![Uuid::new_v4().to_string()],
        )?;
        Ok(())
    }

    fn migrate_schema(conn: &Connection) -> DbResult<()> {
        conn.execute_batch(
            "ALTER TABLE note RENAME TO _note_0_3;
             CREATE TABLE note (
                 rowid INTEGER PRIMARY KEY AUTOINCREMENT,
                 uuid4 TEXT NOT NULL UNIQUE,
                 title TEXT NOT NULL,
                 url TEXT NOT NULL,
                 tags TEXT NOT NULL,
                 description TEXT NOT NULL,
                 comments TEXT NOT NULL,
                 annotations TEXT NOT NULL,
                 created_at TEXT NOT NULL,
                 is_public BOOLEAN NOT NULL DEFAULT 0
             );
             CREATE TABLE IF NOT EXISTS meta (
                 meta_key TEXT PRIMARY KEY,
                 meta_value TEXT NOT NULL
             );
             INSERT INTO meta (meta_key, meta_value) VALUES ('version', '0.4.0'), ('is_upgrading', '1');",
        )?;
        Ok(())
    }

    fn migrate_note(conn: &Connection) -> DbResult<()> {
        let mut stmt = conn.prepare(
            "SELECT rowid, title, url, tags, description, comments, annotations, created_at, is_public FROM _note_0_3 ORDER BY rowid",
        )?;
        let notes: Vec<Note> = stmt
            .query_map([], |row| {
                Ok(Note {
                    rowid: row.get(0)?,
                    uuid4: String::new(),
                    title: row.get(1)?,
                    url: row.get(2)?,
                    tags: row.get(3)?,
                    description: row.get(4)?,
                    comments: row.get(5)?,
                    annotations: row.get(6)?,
                    created_at: row.get(7)?,
                    is_public: row.get(8)?,
                    metadata: String::new(),
                    updated_at: String::new(),
                    deleted: false,
                })
            })?
            .collect::<Result<_, rusqlite::Error>>()?;

        for note in notes {
            let uuid4 = Uuid::new_v4().to_string();
            conn.execute(
                "INSERT INTO note (uuid4, title, url, tags, description, comments, annotations, created_at, is_public)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                rusqlite::params![
                    uuid4,
                    note.title,
                    note.url,
                    note.tags,
                    note.description,
                    note.comments,
                    note.annotations,
                    note.created_at,
                    note.is_public
                ],
            )?;
        }

        conn.execute_batch(
            "DROP TABLE _note_0_3; UPDATE meta SET meta_value = '0' WHERE meta_key = 'is_upgrading';",
        )?;
        Ok(())
    }

    fn drop_ssb_table(conn: &Connection) -> DbResult<()> {
        conn.execute_batch("DROP TABLE IF EXISTS ssb;")?;
        Ok(())
    }

    fn migrate_created_at(conn: &Connection) -> DbResult<()> {
        let mut stmt = conn.prepare("SELECT rowid, created_at FROM note")?;
        let rows: Vec<(i64, String)> = stmt
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?
            .collect::<Result<_, rusqlite::Error>>()?;

        for (rowid, created_at) in rows {
            let new_created_at = parse_old_created_at(&created_at)?;
            conn.execute(
                "UPDATE note SET created_at = ?1 WHERE rowid = ?2",
                rusqlite::params![new_created_at, rowid],
            )?;
        }

        Ok(())
    }

    fn migrate_fts5(conn: &Connection) -> DbResult<()> {
        conn.execute_batch(
            "CREATE VIRTUAL TABLE IF NOT EXISTS note_fts USING fts5(
                 title, url, tags, description,
                 content=note, content_rowid=rowid
             );
             INSERT INTO note_fts(rowid, title, url, tags, description)
                 SELECT rowid, title, url, tags, description FROM note;
             CREATE TRIGGER IF NOT EXISTS note_fts_insert AFTER INSERT ON note BEGIN
                 INSERT INTO note_fts(rowid, title, url, tags, description)
                 VALUES (new.rowid, new.title, new.url, new.tags, new.description);
             END;
             CREATE TRIGGER IF NOT EXISTS note_fts_delete AFTER DELETE ON note BEGIN
                 INSERT INTO note_fts(note_fts, rowid, title, url, tags, description)
                 VALUES ('delete', old.rowid, old.title, old.url, old.tags, old.description);
             END;
             CREATE TRIGGER IF NOT EXISTS note_fts_update AFTER UPDATE ON note BEGIN
                 INSERT INTO note_fts(note_fts, rowid, title, url, tags, description)
                 VALUES ('delete', old.rowid, old.title, old.url, old.tags, old.description);
                 INSERT INTO note_fts(rowid, title, url, tags, description)
                 VALUES (new.rowid, new.title, new.url, new.tags, new.description);
             END;",
        )?;
        Ok(())
    }

    fn migrate_metadata(conn: &Connection) -> DbResult<()> {
        conn.execute_batch("ALTER TABLE note ADD COLUMN metadata TEXT NOT NULL DEFAULT '{}';")?;
        Ok(())
    }

    fn migrate_fts5_trigram(conn: &Connection) -> DbResult<()> {
        conn.execute_batch(
            "CREATE VIRTUAL TABLE IF NOT EXISTS note_fts_trigram USING fts5(
                 title, url, tags, description,
                 content=note, content_rowid=rowid,
                 tokenize='trigram'
             );
             INSERT INTO note_fts_trigram(rowid, title, url, tags, description)
                 SELECT rowid, title, url, tags, description FROM note;
             CREATE TRIGGER IF NOT EXISTS note_fts_trigram_insert AFTER INSERT ON note BEGIN
                 INSERT INTO note_fts_trigram(rowid, title, url, tags, description)
                 VALUES (new.rowid, new.title, new.url, new.tags, new.description);
             END;
             CREATE TRIGGER IF NOT EXISTS note_fts_trigram_delete AFTER DELETE ON note BEGIN
                 INSERT INTO note_fts_trigram(note_fts_trigram, rowid, title, url, tags, description)
                 VALUES ('delete', old.rowid, old.title, old.url, old.tags, old.description);
             END;
             CREATE TRIGGER IF NOT EXISTS note_fts_trigram_update AFTER UPDATE ON note BEGIN
                 INSERT INTO note_fts_trigram(note_fts_trigram, rowid, title, url, tags, description)
                 VALUES ('delete', old.rowid, old.title, old.url, old.tags, old.description);
                 INSERT INTO note_fts_trigram(rowid, title, url, tags, description)
                 VALUES (new.rowid, new.title, new.url, new.tags, new.description);
             END;",
        )?;
        Ok(())
    }

    /// Add last-write-wins / tombstone columns for conflict-resolving sync.
    ///
    /// Idempotent: `init_db_schema` already creates these columns on a freshly
    /// initialised database (which starts at version 0.8.0 and then runs the
    /// later migrations), so guard each `ALTER TABLE` with a column-existence
    /// check to avoid a duplicate-column error.
    fn migrate_lww(conn: &Connection) -> DbResult<()> {
        if !column_exists(conn, "note", "updated_at")? {
            conn.execute_batch("ALTER TABLE note ADD COLUMN updated_at TEXT NOT NULL DEFAULT '';")?;
        }
        if !column_exists(conn, "note", "deleted")? {
            conn.execute_batch("ALTER TABLE note ADD COLUMN deleted INTEGER NOT NULL DEFAULT 0;")?;
        }
        // Seed a stable node id (LWW tiebreaker) if absent.
        conn.execute(
            "INSERT OR IGNORE INTO meta (meta_key, meta_value) VALUES ('node_id', ?1)",
            rusqlite::params![Uuid::new_v4().to_string()],
        )?;
        // Backfill updated_at for legacy rows so they order deterministically
        // against later edits: derive epoch-millis from created_at + node id.
        conn.execute(
            "UPDATE note
             SET updated_at = printf(
                 '%020d-%s',
                 CAST(strftime('%s', created_at) AS INTEGER) * 1000,
                 (SELECT meta_value FROM meta WHERE meta_key = 'node_id')
             )
             WHERE updated_at IS NULL OR updated_at = ''",
            [],
        )?;
        Ok(())
    }

    fn parse_old_created_at(created_at: &str) -> DbResult<String> {
        let created_at = created_at.trim_end_matches(" UTC");
        let parts: Vec<&str> = created_at.split(':').collect();
        if parts.len() != 4 {
            return Err(DatabaseError::InvalidFormat);
        }
        // "YYYY-MM-DD HH:MM:SS" (drop the nanoseconds part)
        Ok(format!("{}:{}:{}", parts[0], parts[1], parts[2]))
    }
}

pub mod sync {
    use super::*;
    use crate::error::{DbError, DbResult};
    use models::Note;
    use rusqlite::Connection;
    use std::collections::HashMap;

    pub fn get_meta_version(conn: &Connection) -> DbResult<String> {
        let row: Option<String> = conn
            .query_row(
                "SELECT meta_value FROM meta WHERE meta_key = 'version'",
                [],
                |row| row.get(0),
            )
            .ok();
        Ok(row.unwrap_or_else(|| "0.3.10".to_string()))
    }

    pub fn get_note_by_uuid4(conn: &Connection, uuid4: &str) -> DbResult<Note> {
        conn.query_row(
            "SELECT rowid, uuid4, title, url, tags, description, comments, hex(annotations) as annotations, created_at, is_public, metadata, updated_at, deleted FROM note WHERE uuid4 = ?1",
            rusqlite::params![uuid4],
            |row| {
                Ok(Note {
                    rowid: row.get("rowid")?,
                    uuid4: row.get("uuid4")?,
                    title: row.get("title")?,
                    url: row.get("url")?,
                    tags: row.get("tags")?,
                    description: row.get("description")?,
                    comments: row.get("comments")?,
                    annotations: row.get("annotations")?,
                    created_at: row.get("created_at")?,
                    is_public: row.get("is_public")?,
                    metadata: row.get("metadata")?,
                    updated_at: row.get("updated_at")?,
                    deleted: row.get("deleted")?,
                })
            },
        )
        .map_err(DbError::from)
    }

    /// `(uuid4, updated_at)` for every note, **including tombstones**, so that
    /// both new notes and deletions are advertised to peers during sync.
    pub fn note_versions(conn: &Connection) -> DbResult<Vec<(String, String)>> {
        let mut stmt = conn.prepare("SELECT uuid4, updated_at FROM note ORDER BY rowid")?;
        let rows = stmt
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?
            .collect::<Result<Vec<(String, String)>, rusqlite::Error>>()?;
        Ok(rows)
    }

    /// Given the *client's* `(uuid4, updated_at)` versions, return the uuid4s the
    /// client should PUSH to this server: notes the server lacks entirely, or for
    /// which the client holds a strictly newer version (last-write-wins).
    pub fn diff_to_server(
        conn: &Connection,
        client_versions: Vec<(String, String)>,
    ) -> DbResult<Vec<String>> {
        let mut r = Vec::new();
        for (uuid4, client_updated_at) in client_versions {
            let server_updated_at: Option<String> = conn
                .query_row(
                    "SELECT updated_at FROM note WHERE uuid4 = ?1",
                    rusqlite::params![uuid4],
                    |row| row.get(0),
                )
                .ok();
            match server_updated_at {
                None => r.push(uuid4),
                Some(server) if client_updated_at > server => r.push(uuid4),
                _ => {}
            }
        }
        Ok(r)
    }

    /// Given the *client's* `(uuid4, updated_at)` versions, return the uuid4s the
    /// client should PULL from this server: notes the client lacks entirely, or
    /// for which the server holds a strictly newer version (last-write-wins).
    pub fn diff_from_server(
        conn: &Connection,
        client_versions: Vec<(String, String)>,
    ) -> DbResult<Vec<String>> {
        let client: HashMap<String, String> = client_versions.into_iter().collect();
        let mut r = Vec::new();
        let mut stmt = conn.prepare("SELECT uuid4, updated_at FROM note")?;
        let server_versions = stmt
            .query_map([], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
            })?
            .collect::<Result<Vec<(String, String)>, rusqlite::Error>>()?;
        for (uuid4, server_updated_at) in server_versions {
            match client.get(&uuid4) {
                None => r.push(uuid4),
                Some(client_updated_at) if *client_updated_at < server_updated_at => r.push(uuid4),
                _ => {}
            }
        }
        Ok(r)
    }

    /// Apply a note received from a peer using last-write-wins: insert it when
    /// new, or overwrite the local copy only when the incoming `updated_at` is
    /// strictly newer. The tombstone flag is carried so deletions replicate.
    pub fn insert(conn: &Connection, note: &Note) -> DbResult<()> {
        let annotations_blob = hex::decode(&note.annotations).unwrap_or_else(|e| {
            tracing::warn!(uuid4 = note.uuid4, %e, "failed to decode hex annotations");
            Vec::new()
        });
        let metadata = if note.metadata.is_empty() {
            "{}".to_string()
        } else {
            note.metadata.clone()
        };
        // A peer on a pre-0.10 core may send an empty updated_at; mint a local
        // token so the row still carries an ordering value.
        let updated_at = if note.updated_at.is_empty() {
            next_update_token(conn)?
        } else {
            note.updated_at.clone()
        };
        conn.execute(
            "INSERT INTO note (uuid4, title, url, tags, description, comments, annotations, created_at, is_public, metadata, updated_at, deleted)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)
            ON CONFLICT(uuid4) DO UPDATE SET
                title = excluded.title,
                url = excluded.url,
                tags = excluded.tags,
                description = excluded.description,
                comments = excluded.comments,
                annotations = excluded.annotations,
                created_at = excluded.created_at,
                is_public = excluded.is_public,
                metadata = excluded.metadata,
                updated_at = excluded.updated_at,
                deleted = excluded.deleted
            WHERE excluded.updated_at > note.updated_at",
            rusqlite::params![
                note.uuid4,
                note.title,
                note.url,
                note.tags,
                note.description,
                note.comments,
                annotations_blob,
                note.created_at,
                note.is_public,
                metadata,
                updated_at,
                note.deleted
            ],
        )?;
        Ok(())
    }
}

// ── SQLCipher encryption support ──────────────────────────────────────────
//
// All functions in this module require the `encryption` feature flag *and*
// the workspace-level `bundled-sqlcipher` rusqlite feature (which replaces
// `bundled`).  When neither feature is active the default build is entirely
// unchanged.

#[cfg(feature = "encryption")]
pub mod encryption {
    use super::*;
    use rusqlite::Connection;
    use std::path::Path;

    /// Apply the encryption key to an already-opened SQLCipher connection.
    ///
    /// This **must** be the first statement executed on the connection
    /// (before any other SQL), otherwise SQLCipher will treat the file as
    /// plain-text and subsequent operations will fail.
    pub fn set_encryption_key(conn: &Connection, key: &str) -> DbResult<()> {
        // Use PRAGMA key with single-quote escaping to avoid injection.
        conn.execute_batch(&format!("PRAGMA key = '{}';", key.replace('\'', "''")))
            .map_err(DatabaseError::from)
    }

    /// Re-key (change the passphrase of) an already-unlocked database.
    ///
    /// The connection must have been opened and unlocked with
    /// [`set_encryption_key`] first.  After this call succeeds the database
    /// file on disk is re-encrypted with `new_key`.
    pub fn change_encryption_key(conn: &Connection, new_key: &str) -> DbResult<()> {
        conn.execute_batch(&format!(
            "PRAGMA rekey = '{}';",
            new_key.replace('\'', "''")
        ))
        .map_err(DatabaseError::from)
    }

    /// Open (or create) the database at the default platform location and
    /// unlock it with the given encryption key, then run migrations.
    ///
    /// This is the encrypted counterpart of [`init_db`](super::init_db).
    pub fn init_db_encrypted(key: &str) -> DbResult<Connection> {
        let db_path = utils::sqlite3_db_location()?;
        tracing::info!(db_path, "opening encrypted database");
        let conn = Connection::open(&db_path)?;

        #[cfg(target_os = "android")]
        conn.execute_batch("PRAGMA temp_store_directory = '/data/data/app.localnative/cache'")?;

        set_encryption_key(&conn, key)?;
        migrations::upgrade(&conn)?;

        Ok(conn)
    }

    /// Migrate an existing *unencrypted* database to a new *encrypted* copy.
    ///
    /// 1. Opens `source_path` as a plain-text SQLite database.
    /// 2. Creates a new encrypted database at `dest_path`.
    /// 3. Copies all data using `ATTACH` + `sqlcipher_export()`.
    ///
    /// `dest_path` must not already exist.  On success the caller can swap
    /// the files and start using the encrypted database.
    pub fn encrypt_existing_db(source_path: &str, dest_path: &str, key: &str) -> DbResult<()> {
        // Safety: dest must not exist yet.
        if Path::new(dest_path).exists() {
            return Err(DatabaseError::IoError(std::io::Error::new(
                std::io::ErrorKind::AlreadyExists,
                format!("destination already exists: {dest_path}"),
            )));
        }

        let source_conn = Connection::open(source_path)?;

        // Attach the (not-yet-existing) destination as an encrypted database.
        source_conn.execute_batch(&format!(
            "ATTACH DATABASE '{}' AS encrypted KEY '{}';",
            dest_path.replace('\'', "''"),
            key.replace('\'', "''"),
        ))?;

        // Copy all schema and data from main to the encrypted database.
        source_conn.execute_batch("SELECT sqlcipher_export('encrypted');")?;

        source_conn.execute_batch("DETACH DATABASE encrypted;")?;

        Ok(())
    }
}

#[cfg(test)]
mod db_tests {
    use super::*;
    use models::Note;
    use rusqlite::Connection;

    fn setup_test_db() -> Connection {
        let conn = Connection::open_in_memory().expect("in-memory db");
        conn.execute_batch(
            "CREATE TABLE note (
                 rowid INTEGER PRIMARY KEY AUTOINCREMENT,
                 uuid4 TEXT NOT NULL UNIQUE,
                 title TEXT NOT NULL,
                 url TEXT NOT NULL,
                 tags TEXT NOT NULL,
                 description TEXT NOT NULL,
                 comments TEXT NOT NULL,
                 annotations TEXT NOT NULL,
                 created_at TEXT NOT NULL,
                 is_public BOOLEAN NOT NULL DEFAULT 0,
                 metadata TEXT NOT NULL DEFAULT '{}',
                 updated_at TEXT NOT NULL DEFAULT '',
                 deleted INTEGER NOT NULL DEFAULT 0
             );
             CREATE TABLE IF NOT EXISTS meta (
                 meta_key TEXT PRIMARY KEY,
                 meta_value TEXT NOT NULL
             );
             CREATE VIRTUAL TABLE IF NOT EXISTS note_fts USING fts5(
                 title, url, tags, description,
                 content=note, content_rowid=rowid
             );
             CREATE TRIGGER note_fts_insert AFTER INSERT ON note BEGIN
                 INSERT INTO note_fts(rowid, title, url, tags, description)
                 VALUES (new.rowid, new.title, new.url, new.tags, new.description);
             END;
             CREATE TRIGGER note_fts_delete AFTER DELETE ON note BEGIN
                 INSERT INTO note_fts(note_fts, rowid, title, url, tags, description)
                 VALUES ('delete', old.rowid, old.title, old.url, old.tags, old.description);
             END;
             CREATE TRIGGER note_fts_update AFTER UPDATE ON note BEGIN
                 INSERT INTO note_fts(note_fts, rowid, title, url, tags, description)
                 VALUES ('delete', old.rowid, old.title, old.url, old.tags, old.description);
                 INSERT INTO note_fts(rowid, title, url, tags, description)
                 VALUES (new.rowid, new.title, new.url, new.tags, new.description);
             END;
             INSERT INTO meta (meta_key, meta_value) VALUES ('version', '0.8.0');",
        )
        .expect("schema init");
        conn
    }

    #[test]
    fn test_insert_and_select() {
        let conn = setup_test_db();
        let note = queries::insert_note(
            &conn,
            "Test Title",
            "https://example.com",
            "rust,test",
            "A test note",
            "no comments",
            b"hello",
            false,
        )
        .expect("insert should succeed");

        assert_eq!(note.title, "Test Title");
        assert_eq!(note.url, "https://example.com");
        assert_eq!(note.tags, "rust,test");
        assert!(!note.uuid4.is_empty());

        let result = queries::do_select(&conn, 10, 0).expect("select should succeed");
        assert_eq!(result.count, 1);
        assert_eq!(result.notes.len(), 1);
        assert_eq!(result.notes[0].title, "Test Title");
    }

    #[test]
    fn test_filter_by_day_with_empty_query() {
        // Regression: do_filter with an empty query must still honour the date
        // range (e.g. clicking a day in a GUI with no active search). Previously
        // the empty-FTS path fell back to an unbounded select and returned ALL
        // notes, silently ignoring the from/to bounds.
        let conn = setup_test_db();
        queries::insert_note_with_timestamp(
            &conn,
            "Day One",
            "https://one.example",
            "a,b",
            "",
            "",
            b"",
            true,
            "2026-06-20 09:00:00",
        )
        .unwrap();
        queries::insert_note_with_timestamp(
            &conn,
            "Day Two",
            "https://two.example",
            "b,c",
            "",
            "",
            b"",
            true,
            "2026-06-21 09:00:00",
        )
        .unwrap();
        queries::insert_note_with_timestamp(
            &conn,
            "Day Two Again",
            "https://three.example",
            "c",
            "",
            "",
            b"",
            true,
            "2026-06-21 18:30:00",
        )
        .unwrap();

        // Empty query + single-day range must return only that day's notes.
        let day = queries::do_filter(&conn, "", 10, 0, "2026-06-21", "2026-06-21")
            .expect("filter should succeed");
        assert_eq!(day.count, 2, "only the two 2026-06-21 notes should match");
        assert_eq!(day.notes.len(), 2);
        assert!(
            day.notes
                .iter()
                .all(|n| n.created_at.starts_with("2026-06-21"))
        );
        // Tag aggregation is date-scoped too: 'a' (only on 2026-06-20) is absent.
        assert!(day.tags.iter().all(|t| t.tag != "a"));
        assert!(day.tags.iter().any(|t| t.tag == "c"));

        // The other day still returns its single note.
        let other = queries::do_filter(&conn, "", 10, 0, "2026-06-20", "2026-06-20")
            .expect("filter should succeed");
        assert_eq!(other.count, 1);
        assert_eq!(other.notes[0].title, "Day One");
    }

    #[test]
    fn test_export_db() {
        let conn = setup_test_db();
        queries::insert_note(
            &conn,
            "Exported Note",
            "https://export.example",
            "x,y",
            "",
            "",
            b"",
            true,
        )
        .unwrap();

        let dest = std::env::temp_dir().join(format!("ln_export_{}.sqlite3", std::process::id()));
        let dest_str = dest.to_str().unwrap();

        // Export, then export again to confirm an existing target is replaced.
        queries::export_db(&conn, dest_str).expect("export should succeed");
        queries::export_db(&conn, dest_str).expect("re-export should overwrite");

        // The exported copy is a standalone file with the data present.
        let exported = Connection::open(dest_str).expect("open exported db");
        let count: i64 = exported
            .query_row("SELECT COUNT(1) FROM note", [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 1);
        let title: String = exported
            .query_row("SELECT title FROM note LIMIT 1", [], |r| r.get(0))
            .unwrap();
        assert_eq!(title, "Exported Note");

        std::fs::remove_file(&dest).ok();

        // Empty destination is rejected.
        assert!(queries::export_db(&conn, "  ").is_err());
    }

    #[test]
    fn test_search() {
        let conn = setup_test_db();
        queries::insert_note(
            &conn,
            "Rust Programming",
            "https://rust-lang.org",
            "rust,lang",
            "Learn Rust",
            "",
            b"",
            true,
        )
        .unwrap();
        queries::insert_note(
            &conn,
            "Python Guide",
            "https://python.org",
            "python",
            "Learn Python",
            "",
            b"",
            false,
        )
        .unwrap();

        let result = queries::do_search(&conn, "rust", 10, 0).expect("search should succeed");
        assert_eq!(result.count, 1);
        assert_eq!(result.notes[0].title, "Rust Programming");

        let result = queries::do_search(&conn, "learn", 10, 0).expect("search should succeed");
        assert_eq!(result.count, 2);

        let result = queries::do_search(&conn, "", 10, 0).expect("empty search returns all");
        assert_eq!(result.count, 2);
    }

    #[test]
    fn test_delete() {
        let conn = setup_test_db();
        let note = queries::insert_note(
            &conn,
            "To Delete",
            "https://example.com",
            "tmp",
            "",
            "",
            b"",
            false,
        )
        .unwrap();

        let result = queries::do_select(&conn, 10, 0).unwrap();
        assert_eq!(result.count, 1);

        queries::delete_note(&conn, note.rowid).expect("delete should succeed");

        let result = queries::do_select(&conn, 10, 0).unwrap();
        assert_eq!(result.count, 0);
    }

    #[test]
    fn test_search_multiple_words() {
        let conn = setup_test_db();
        queries::insert_note(
            &conn,
            "Rust Web Framework",
            "https://actix.rs",
            "rust,web",
            "Fast web framework",
            "",
            b"",
            false,
        )
        .unwrap();
        queries::insert_note(
            &conn,
            "Rust CLI Tools",
            "https://clap.rs",
            "rust,cli",
            "CLI framework",
            "",
            b"",
            false,
        )
        .unwrap();
        queries::insert_note(
            &conn,
            "Python Web",
            "https://django.com",
            "python,web",
            "Django framework",
            "",
            b"",
            false,
        )
        .unwrap();

        let result = queries::do_search(&conn, "rust web", 10, 0).unwrap();
        assert_eq!(result.count, 1);
        assert_eq!(result.notes[0].title, "Rust Web Framework");
    }

    #[test]
    fn test_tags_aggregation() {
        let conn = setup_test_db();
        queries::insert_note(
            &conn,
            "Note 1",
            "https://a.com",
            "rust,web",
            "",
            "",
            b"",
            false,
        )
        .unwrap();
        queries::insert_note(
            &conn,
            "Note 2",
            "https://b.com",
            "rust,cli",
            "",
            "",
            b"",
            false,
        )
        .unwrap();
        queries::insert_note(
            &conn,
            "Note 3",
            "https://c.com",
            "python,web",
            "",
            "",
            b"",
            false,
        )
        .unwrap();

        let result = queries::do_select(&conn, 10, 0).unwrap();
        let tag_map: std::collections::HashMap<_, _> =
            result.tags.into_iter().map(|t| (t.tag, t.count)).collect();
        assert_eq!(*tag_map.get("rust").unwrap(), 2);
        assert_eq!(*tag_map.get("web").unwrap(), 2);
        assert_eq!(*tag_map.get("cli").unwrap(), 1);
        assert_eq!(*tag_map.get("python").unwrap(), 1);
    }

    #[test]
    fn test_pagination() {
        let conn = setup_test_db();
        for i in 0..5 {
            queries::insert_note(
                &conn,
                &format!("Note {}", i),
                "https://example.com",
                "tag",
                "",
                "",
                b"",
                false,
            )
            .unwrap();
        }

        let result = queries::do_select(&conn, 2, 0).unwrap();
        assert_eq!(result.count, 5);
        assert_eq!(result.notes.len(), 2);

        let result = queries::do_select(&conn, 2, 2).unwrap();
        assert_eq!(result.notes.len(), 2);

        let result = queries::do_select(&conn, 2, 4).unwrap();
        assert_eq!(result.notes.len(), 1);
    }

    #[test]
    fn test_days_aggregation() {
        let conn = setup_test_db();
        conn.execute(
            "INSERT INTO note (uuid4, title, url, tags, description, comments, annotations, created_at, is_public)
             VALUES ('aaa-111', 'A', '', '', '', '', '', '2024-01-15 10:00:00', 0)",
            [],
        ).unwrap();
        conn.execute(
            "INSERT INTO note (uuid4, title, url, tags, description, comments, annotations, created_at, is_public)
             VALUES ('bbb-222', 'B', '', '', '', '', '', '2024-01-15 11:00:00', 0)",
            [],
        ).unwrap();
        conn.execute(
            "INSERT INTO note (uuid4, title, url, tags, description, comments, annotations, created_at, is_public)
             VALUES ('ccc-333', 'C', '', '', '', '', '', '2024-01-16 09:00:00', 0)",
            [],
        ).unwrap();

        let result = queries::do_select(&conn, 10, 0).unwrap();
        assert_eq!(result.days.len(), 2);
        let day_map: std::collections::HashMap<_, _> = result
            .days
            .into_iter()
            .map(|d| (d.date.to_string(), d.count))
            .collect();
        assert_eq!(*day_map.get("2024-01-15").unwrap(), 2);
        assert_eq!(*day_map.get("2024-01-16").unwrap(), 1);
    }

    #[test]
    fn test_filter_by_date_range() {
        let conn = setup_test_db();
        conn.execute(
            "INSERT INTO note (uuid4, title, url, tags, description, comments, annotations, created_at, is_public)
             VALUES ('d1', 'Old', '', 'tag', '', '', '', '2024-01-01 10:00:00', 0)",
            [],
        ).unwrap();
        conn.execute(
            "INSERT INTO note (uuid4, title, url, tags, description, comments, annotations, created_at, is_public)
             VALUES ('d2', 'Mid', '', 'tag', '', '', '', '2024-06-15 10:00:00', 0)",
            [],
        ).unwrap();
        conn.execute(
            "INSERT INTO note (uuid4, title, url, tags, description, comments, annotations, created_at, is_public)
             VALUES ('d3', 'New', '', 'tag', '', '', '', '2024-12-01 10:00:00', 0)",
            [],
        ).unwrap();

        let result = queries::do_filter(&conn, "tag", 10, 0, "2024-05-01", "2024-07-01").unwrap();
        assert_eq!(result.count, 1);
        assert_eq!(result.notes[0].title, "Mid");
    }

    #[test]
    fn test_make_fts_query() {
        let q = queries::make_fts_query("hello   world");
        // Each word should be quoted and joined with AND
        assert!(q.contains("\"hello\""));
        assert!(q.contains("\"world\""));
        assert!(q.contains("AND"));

        let q = queries::make_fts_query("  single  ");
        assert!(q.contains("\"single\""));
        assert!(!q.contains("AND"));

        // Empty query should produce empty string
        let q = queries::make_fts_query("   ");
        assert!(q.is_empty());
    }

    #[test]
    fn test_fts_where_clause() {
        let clause = queries::fts_where_clause(3);
        assert!(clause.contains("?3"));
        assert!(clause.contains("note_fts"));
        assert!(clause.contains("MATCH"));
    }

    #[test]
    fn test_sync_insert_with_valid_hex() {
        let conn = setup_test_db();
        let note = Note {
            rowid: 0,
            uuid4: "550e8400-e29b-41d4-a716-446655440000".to_string(),
            title: "Sync Note".to_string(),
            url: "https://example.com".to_string(),
            tags: "sync".to_string(),
            description: "synced".to_string(),
            comments: "".to_string(),
            annotations: "48656c6c6f".to_string(), // "Hello" in hex
            created_at: "2024-01-01 00:00:00".to_string(),
            is_public: false,
            metadata: String::new(),
            updated_at: String::new(),
            deleted: false,
        };
        sync::insert(&conn, &note).expect("sync insert should succeed");

        let result = queries::do_select(&conn, 10, 0).unwrap();
        assert_eq!(result.count, 1);
        assert_eq!(result.notes[0].title, "Sync Note");
    }

    #[test]
    fn test_sync_insert_with_invalid_hex_still_succeeds() {
        let conn = setup_test_db();
        let note = Note {
            rowid: 0,
            uuid4: "550e8400-e29b-41d4-a716-446655440001".to_string(),
            title: "Bad Hex Note".to_string(),
            url: "".to_string(),
            tags: "".to_string(),
            description: "".to_string(),
            comments: "".to_string(),
            annotations: "not-valid-hex!!!".to_string(),
            created_at: "2024-01-01 00:00:00".to_string(),
            is_public: false,
            metadata: String::new(),
            updated_at: String::new(),
            deleted: false,
        };
        // Should succeed but with empty annotations (logged warning)
        sync::insert(&conn, &note).expect("insert with bad hex should not fail");
    }

    #[test]
    fn test_process_cmd_insert() {
        let conn = setup_test_db();
        let cmd = models::Cmd::Insert(models::CmdInsert {
            title: "Cmd Insert".to_string(),
            url: "https://cmd.test".to_string(),
            tags: "cmd".to_string(),
            description: "via cmd".to_string(),
            comments: "".to_string(),
            annotations: "".to_string(),
            limit: 10,
            offset: 0,
            is_public: false,
        });
        let result = process_cmd(cmd, &conn).expect("process_cmd insert");
        let parsed: models::QueryResult = serde_json::from_str(&result).expect("valid json");
        assert_eq!(parsed.count, 1);
    }

    #[test]
    fn test_process_cmd_search() {
        let conn = setup_test_db();
        // Insert first
        let insert = models::Cmd::Insert(models::CmdInsert {
            title: "Searchable".to_string(),
            url: "https://search.test".to_string(),
            tags: "findme".to_string(),
            description: "".to_string(),
            comments: "".to_string(),
            annotations: "".to_string(),
            limit: 10,
            offset: 0,
            is_public: false,
        });
        process_cmd(insert, &conn).unwrap();

        let search = models::Cmd::Search(models::CmdSearch {
            query: "findme".to_string(),
            limit: 10,
            offset: 0,
        });
        let result = process_cmd(search, &conn).expect("process_cmd search");
        let parsed: models::QueryResult = serde_json::from_str(&result).expect("valid json");
        assert_eq!(parsed.count, 1);
    }

    #[test]
    fn test_process_cmd_delete() {
        let conn = setup_test_db();
        let insert = models::Cmd::Insert(models::CmdInsert {
            title: "To Delete".to_string(),
            url: "".to_string(),
            tags: "".to_string(),
            description: "".to_string(),
            comments: "".to_string(),
            annotations: "".to_string(),
            limit: 10,
            offset: 0,
            is_public: false,
        });
        let result = process_cmd(insert, &conn).unwrap();
        let parsed: models::QueryResult = serde_json::from_str(&result).unwrap();
        let rowid = parsed.notes[0].rowid;

        let delete = models::Cmd::Delete(models::CmdDelete {
            query: "".to_string(),
            rowid,
            limit: 10,
            offset: 0,
        });
        let result = process_cmd(delete, &conn).expect("process_cmd delete");
        let parsed: models::QueryResult = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed.count, 0);
    }

    #[test]
    fn test_migration_upgrade_on_fresh_db() {
        let conn = Connection::open_in_memory().expect("in-memory db");
        migrations::upgrade(&conn).expect("upgrade on fresh db");

        // Should have created tables and set version
        let version = migrations::get_meta_version(&conn).unwrap();
        assert_eq!(version, "0.8.0");
    }

    #[test]
    fn test_migration_upgrade_is_idempotent() {
        let conn = Connection::open_in_memory().expect("in-memory db");
        migrations::upgrade(&conn).expect("first upgrade");
        migrations::upgrade(&conn).expect("second upgrade should be idempotent");
    }

    #[test]
    fn test_sync_version_diff() {
        let conn = setup_test_db();
        queries::insert_note(&conn, "Note A", "", "", "", "", b"", false).unwrap();
        queries::insert_note(&conn, "Note B", "", "", "", "", b"", false).unwrap();

        let versions = sync::note_versions(&conn).unwrap();
        assert_eq!(versions.len(), 2);
        // Every note carries a non-empty last-write-wins token.
        assert!(versions.iter().all(|(_, ts)| !ts.is_empty()));

        // diff_to_server: a uuid the server lacks should be pushed by the client.
        let unknown = vec![(
            "unknown-uuid".to_string(),
            "00000000000000000001-x".to_string(),
        )];
        let diff = sync::diff_to_server(&conn, unknown).unwrap();
        assert_eq!(diff, vec!["unknown-uuid".to_string()]);

        // Notes the server already has at the same version are not requested.
        let diff = sync::diff_to_server(&conn, versions.clone()).unwrap();
        assert!(diff.is_empty());

        // diff_from_server: a client that knows nothing pulls everything ...
        let diff = sync::diff_from_server(&conn, vec![]).unwrap();
        assert_eq!(diff.len(), 2);

        // ... and an up-to-date client pulls nothing.
        let diff = sync::diff_from_server(&conn, versions).unwrap();
        assert!(diff.is_empty());
    }

    #[test]
    fn test_sync_lww_newer_wins_older_ignored() {
        let conn = setup_test_db();
        let original = queries::insert_note(&conn, "Original", "", "", "", "", b"", false).unwrap();

        // An incoming edit with a strictly newer token overwrites the local copy.
        let mut newer = original.clone();
        newer.title = "Edited".to_string();
        newer.updated_at = "99999999999999999999-peer".to_string();
        sync::insert(&conn, &newer).unwrap();
        assert_eq!(
            sync::get_note_by_uuid4(&conn, &original.uuid4)
                .unwrap()
                .title,
            "Edited"
        );

        // An incoming edit with an older token is ignored (no clobber).
        let mut older = original.clone();
        older.title = "Stale".to_string();
        older.updated_at = "00000000000000000001-peer".to_string();
        sync::insert(&conn, &older).unwrap();
        assert_eq!(
            sync::get_note_by_uuid4(&conn, &original.uuid4)
                .unwrap()
                .title,
            "Edited"
        );
    }

    #[test]
    fn test_delete_is_tombstoned_and_propagates() {
        let conn = setup_test_db();
        let note = queries::insert_note(&conn, "ToDelete", "", "", "", "", b"", false).unwrap();
        assert_eq!(queries::do_select(&conn, 10, 0).unwrap().count, 1);

        queries::delete_note(&conn, note.rowid).unwrap();
        // Hidden from reads ...
        assert_eq!(queries::do_select(&conn, 10, 0).unwrap().count, 0);
        // ... but retained as a tombstone that sync still advertises.
        assert_eq!(sync::note_versions(&conn).unwrap().len(), 1);
        let tomb = sync::get_note_by_uuid4(&conn, &note.uuid4).unwrap();
        assert!(tomb.deleted);

        // A peer holding the live note, then receiving the newer tombstone, hides it.
        let peer = setup_test_db();
        let mut live = tomb.clone();
        live.deleted = false;
        live.updated_at = "00000000000000000001-old".to_string();
        sync::insert(&peer, &live).unwrap();
        assert_eq!(queries::do_select(&peer, 10, 0).unwrap().count, 1);
        sync::insert(&peer, &tomb).unwrap();
        assert_eq!(queries::do_select(&peer, 10, 0).unwrap().count, 0);
    }
}

#[cfg(all(test, feature = "encryption"))]
mod encryption_tests {
    use super::encryption::*;
    use super::*;
    use rusqlite::Connection;

    /// Helper: open an in-memory encrypted database and run migrations.
    fn setup_encrypted_test_db(key: &str) -> Connection {
        let conn = Connection::open_in_memory().expect("in-memory db");
        set_encryption_key(&conn, key).expect("set key");
        migrations::upgrade(&conn).expect("upgrade");
        conn
    }

    #[test]
    fn test_set_encryption_key_and_use_db() {
        let conn = setup_encrypted_test_db("test-secret-key");

        // Should be able to insert and query after keying.
        queries::insert_note(
            &conn,
            "Encrypted Note",
            "https://example.com",
            "encrypted",
            "secret data",
            "",
            b"",
            false,
        )
        .expect("insert into encrypted db");

        let result = queries::do_select(&conn, 10, 0).expect("select");
        assert_eq!(result.count, 1);
        assert_eq!(result.notes[0].title, "Encrypted Note");
    }

    #[test]
    fn test_change_encryption_key() {
        let conn = setup_encrypted_test_db("old-key");

        queries::insert_note(&conn, "Before rekey", "", "", "", "", b"", false)
            .expect("insert before rekey");

        // Re-key the in-memory database (no-op on disk, but exercises the code path).
        change_encryption_key(&conn, "new-key").expect("rekey");

        // Data should still be accessible.
        let result = queries::do_select(&conn, 10, 0).expect("select after rekey");
        assert_eq!(result.count, 1);
    }

    #[test]
    fn test_encrypt_existing_db_dest_already_exists() {
        let dir = std::env::temp_dir();
        let source = dir.join("enc_test_source.sqlite3");
        let dest = dir.join("enc_test_dest_exists.sqlite3");

        // Create both files.
        std::fs::write(&source, "fake").unwrap();
        std::fs::write(&dest, "fake").unwrap();

        let result = encrypt_existing_db(source.to_str().unwrap(), dest.to_str().unwrap(), "key");

        // Clean up.
        std::fs::remove_file(&source).ok();
        std::fs::remove_file(&dest).ok();

        assert!(result.is_err(), "should fail when dest already exists");
    }

    #[test]
    fn test_key_with_special_characters() {
        // Single quotes in the key must be properly escaped.
        let conn = setup_encrypted_test_db("it's a \"key\" with 'quotes'");

        queries::insert_note(&conn, "Special", "", "", "", "", b"", false)
            .expect("insert with special-char key");

        let result = queries::do_select(&conn, 10, 0).expect("select");
        assert_eq!(result.count, 1);
    }
}
