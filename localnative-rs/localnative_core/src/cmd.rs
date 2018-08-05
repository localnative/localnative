extern crate rusqlite;
extern crate serde;
extern crate serde_json;
extern crate time;

use self::rusqlite::Connection;
use super::Note;

pub fn count(conn: &Connection, tbl: &str) -> i64 {
    let mut stmt = conn
        .prepare(&format!("select count(1) as cnt from {}", tbl))
        .unwrap();
    let rs = stmt.query_row(&[], |row| row.get(0)).unwrap();
    rs
}

pub fn select(conn: &Connection, sql: &str) -> String {
    let mut stmt = conn.prepare(sql).unwrap();
    let note_iter =
        stmt.query_map(&[], |row| Note {
            rowid: row.get(0),
            title: row.get(1),
            url: row.get(2),
            tags: row.get(3),
            description: row.get(4),
            comments: row.get(5),
            annotations: row.get(6),
            created_at: row.get(7),
        }).unwrap();

    let mut j = "[ ".to_owned();
    for note in note_iter {
        let note = note.unwrap();
        //eprintln!("Found note {:?}", note);
        j.push_str(&serde_json::to_string(&note).unwrap());
        j.push_str(",");
    }
    j.pop();
    j.push_str("]");
    j
}

pub fn delete(conn: &Connection, rowid: i64) {
    conn.execute("delete from note where rowid = ?1", &[&rowid])
        .unwrap();
}

pub fn insert(conn: &Connection, note: Note) {
    // mark is_last_note = 0 to indicate out of sync, i.e. db > ssb
    conn.execute(
        "
        INSERT INTO note (title, url, tags, description, comments, annotations, created_at)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7);

        ",
        &[
            &note.title,
            &note.url,
            &note.tags,
            &note.description,
            &note.comments,
            &note.annotations,
            &note.created_at,
        ],
    ).unwrap();

    conn.execute_batch(
        "BEGIN;
        UPDATE ssb SET is_last_note = 0
        WHERE is_active_author = 1;
        COMMIT;
        ",
    ).unwrap();
}

pub fn create(conn: &Connection) {
    conn.execute_batch(
        "BEGIN;
        CREATE TABLE IF NOT EXISTS note (
         rowid          INTEGER PRIMARY KEY AUTOINCREMENT,
         title          TEXT NOT NULL,
         url            TEXT NOT NULL,
         tags           TEXT NOT NULL,
         description    TEXT NOT NULL,
         comments       TEXT NOT NULL,
         annotations    TEXT NOT NULL,
         created_at     TEXT NOT NULL
         );

         CREATE TABLE IF NOT EXISTS ssb (
         note_rowid         INTEGER NOT NULL UNIQUE,
         author             TEXT PRIMARY KEY,
         is_active_author   BOOLEAN NOT NULL,
         is_last_note       BOOLEAN NOT NULL,
         ts                 INTEGER NOT NULL,
         seq                INTEGER NOT NULL,
         key                TEXT    NOT NULL,
         prev               TEXT    NOT NULL
         ) WITHOUT ROWID;
         COMMIT;",
    ).unwrap();
}

pub fn clear(conn: &Connection) {
    conn.execute_batch(
        "BEGIN;
        drop TABLE IF EXISTS note;
        drop TABLE IF EXISTS ssb;
        COMMIT;
        ",
    ).unwrap();
}
