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
extern crate linked_hash_set;
extern crate regex;
extern crate rusqlite;
extern crate serde;
extern crate serde_json;
use self::linked_hash_set::LinkedHashSet;
use self::regex::Regex;
use self::rusqlite::types::ToSql;
use self::rusqlite::{Connection, NO_PARAMS};
use std::iter::FromIterator;
use {KVStringI64, Note};
mod filter;
pub mod image;
mod search;
mod select;
mod utils;
pub use self::filter::{filter, filter_by_tag, filter_count};
pub use self::search::{search, search_by_day, search_by_tag, search_count};
pub use self::select::{select, select_by_day, select_by_tag, select_count};

pub fn sync_via_attach(conn: &Connection, uri: &str) -> String {
    if let Ok(_) = conn.execute("attach ? as 'other'", &[uri]) {
        match conn.execute_batch("BEGIN;
        insert into note (title, url, tags, description, comments, annotations, created_at, is_public)
        select title, url, tags, description, comments, annotations, created_at, is_public
from other.note
        where not exists (
            select 1 from note
            where note.title = other.note.title
            and note.url = other.note.url
            and note.tags = other.note.tags
            and note.description = other.note.description
            and note.comments = other.note.comments
            and note.annotations= other.note.annotations
            and note.created_at = other.note.created_at
            and note.is_public = other.note.is_public
        );
        insert into other.note (title, url, tags, description, comments, annotations, created_at, is_public)
        select title, url, tags, description, comments, annotations, created_at, is_public
from note
        where not exists (
            select 1 from other.note as o
            where o.title = note.title
            and o.url = note.url
            and o.tags = note.tags
            and o.description = note.description
            and o.comments = note.comments
            and o.annotations= note.annotations
            and o.created_at = note.created_at
            and o.is_public = note.is_public
        );
        COMMIT;
        detach database other;
        "){
            Ok(_) => {
                format!(r#"{{"sync-via-attach-done": "{}"}}"#, uri)
            }
            Err(err) => {
                eprintln!("Err {:?}", err);
                format!(r#"{{"error": "{}"}}"#, err.to_string())
            }
        }
    } else {
        format!(r#"{{"error": "can not attach {}"}}"#, uri)
    }
}

pub fn count(conn: &Connection, tbl: &str) -> i64 {
    let mut stmt = conn
        .prepare(&format!("select count(1) as cnt from {}", tbl))
        .unwrap();
    let rs = stmt.query_row(NO_PARAMS, |row| row.get(0)).unwrap();
    rs
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
    let conn = &mut super::exe::get_sqlite_connection();
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
                &note.is_public as &ToSql,
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
            NO_PARAMS,
        )
        .unwrap();
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
    )
    .unwrap();
}

pub fn clear(conn: &Connection) {
    conn.execute_batch(
        "BEGIN;
        drop TABLE IF EXISTS note;
        drop TABLE IF EXISTS ssb;
        COMMIT;
        ",
    )
    .unwrap();
}
