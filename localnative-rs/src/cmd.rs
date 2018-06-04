extern crate rusqlite;
extern crate serde;
extern crate serde_json;
extern crate time;

use self::rusqlite::Connection;
use super::Cmd;
use super::Note;

pub fn select(conn: &Connection, sql: &str) -> String {
    let mut stmt = conn.prepare(sql).unwrap();
    let note_iter = stmt.query_map(&[], |row| Note {
        title: row.get(0),
        url: row.get(1),
        tags: row.get(2),
        description: row.get(3),
        comments: row.get(4),
        annotations: row.get(5),
        created_at: row.get(6),
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

pub fn insert(conn: &Connection, note: Note) {
    conn.execute(
        "INSERT INTO note (title, url, tags, description, comments, annotations, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
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
}

pub fn create(conn: &Connection) {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS note (
         title          TEXT NOT NULL,
         url            TEXT NOT NULL,
         tags           TEXT NOT NULL,
         description    TEXT NOT NULL,
         comments       TEXT NOT NULL,
         annotations    TEXT NOT NULL,
         created_at     TEXT NOT NULL
         )",
        &[],
    ).unwrap();
}
