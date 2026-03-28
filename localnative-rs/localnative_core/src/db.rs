// db.rs
pub use crate::error::{DatabaseError, DbError, DbResult, ValidationError};
pub use models::Note;
use models::Cmd;
use rusqlite::Connection;

/// Open (or create) the SQLite database at the platform-appropriate location and run any
/// pending schema migrations.
pub fn init_db() -> DbResult<Connection> {
    let db_path = utils::sqlite3_db_location()?;
    tracing::info!(db_path, "opening database");
    let conn = Connection::open(&db_path)?;

    #[cfg(target_os = "android")]
    conn.execute_batch("PRAGMA temp_store_directory = '/data/data/app.localnative/cache'")?;

    migrations::upgrade(&conn)?;

    Ok(conn)
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
    use base64::engine::general_purpose::STANDARD;
    use base64::Engine as _;
    use models::{
        CmdDelete, CmdFilter, CmdInsert, CmdSearch, CmdSelect, CmdSyncViaAttach, Note, QueryResult,
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

        conn.execute(
            "INSERT INTO note (uuid4, title, url, tags, description, comments, annotations, created_at, is_public)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            rusqlite::params![uuid4, title, url, tags, description, comments, annotations, created_at, is_public],
        )?;

        let note = conn.query_row(
            "SELECT rowid, uuid4, title, url, tags, description, comments, hex(annotations) as annotations, created_at, is_public FROM note WHERE uuid4 = ?1",
            rusqlite::params![uuid4],
            map_note,
        )?;

        Ok(note)
    }

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

        conn.execute(
            "INSERT INTO note (uuid4, title, url, tags, description, comments, annotations, created_at, is_public)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            rusqlite::params![uuid4, title, url, tags, description, comments, annotations, created_at, is_public],
        )?;

        let note = conn.query_row(
            "SELECT rowid, uuid4, title, url, tags, description, comments, hex(annotations) as annotations, created_at, is_public FROM note WHERE uuid4 = ?1",
            rusqlite::params![uuid4],
            map_note,
        )?;

        Ok(note)
    }

    pub fn delete_note(conn: &Connection, rowid: i64) -> DbResult<()> {
        conn.execute(
            "DELETE FROM note WHERE rowid = ?1",
            rusqlite::params![rowid],
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

    pub fn sync_via_attach(conn: &Connection, uri: &str) -> DbResult<()> {
        validate_sync_file_path(uri)?;
        conn.execute("ATTACH ?1 AS other", rusqlite::params![uri])?;
        conn.execute_batch(
            "INSERT INTO main.note (uuid4, title, url, tags, description, comments, annotations, created_at, is_public)
            SELECT uuid4, title, url, tags, description, comments, annotations, created_at, is_public
            FROM other.note
            WHERE NOT EXISTS (
                SELECT 1 FROM main.note
                WHERE main.note.uuid4 = other.note.uuid4
            ) ORDER BY created_at;

            INSERT INTO other.note (uuid4, title, url, tags, description, comments, annotations, created_at, is_public)
            SELECT uuid4, title, url, tags, description, comments, annotations, created_at, is_public
            FROM main.note
            WHERE NOT EXISTS (
                SELECT 1 FROM other.note
                WHERE other.note.uuid4 = main.note.uuid4
            ) ORDER BY created_at;

            DETACH DATABASE other;",
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
        let count: i64 = conn.query_row("SELECT COUNT(1) FROM note", [], |row| row.get(0))?;
        Ok(u32::try_from(count).unwrap_or(u32::MAX))
    }

    fn select(conn: &Connection, limit: u32, offset: u32) -> DbResult<Vec<Note>> {
        let mut stmt = conn.prepare(
            "SELECT rowid, uuid4, title, url, tags, description, comments,
             hex(annotations) as annotations, created_at, is_public
             FROM note
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
        let mut stmt = conn.prepare("SELECT tags FROM note")?;
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
            WHERE {}",
            fts_where_clause(1)
        );

        let count: i64 =
            conn.query_row(&sql, rusqlite::params![fts_query], |row| row.get(0))?;
        Ok(u32::try_from(count).unwrap_or(u32::MAX))
    }

    fn search(conn: &Connection, query: &str, limit: u32, offset: u32) -> DbResult<Vec<Note>> {
        if query.is_empty() {
            return select(conn, limit, offset);
        }

        let fts_query = make_fts_query(query);
        let sql = format!(
            "SELECT rowid, uuid4, title, url, tags, description, comments,
             hex(annotations) as annotations, created_at, is_public
             FROM note
             WHERE {}
             ORDER BY created_at DESC
             LIMIT ?1 OFFSET ?2",
            fts_where_clause(3),
        );

        let mut stmt = conn.prepare(&sql)?;
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
            WHERE {}
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
            WHERE {}",
            fts_where_clause(1)
        );

        let mut tag_count_map = HashMap::new();
        let mut stmt = conn.prepare(&sql)?;
        let tags_iter = stmt.query_map(rusqlite::params![fts_query], |row| {
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

    fn filter_count(conn: &Connection, query: &str, from: &str, to: &str) -> DbResult<u32> {
        let fts_query = make_fts_query(query);
        if fts_query.is_empty() {
            return select_count(conn);
        }

        let sql = format!(
            "SELECT COUNT(1)
            FROM note
            WHERE substr(created_at, 1, 10) >= ?1
            AND substr(created_at, 1, 10) <= ?2
            AND {}",
            fts_where_clause(3)
        );

        let count: i64 = conn.query_row(
            &sql,
            rusqlite::params![from, to, fts_query],
            |row| row.get(0),
        )?;
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
            return select(conn, limit, offset);
        }

        let sql = format!(
            "SELECT rowid, uuid4, title, url, tags, description, comments,
             hex(annotations) as annotations, created_at, is_public
             FROM note
             WHERE substr(created_at, 1, 10) >= ?1
             AND substr(created_at, 1, 10) <= ?2
             AND {}
             ORDER BY created_at DESC
             LIMIT ?3 OFFSET ?4",
            fts_where_clause(5)
        );

        let mut stmt = conn.prepare(&sql)?;
        let notes = stmt
            .query_map(rusqlite::params![from, to, limit, offset, fts_query], map_note)?
            .collect::<Result<Vec<_>, rusqlite::Error>>()?;
        Ok(notes)
    }

    fn filter_by_tag(conn: &Connection, query: &str, from: &str, to: &str) -> DbResult<Vec<Tags>> {
        let fts_query = make_fts_query(query);

        if fts_query.is_empty() {
            return select_by_tag(conn);
        }

        let sql = format!(
            "SELECT tags
            FROM note
            WHERE substr(created_at, 1, 10) >= ?1
            AND substr(created_at, 1, 10) <= ?2
            AND {}",
            fts_where_clause(3)
        );

        let mut tag_count_map = HashMap::new();
        let mut stmt = conn.prepare(&sql)?;
        let tags_iter = stmt.query_map(
            rusqlite::params![from, to, fts_query],
            |row| row.get::<_, String>(0),
        )?;

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
             hex(annotations) as annotations, created_at, is_public
             FROM note
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
        let sql = format!(
            "SELECT rowid, uuid4, title, url, tags, description, comments,
             hex(annotations) as annotations, created_at, is_public
             FROM note
             WHERE {}
             ORDER BY created_at DESC",
            fts_where_clause(1),
        );

        let mut stmt = conn.prepare(&sql)?;
        let notes = stmt
            .query_map(rusqlite::params![fts_query], map_note)?
            .collect::<Result<Vec<_>, rusqlite::Error>>()?;
        Ok(notes)
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
        format!(
            "rowid IN (SELECT rowid FROM note_fts WHERE note_fts MATCH ?{fts_param})"
        )
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
                 is_public BOOLEAN NOT NULL DEFAULT 0
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
             INSERT INTO meta (meta_key, meta_value) VALUES ('version', '0.7.0');",
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
    use std::collections::HashSet;

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
            "SELECT rowid, uuid4, title, url, tags, description, comments, hex(annotations) as annotations, created_at, is_public FROM note WHERE uuid4 = ?1",
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
                })
            },
        )
        .map_err(DbError::from)
    }

    pub fn next_uuid4_candidates(conn: &Connection) -> DbResult<Vec<String>> {
        let mut stmt = conn.prepare("SELECT uuid4 FROM note ORDER BY rowid")?;
        let uuids = stmt
            .query_map([], |row| row.get(0))?
            .collect::<Result<Vec<String>, rusqlite::Error>>()?;
        Ok(uuids)
    }

    pub fn diff_uuid4_to_server(
        conn: &Connection,
        candidates: Vec<String>,
    ) -> DbResult<Vec<String>> {
        let mut r = Vec::new();
        for uuid4 in candidates {
            let exists: bool = conn.query_row(
                "SELECT EXISTS(SELECT 1 FROM note WHERE uuid4 = ?1)",
                rusqlite::params![uuid4],
                |row| row.get(0),
            )?;
            if !exists {
                r.push(uuid4);
            }
        }
        Ok(r)
    }

    pub fn diff_uuid4_from_server(
        conn: &Connection,
        candidates: Vec<String>,
    ) -> DbResult<Vec<String>> {
        let candidates: HashSet<_> = candidates.iter().collect();
        let mut r = Vec::new();
        let mut stmt = conn.prepare("SELECT uuid4 FROM note")?;
        let uuid4s: Vec<String> = stmt
            .query_map([], |row| row.get(0))?
            .collect::<Result<_, rusqlite::Error>>()?;
        for uuid4 in uuid4s {
            if !candidates.contains(&uuid4) {
                r.push(uuid4);
            }
        }
        Ok(r)
    }

    pub fn insert(conn: &Connection, note: &Note) -> DbResult<()> {
        let annotations_blob = hex::decode(&note.annotations).unwrap_or_else(|e| {
            tracing::warn!(uuid4 = note.uuid4, %e, "failed to decode hex annotations");
            Vec::new()
        });
        conn.execute(
            "INSERT INTO note (uuid4, title, url, tags, description, comments, annotations, created_at, is_public)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            rusqlite::params![
                note.uuid4,
                note.title,
                note.url,
                note.tags,
                note.description,
                note.comments,
                annotations_blob,
                note.created_at,
                note.is_public
            ],
        )?;
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
                 is_public BOOLEAN NOT NULL DEFAULT 0
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
             INSERT INTO meta (meta_key, meta_value) VALUES ('version', '0.7.0');",
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
        assert_eq!(version, "0.7.0");
    }

    #[test]
    fn test_migration_upgrade_is_idempotent() {
        let conn = Connection::open_in_memory().expect("in-memory db");
        migrations::upgrade(&conn).expect("first upgrade");
        migrations::upgrade(&conn).expect("second upgrade should be idempotent");
    }

    #[test]
    fn test_sync_uuid4_operations() {
        let conn = setup_test_db();
        queries::insert_note(&conn, "Note A", "", "", "", "", b"", false).unwrap();
        queries::insert_note(&conn, "Note B", "", "", "", "", b"", false).unwrap();

        let candidates = sync::next_uuid4_candidates(&conn).unwrap();
        assert_eq!(candidates.len(), 2);

        // diff_uuid4_to_server: given candidates, return those NOT in our db
        let unknown = vec!["unknown-uuid".to_string()];
        let diff = sync::diff_uuid4_to_server(&conn, unknown).unwrap();
        assert_eq!(diff.len(), 1);
        assert_eq!(diff[0], "unknown-uuid");

        // Known uuids should not appear in diff
        let diff = sync::diff_uuid4_to_server(&conn, candidates.clone()).unwrap();
        assert!(diff.is_empty());

        // diff_uuid4_from_server: return our uuids NOT in candidates
        let diff = sync::diff_uuid4_from_server(&conn, vec![]).unwrap();
        assert_eq!(diff.len(), 2);

        let diff = sync::diff_uuid4_from_server(&conn, candidates).unwrap();
        assert!(diff.is_empty());
    }
}
