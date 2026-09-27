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

//! Reads and writes on the `note` table: inserts, soft deletes, search with
//! facets, and whole-file merge, import and export.

use super::models::{
    CmdDelete, CmdExportDb, CmdFilter, CmdImportDb, CmdInsert, CmdSearch, CmdSelect,
    CmdSyncViaAttach, CmdUpdate, Day, Note, QueryResult, Tags,
};
use super::{
    DbResult, TOMBSTONE_CONTENT, ValidationError, next_update_token, normalize_stored_tags,
    normalize_tags, observe_stored_tokens,
};
use base64::Engine as _;
use base64::engine::general_purpose::STANDARD;
use chrono::NaiveDate;
use rusqlite::types::{Value, ValueRef};
use rusqlite::{Connection, OptionalExtension};
use std::collections::HashMap;
use std::path::Path;
use uuid::Uuid;

// ── Command handlers ──────────────────────────────────────────────────────

impl CmdFilter {
    pub fn process(&self, conn: &Connection) -> DbResult<QueryResult> {
        do_filter(
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
        insert_note(
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

    /// Insert a note whose annotations are a base64 `data:` URL (a screenshot).
    pub fn process_image(&self, conn: &Connection) -> DbResult<Note> {
        let data64 = self
            .annotations
            .split_once(";base64,")
            .map_or(self.annotations.as_str(), |(_, data)| data);
        let decoded = STANDARD.decode(data64.trim())?;
        insert_note(
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
        delete_note(conn, self.rowid)
    }
}

impl CmdUpdate {
    pub fn process(&self, conn: &Connection) -> DbResult<Note> {
        let annotations = self.annotations.as_deref().map(|text| {
            // Same forms insert accepts: plain UTF-8, or a base64 `data:` URL.
            match text.split_once(";base64,") {
                Some((_, data)) => STANDARD
                    .decode(data.trim())
                    .unwrap_or_else(|_| text.as_bytes().to_vec()),
                None => text.as_bytes().to_vec(),
            }
        });
        update_note(
            conn,
            &self.uuid4,
            self.title.as_deref(),
            self.url.as_deref(),
            self.tags.as_deref(),
            self.description.as_deref(),
            self.comments.as_deref(),
            annotations.as_deref(),
        )
    }
}

impl CmdSyncViaAttach {
    pub fn process(&self, conn: &Connection) -> DbResult<()> {
        sync_via_attach(conn, &self.uri)
    }
}

impl CmdExportDb {
    pub fn process(&self, conn: &Connection) -> DbResult<()> {
        export_db(conn, &self.dest)
    }
}

impl CmdImportDb {
    pub fn process(&self, conn: &Connection) -> DbResult<usize> {
        import_db(conn, &self.src)
    }
}

impl CmdSelect {
    pub fn process(&self, conn: &Connection) -> DbResult<QueryResult> {
        do_select(conn, self.limit, self.offset)
    }
}

impl CmdSearch {
    pub fn process(&self, conn: &Connection) -> DbResult<QueryResult> {
        do_search(conn, &self.query, self.limit, self.offset)
    }
}

// ── Row mapping ───────────────────────────────────────────────────────────

/// Columns every read query projects, in the order [`map_note`] expects.
const NOTE_COLUMNS: &str = "note.rowid, note.uuid4, note.title, note.url, note.tags, \
     note.description, note.comments, note.annotations, note.created_at, note.is_public, \
     note.metadata";

/// Present stored annotations the way the write side accepts them: UTF-8 as
/// text, anything else (screenshots) as a base64 `data:` URL.
pub(crate) fn annotations_for_api(value: ValueRef<'_>) -> String {
    match value {
        ValueRef::Text(bytes) => String::from_utf8_lossy(bytes).into_owned(),
        ValueRef::Blob(bytes) => match std::str::from_utf8(bytes) {
            Ok(text) => text.to_string(),
            Err(_) => {
                let mime = if bytes.starts_with(b"\x89PNG\r\n\x1a\n") {
                    "image/png"
                } else {
                    "application/octet-stream"
                };
                format!("data:{mime};base64,{}", STANDARD.encode(bytes))
            }
        },
        ValueRef::Null => String::new(),
        ValueRef::Integer(i) => i.to_string(),
        ValueRef::Real(f) => f.to_string(),
    }
}

fn map_note(row: &rusqlite::Row) -> rusqlite::Result<Note> {
    Ok(Note {
        rowid: row.get("rowid")?,
        uuid4: row.get("uuid4")?,
        title: row.get("title")?,
        url: row.get("url")?,
        tags: row.get("tags")?,
        description: row.get("description")?,
        comments: row.get("comments")?,
        annotations: annotations_for_api(row.get_ref("annotations")?),
        created_at: row.get("created_at")?,
        is_public: row.get("is_public")?,
        metadata: row.get("metadata")?,
        // Read paths don't project these; tombstones are never returned.
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

fn clamp_count(count: i64) -> u32 {
    u32::try_from(count.max(0)).unwrap_or(u32::MAX)
}

// ── Writes ────────────────────────────────────────────────────────────────

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
    let created_at = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
    insert_note_with_timestamp(
        conn,
        title,
        url,
        tags,
        description,
        comments,
        annotations,
        is_public,
        &created_at,
    )
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
    let updated_at = next_update_token(conn, None)?;

    conn.execute(
        "INSERT INTO note (uuid4, title, url, tags, description, comments, annotations, created_at, is_public, metadata, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, '{}', ?10)",
        rusqlite::params![
            uuid4,
            title,
            url,
            normalize_tags(tags),
            description,
            comments,
            annotations,
            created_at,
            is_public,
            updated_at
        ],
    )?;

    let note = conn.query_row(
        &format!("SELECT {NOTE_COLUMNS} FROM note WHERE uuid4 = ?1"),
        rusqlite::params![uuid4],
        map_note,
    )?;
    Ok(note)
}

/// Edit an existing note (identified by `uuid4`), leaving absent fields
/// and `created_at`/`is_public` untouched. The row gets a fresh
/// last-write-wins token — strictly newer than the version it replaces — so
/// the edit outranks the older copy on every peer. Updating a deleted note is
/// an error: un-deleting through the edit path would silently resurrect it
/// with a token the tombstone should keep beating.
#[allow(clippy::too_many_arguments)]
pub fn update_note(
    conn: &Connection,
    uuid4: &str,
    title: Option<&str>,
    url: Option<&str>,
    tags: Option<&str>,
    description: Option<&str>,
    comments: Option<&str>,
    annotations: Option<&[u8]>,
) -> DbResult<Note> {
    use rusqlite::OptionalExtension;
    let current: Option<(i64, String)> = conn
        .query_row(
            "SELECT rowid, updated_at FROM note WHERE uuid4 = ?1 AND deleted = 0",
            rusqlite::params![uuid4],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .optional()?;
    let Some((rowid, previous_token)) = current else {
        return Err(ValidationError::Other(format!("no live note with uuid4 {uuid4}")).into());
    };

    let updated_at = next_update_token(conn, Some(&previous_token))?;
    conn.execute(
        "UPDATE note SET
             title = COALESCE(?2, title),
             url = COALESCE(?3, url),
             tags = COALESCE(?4, tags),
             description = COALESCE(?5, description),
             comments = COALESCE(?6, comments),
             annotations = COALESCE(?7, annotations),
             updated_at = ?8
         WHERE rowid = ?1",
        rusqlite::params![
            rowid,
            title,
            url,
            tags.map(normalize_tags),
            description,
            comments,
            annotations,
            updated_at
        ],
    )?;

    let note = conn.query_row(
        &format!("SELECT {NOTE_COLUMNS} FROM note WHERE uuid4 = ?1"),
        rusqlite::params![uuid4],
        map_note,
    )?;
    Ok(note)
}

/// Soft-delete a note: turn it into a tombstone. The row keeps only its UUID,
/// `created_at` and a token newer than the version it replaces, so the delete
/// wins everywhere it propagates; every content column is emptied, so a
/// deleted note's text neither stays on disk nor travels to peers.
pub fn delete_note(conn: &Connection, rowid: i64) -> DbResult<()> {
    let current: Option<String> = conn
        .query_row(
            "SELECT updated_at FROM note WHERE rowid = ?1 AND deleted = 0",
            rusqlite::params![rowid],
            |row| row.get(0),
        )
        .optional()?;
    let Some(current) = current else {
        return Ok(());
    };
    let updated_at = next_update_token(conn, Some(&current))?;
    conn.execute(
        &format!(
            "UPDATE note SET deleted = 1, updated_at = ?2, {TOMBSTONE_CONTENT} WHERE rowid = ?1"
        ),
        rusqlite::params![rowid, updated_at],
    )?;
    Ok(())
}

// ── Search ────────────────────────────────────────────────────────────────

/// A user query, split by how each whitespace-separated term can be matched.
///
/// Terms of three or more characters go to the trigram full-text index, which
/// matches substrings in any script (学习Rust, script → JavaScript). A trigram
/// index cannot match shorter terms (学习, AI, go), so those fall back to a
/// LIKE scan. Every term must match.
struct TextQuery {
    fts: Option<String>,
    likes: Vec<String>,
}

impl TextQuery {
    fn parse(query: &str) -> Self {
        let mut fts_terms = Vec::new();
        let mut likes = Vec::new();
        for term in query.split_whitespace() {
            if term.chars().count() >= 3 {
                // Quote every term so FTS5 operators in user input stay literal.
                fts_terms.push(format!("\"{}\"", term.replace('"', "\"\"")));
            } else {
                likes.push(format!("%{}%", escape_like(term)));
            }
        }
        Self {
            fts: (!fts_terms.is_empty()).then(|| fts_terms.join(" AND ")),
            likes,
        }
    }
}

fn escape_like(term: &str) -> String {
    let mut escaped = String::with_capacity(term.len());
    for c in term.chars() {
        if matches!(c, '\\' | '%' | '_') {
            escaped.push('\\');
        }
        escaped.push(c);
    }
    escaped
}

/// `WHERE` conditions over `note` selecting live notes that match a text query
/// and an optional inclusive day range (`YYYY-MM-DD`).
struct NoteFilter {
    sql: String,
    params: Vec<Value>,
}

impl NoteFilter {
    /// With `fts_in_join`, the caller joins `note_fts_trigram` itself (to rank
    /// by bm25) and supplies the MATCH parameter first.
    fn new(text: &TextQuery, range: Option<(&str, &str)>, fts_in_join: bool) -> Self {
        let mut sql = String::from("note.deleted = 0");
        let mut params = Vec::new();
        if let Some((from, to)) = range {
            sql.push_str(
                " AND substr(note.created_at, 1, 10) >= ? AND substr(note.created_at, 1, 10) <= ?",
            );
            params.push(Value::Text(from.to_string()));
            params.push(Value::Text(to.to_string()));
        }
        if let Some(fts) = &text.fts
            && !fts_in_join
        {
            sql.push_str(
                " AND note.rowid IN (SELECT rowid FROM note_fts_trigram WHERE note_fts_trigram MATCH ?)",
            );
            params.push(Value::Text(fts.clone()));
        }
        for like in &text.likes {
            sql.push_str(
                " AND (note.title LIKE ? ESCAPE '\\' OR note.url LIKE ? ESCAPE '\\' \
                 OR note.tags LIKE ? ESCAPE '\\' OR note.description LIKE ? ESCAPE '\\')",
            );
            params.extend(std::iter::repeat_n(Value::Text(like.clone()), 4));
        }
        Self { sql, params }
    }
}

fn count_notes(conn: &Connection, text: &TextQuery, range: Option<(&str, &str)>) -> DbResult<u32> {
    let filter = NoteFilter::new(text, range, false);
    let count: i64 = conn.query_row(
        &format!("SELECT COUNT(1) FROM note WHERE {}", filter.sql),
        rusqlite::params_from_iter(filter.params),
        |row| row.get(0),
    )?;
    Ok(clamp_count(count))
}

/// One page of matching notes: best match first for text queries, newest
/// first otherwise. `limit = None` returns every match.
fn note_page(
    conn: &Connection,
    text: &TextQuery,
    range: Option<(&str, &str)>,
    limit: Option<u32>,
    offset: u32,
) -> DbResult<Vec<Note>> {
    let limit = limit.map_or(-1, i64::from);
    let (sql, mut params) = match &text.fts {
        Some(fts) => {
            let filter = NoteFilter::new(text, range, true);
            let sql = format!(
                "SELECT {NOTE_COLUMNS} FROM note
                 JOIN note_fts_trigram ON note_fts_trigram.rowid = note.rowid
                 WHERE note_fts_trigram MATCH ? AND {}
                 ORDER BY bm25(note_fts_trigram, 10.0, 5.0, 3.0, 1.0), note.created_at DESC
                 LIMIT ? OFFSET ?",
                filter.sql
            );
            let mut params = vec![Value::Text(fts.clone())];
            params.extend(filter.params);
            (sql, params)
        }
        None => {
            let filter = NoteFilter::new(text, range, false);
            let sql = format!(
                "SELECT {NOTE_COLUMNS} FROM note WHERE {}
                 ORDER BY note.created_at DESC LIMIT ? OFFSET ?",
                filter.sql
            );
            (sql, filter.params)
        }
    };
    params.push(Value::Integer(limit));
    params.push(Value::Integer(i64::from(offset)));
    let mut stmt = conn.prepare(&sql)?;
    let notes = stmt
        .query_map(rusqlite::params_from_iter(params), map_note)?
        .collect::<Result<Vec<_>, rusqlite::Error>>()?;
    Ok(notes)
}

/// Matching notes per day, oldest day first.
fn day_histogram(conn: &Connection, text: &TextQuery) -> DbResult<Vec<Day>> {
    let filter = NoteFilter::new(text, None, false);
    let sql = format!(
        "SELECT DATE(substr(note.created_at, 1, 10)) AS date, COUNT(1) AS count
         FROM note WHERE {} AND DATE(substr(note.created_at, 1, 10)) IS NOT NULL
         GROUP BY date ORDER BY date",
        filter.sql
    );
    let mut stmt = conn.prepare(&sql)?;
    let days = stmt
        .query_map(rusqlite::params_from_iter(filter.params), map_day)?
        .collect::<Result<Vec<_>, rusqlite::Error>>()?;
    Ok(days)
}

/// Tag counts over the matching notes, case-insensitive.
fn tag_counts(
    conn: &Connection,
    text: &TextQuery,
    range: Option<(&str, &str)>,
) -> DbResult<Vec<Tags>> {
    let filter = NoteFilter::new(text, range, false);
    let mut stmt = conn.prepare(&format!(
        "SELECT note.tags FROM note WHERE {} AND note.tags <> ''",
        filter.sql
    ))?;
    let mut counts: HashMap<String, i64> = HashMap::new();
    let rows = stmt.query_map(rusqlite::params_from_iter(filter.params), |row| {
        row.get::<_, String>(0)
    })?;
    for tags in rows {
        for tag in tags?.split(',').map(str::trim).filter(|t| !t.is_empty()) {
            *counts.entry(tag.to_lowercase()).or_insert(0) += 1;
        }
    }
    Ok(counts
        .into_iter()
        .map(|(tag, count)| Tags { tag, count })
        .collect())
}

fn run_query(
    conn: &Connection,
    query: &str,
    range: Option<(&str, &str)>,
    limit: u32,
    offset: u32,
) -> DbResult<QueryResult> {
    let text = TextQuery::parse(query);
    Ok(QueryResult {
        count: count_notes(conn, &text, range)?,
        notes: note_page(conn, &text, range, Some(limit), offset)?,
        // The histogram always spans every day, so a selected range can be widened again.
        days: day_histogram(conn, &text)?,
        tags: tag_counts(conn, &text, range)?,
    })
}

/// Newest notes plus the day histogram and tag counts.
pub fn do_select(conn: &Connection, limit: u32, offset: u32) -> DbResult<QueryResult> {
    run_query(conn, "", None, limit, offset)
}

/// Notes matching `query`; an empty query lists every note, newest first.
pub fn do_search(conn: &Connection, query: &str, limit: u32, offset: u32) -> DbResult<QueryResult> {
    run_query(conn, query, None, limit, offset)
}

/// Notes matching `query` created between `from` and `to` (inclusive,
/// `YYYY-MM-DD`). The day histogram ignores the range.
pub fn do_filter(
    conn: &Connection,
    query: &str,
    limit: u32,
    offset: u32,
    from: &str,
    to: &str,
) -> DbResult<QueryResult> {
    run_query(conn, query, Some((from, to)), limit, offset)
}

/// Every live note, newest first.
pub fn select_all(conn: &Connection) -> DbResult<Vec<Note>> {
    search_all(conn, "")
}

/// Every note matching `query`, without pagination.
pub fn search_all(conn: &Connection, query: &str) -> DbResult<Vec<Note>> {
    note_page(conn, &TextQuery::parse(query), None, None, 0)
}

// ── Whole-file operations ─────────────────────────────────────────────────

fn validate_sync_file_path(uri: &str) -> DbResult<()> {
    let path = Path::new(uri);

    // Ensure the path is absolute to prevent relative path traversal
    if !path.is_absolute() {
        return Err(
            ValidationError::InvalidPath("Sync file path must be absolute".to_string()).into(),
        );
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

/// SQL producing a legacy row's last-write-wins token from its `created_at`.
const LEGACY_TOKEN_SQL: &str =
    "printf('%020d-legacy', CAST(strftime('%s', created_at) AS INTEGER) * 1000)";

/// Bring `schema.note` up to the last-write-wins schema, adding and
/// back-filling the columns when an attached peer database predates 0.10.0.
/// Only used for two-way file sync, where changing the other file is the point.
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
        "UPDATE {schema}.note SET updated_at = {LEGACY_TOKEN_SQL}
         WHERE updated_at IS NULL OR updated_at = '';"
    ))?;
    Ok(())
}

/// Empty the content of every tombstone in `schema.note`.
fn strip_tombstones(conn: &Connection, schema: &str) -> DbResult<()> {
    conn.execute_batch(&format!(
        "UPDATE {schema}.note SET {TOMBSTONE_CONTENT}
         WHERE deleted = 1 AND (title <> '' OR url <> '' OR tags <> '' OR description <> ''
               OR comments <> '' OR length(annotations) > 0 OR metadata <> '{{}}');"
    ))?;
    Ok(())
}

const MERGE_COLUMNS: &str = "uuid4, title, url, tags, description, comments, annotations, created_at, is_public, metadata, updated_at, deleted";

/// `INSERT … ON CONFLICT` from `source` into `target`, newest token wins.
fn merge_sql(target: &str, source_select: &str) -> String {
    format!(
        "INSERT INTO {target}.note AS t ({MERGE_COLUMNS})
         {source_select}
         ON CONFLICT(uuid4) DO UPDATE SET
             title = excluded.title, url = excluded.url, tags = excluded.tags,
             description = excluded.description, comments = excluded.comments,
             annotations = excluded.annotations, created_at = excluded.created_at,
             is_public = excluded.is_public, metadata = excluded.metadata,
             updated_at = excluded.updated_at, deleted = excluded.deleted
         WHERE excluded.updated_at > t.updated_at"
    )
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

fn sync_attached(conn: &Connection) -> DbResult<()> {
    let tx = conn.unchecked_transaction()?;
    ensure_lww_columns(&tx, "main")?;
    ensure_lww_columns(&tx, "other")?;
    // `WHERE true` is required for SQLite to parse ON CONFLICT after a SELECT.
    tx.execute(
        &merge_sql(
            "main",
            &format!("SELECT {MERGE_COLUMNS} FROM other.note WHERE true"),
        ),
        [],
    )?;
    tx.execute(
        &merge_sql(
            "other",
            &format!("SELECT {MERGE_COLUMNS} FROM main.note WHERE true"),
        ),
        [],
    )?;
    strip_tombstones(&tx, "main")?;
    strip_tombstones(&tx, "other")?;
    normalize_stored_tags(&tx)?;
    observe_stored_tokens(&tx)?;
    tx.commit()?;
    Ok(())
}

/// A `file:` URI that opens `path` read-only, so an import can never modify
/// the file it reads from.
fn read_only_uri(path: &str) -> String {
    let mut uri = String::from("file:");
    for byte in path.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'/' | b'-' | b'_' | b'.' | b'~') {
            uri.push(char::from(byte));
        } else {
            uri.push_str(&format!("%{byte:02X}"));
        }
    }
    uri.push_str("?mode=ro");
    uri
}

/// Export a clean, single-file copy of the database to `dest` using
/// `VACUUM INTO`. The copy is compacted and contains all committed data —
/// including transactions still resident in the WAL — with no `-wal`/`-shm`
/// sidecars, so it is safe to read or share as a standalone file. Any
/// existing file at `dest` is replaced (VACUUM INTO refuses to overwrite).
///
/// The copy carries no device identity (node id, clock, sync keys, paired
/// peers): restored elsewhere it behaves as a new device instead of cloning
/// this one.
///
/// Note: with the optional SQLCipher `encryption` feature, the exported
/// copy is written **unencrypted** (VACUUM INTO does not carry the key).
pub fn export_db(conn: &Connection, dest: &str) -> DbResult<()> {
    if dest.trim().is_empty() {
        return Err(
            ValidationError::InvalidPath("Export destination path is empty".to_string()).into(),
        );
    }
    let dest_path = Path::new(dest);
    // VACUUM INTO errors if the target exists; make re-export idempotent.
    if dest_path.exists() {
        std::fs::remove_file(dest_path)?;
    }
    if let Some(parent) = dest_path.parent()
        && !parent.as_os_str().is_empty()
    {
        std::fs::create_dir_all(parent)?;
    }
    conn.execute("VACUUM INTO ?1", rusqlite::params![dest])?;
    let copy = Connection::open(dest_path)?;
    copy.execute_batch(
        "DELETE FROM meta WHERE meta_key IN ('node_id', 'hlc', 'sync_private_key', 'sync_public_key');
         DELETE FROM peer;",
    )?;
    Ok(())
}

/// Import notes from another database file at `src` into the current
/// database with a **one-way** last-write-wins merge: rows from `src` are
/// applied to the local DB when their `updated_at` is newer, and tombstones
/// propagate. The source is attached read-only and never modified, so it is
/// safe to import from a read-only backup; files older than schema 0.10.0 get
/// their tokens computed on the fly. Returns the number of local rows inserted
/// or updated.
pub fn import_db(conn: &Connection, src: &str) -> DbResult<usize> {
    validate_sync_file_path(src)?;
    conn.execute("ATTACH ?1 AS other", rusqlite::params![read_only_uri(src)])?;
    let result = import_attached(conn);
    // Always detach, even on failure, so a later import can re-attach.
    let _ = conn.execute_batch("DETACH DATABASE other;");
    result
}

fn import_attached(conn: &Connection) -> DbResult<usize> {
    let updated_at = if schema_column_exists(conn, "other", "updated_at")? {
        format!("CASE WHEN updated_at = '' THEN {LEGACY_TOKEN_SQL} ELSE updated_at END")
    } else {
        LEGACY_TOKEN_SQL.to_string()
    };
    let deleted = if schema_column_exists(conn, "other", "deleted")? {
        "deleted"
    } else {
        "0"
    };
    let metadata = if schema_column_exists(conn, "other", "metadata")? {
        "metadata"
    } else {
        "'{}'"
    };
    let select = format!(
        "SELECT uuid4, title, url, tags, description, comments, annotations, created_at,
                is_public, {metadata}, {updated_at}, {deleted}
         FROM other.note WHERE true"
    );
    let tx = conn.unchecked_transaction()?;
    let merged = tx.execute(&merge_sql("main", &select), [])?;
    strip_tombstones(&tx, "main")?;
    normalize_stored_tags(&tx)?;
    observe_stored_tokens(&tx)?;
    tx.commit()?;
    Ok(merged)
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

    #[test]
    fn test_text_query_routes_terms_by_length() {
        let q = TextQuery::parse("  学习Rust  go 编程  ");
        assert_eq!(q.fts.as_deref(), Some("\"学习Rust\""));
        assert_eq!(q.likes, vec!["%go%".to_string(), "%编程%".to_string()]);

        let q = TextQuery::parse("say \"hi\" 100%");
        assert_eq!(
            q.fts.as_deref(),
            Some("\"say\" AND \"\"\"hi\"\"\" AND \"100%\"")
        );
        assert!(q.likes.is_empty());

        let q = TextQuery::parse("a_ %");
        assert_eq!(q.likes, vec!["%a\\_%".to_string(), "%\\%%".to_string()]);

        let q = TextQuery::parse("   ");
        assert!(q.fts.is_none() && q.likes.is_empty());
    }

    #[test]
    fn test_read_only_uri_escapes_path() {
        assert_eq!(
            read_only_uri("/tmp/my backup#1.sqlite3"),
            "file:/tmp/my%20backup%231.sqlite3?mode=ro"
        );
    }

    #[test]
    fn test_annotations_for_api() {
        assert_eq!(annotations_for_api(ValueRef::Blob(b"hello")), "hello");
        assert_eq!(annotations_for_api(ValueRef::Text(b"text")), "text");
        let png = b"\x89PNG\r\n\x1a\n\x00\xff";
        assert!(annotations_for_api(ValueRef::Blob(png)).starts_with("data:image/png;base64,"));
        assert!(
            annotations_for_api(ValueRef::Blob(b"\xff\xfe"))
                .starts_with("data:application/octet-stream;base64,")
        );
    }
}
