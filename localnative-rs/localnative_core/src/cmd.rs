extern crate linked_hash_set;
extern crate regex;
extern crate rusqlite;
extern crate serde;
extern crate serde_json;
use self::linked_hash_set::LinkedHashSet;
use self::regex::Regex;
use self::rusqlite::Connection;
use super::Note;
use std::iter::FromIterator;

pub fn count(conn: &Connection, tbl: &str) -> i64 {
    let mut stmt = conn
        .prepare(&format!("select count(1) as cnt from {}", tbl))
        .unwrap();
    let rs = stmt.query_row(&[], |row| row.get(0)).unwrap();
    rs
}

pub fn select(conn: &Connection, query: &str) -> String {
    let mut stmt = conn.prepare(
        "SELECT rowid, title, url, tags, description, comments, annotations, created_at, is_public
        FROM note
        where title like :query
        or url like :query
        or tags like :query
        or description like :query
        order by created_at desc limit 15"
        ).unwrap();
    let note_iter = stmt
        .query_map_named(&[(":query", &format!("%{}%", query))], |row| Note {
            rowid: row.get(0),
            title: row.get(1),
            url: row.get(2),
            tags: row.get(3),
            description: row.get(4),
            comments: row.get(5),
            annotations: "".to_string(), //row.get(6),
            created_at: row.get(7),
            is_public: row.get(8),
        }).unwrap();

    let mut j = "[ ".to_owned();
    for note in note_iter {
        let mut note = note.unwrap();
        note.tags = make_tags(&note.tags);
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

// format and dedup tags
pub fn make_tags(input: &str) -> String {
    let re1 = Regex::new(r",+").unwrap();
    let re2 = Regex::new(r"\s+").unwrap();
    let s1 = re1.replace_all(input, " ");
    let s2 = re2.replace_all(s1.trim(), ",");
    let v1 = s2.split(",");
    let h1: LinkedHashSet<&str> = LinkedHashSet::from_iter(v1);
    let mut s = "".to_string();
    for e in h1 {
        s.push_str(e);
        s.push_str(",")
    }
    s.pop();
    s.to_string()
}

pub fn insert(note: Note) {
    let conn = &mut super::ssb::get_sqlite_connection();
    let tx = conn.transaction().unwrap();
    {
        tx.execute(
            "
        INSERT INTO note (title, url, tags, description, comments, annotations, created_at, is_public)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8);

        ",
            &[
                &note.title,
                &note.url,
                &make_tags(&note.tags),
                &note.description,
                &note.comments,
                &note.annotations,
                &note.created_at,
                &note.is_public,
            ],
        ).unwrap();
    }
    {
        // mark is_last_note = 0 to indicate out of sync, i.e. db > ssb
        tx.execute(
            "
        UPDATE ssb SET is_last_note = 0
        WHERE is_active_author = 1
        ",
            &[],
        ).unwrap();
    }
    tx.commit().unwrap();
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
         created_at     TEXT NOT NULL,
         is_public      BOOLEAN NOT NULL default 0
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
