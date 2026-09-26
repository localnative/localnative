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

//! Schema migrations.
//!
//! There is exactly one path to the current schema. A new database gets the
//! frozen 0.8.0 baseline ([`init_baseline_schema`]) and then runs the same
//! migrations an old database runs. To change the schema, append a
//! `(Version, fn)` entry to [`MIGRATIONS`]; never edit the baseline or an
//! existing migration. Each migration runs in its own transaction together
//! with its version bump.

use super::{
    DatabaseError, DbResult, TOMBSTONE_CONTENT, meta_get, normalize_stored_tags,
    observe_stored_tokens,
};
use crate::db::models::Note;
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
    (Version::new(0, 11, 0), migrate_0_11),
];

/// The schema version of a fully migrated database.
pub fn current_version() -> Version {
    MIGRATIONS
        .last()
        .map_or(Version::new(0, 8, 0), |(version, _)| version.clone())
}

/// Bring the database to the current schema, creating it if needed.
pub fn upgrade(conn: &Connection) -> DbResult<()> {
    if !table_exists(conn)? {
        let tx = conn.unchecked_transaction()?;
        init_baseline_schema(&tx)?;
        tx.commit()?;
    }

    let current_version = Version::parse(&get_meta_version(conn)?)?;
    for (version, migrate) in MIGRATIONS {
        if current_version < *version {
            let tx = conn.unchecked_transaction()?;
            migrate(&tx)?;
            set_meta_version(&tx, &version.to_string())?;
            tx.commit()?;
        }
    }

    // A database restored from an export carries no clock; start it past
    // every token it holds.
    if meta_get(conn, "hlc")?.is_none() {
        observe_stored_tokens(conn)?;
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
    Ok(meta_get(conn, "version")
        .ok()
        .flatten()
        .unwrap_or_else(|| "0.3.10".to_string()))
}

fn set_meta_version(conn: &Connection, version: &str) -> DbResult<()> {
    conn.execute(
        "INSERT OR REPLACE INTO meta (meta_key, meta_value) VALUES ('version', ?1)",
        rusqlite::params![version],
    )?;
    Ok(())
}

/// The schema a brand-new database starts from: exactly version 0.8.0.
/// Frozen — later changes belong in [`MIGRATIONS`].
fn init_baseline_schema(conn: &Connection) -> DbResult<()> {
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
             metadata TEXT NOT NULL DEFAULT '{}'
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

/// Whether `table` has a column named `column`.
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

/// Add last-write-wins / tombstone columns for conflict-resolving sync.
fn migrate_lww(conn: &Connection) -> DbResult<()> {
    // Guarded: databases created by pre-0.11 builds already had these columns.
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

/// Schema 0.11.0: search, tombstones, tags and sync identity.
fn migrate_0_11(conn: &Connection) -> DbResult<()> {
    // The unicode61 index never matched CJK words or substrings; searches now
    // use the trigram index (added in 0.9.0), so this one only cost writes.
    conn.execute_batch(
        "DROP TRIGGER IF EXISTS note_fts_insert;
         DROP TRIGGER IF EXISTS note_fts_delete;
         DROP TRIGGER IF EXISTS note_fts_update;
         DROP TABLE IF EXISTS note_fts;",
    )?;
    // Tombstones keep no content.
    conn.execute(
        &format!("UPDATE note SET {TOMBSTONE_CONTENT} WHERE deleted = 1"),
        [],
    )?;
    // One canonical tag format across every front-end.
    normalize_stored_tags(conn)?;
    // Tokens that aren't `<20 digits>-<node>` (e.g. written by a hostile peer
    // before tokens were validated) would outrank every real edit forever.
    conn.execute(
        "UPDATE note
         SET updated_at = printf('%020d-legacy', CAST(strftime('%s', created_at) AS INTEGER) * 1000)
         WHERE NOT (updated_at GLOB '[0-9][0-9][0-9][0-9][0-9][0-9][0-9][0-9][0-9][0-9][0-9][0-9][0-9][0-9][0-9][0-9][0-9][0-9][0-9][0-9]-?*')",
        [],
    )?;
    // Resynchronise the trigram index with the table after the rewrites above.
    conn.execute_batch("INSERT INTO note_fts_trigram(note_fts_trigram) VALUES('rebuild');")?;
    // Devices this one may sync with (see `crate::secure`).
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS peer (
             public_key TEXT PRIMARY KEY,
             node_id TEXT NOT NULL,
             name TEXT NOT NULL,
             addr TEXT NOT NULL DEFAULT '',
             paired_at TEXT NOT NULL,
             last_sync_at TEXT
         );
         CREATE INDEX IF NOT EXISTS note_created_at ON note(created_at);",
    )?;
    observe_stored_tokens(conn)?;
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
