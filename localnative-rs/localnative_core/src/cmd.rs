extern crate linked_hash_set;
extern crate regex;
extern crate rusqlite;
extern crate serde;
extern crate serde_json;
use self::linked_hash_set::LinkedHashSet;
use self::regex::Regex;
use self::rusqlite::types::ToSql;
use self::rusqlite::Result;
use self::rusqlite::{Connection, NO_PARAMS};
use super::Note;
use std::iter::FromIterator;

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
        COMMIT;
        BEGIN;
        insert into other.note (title, url, tags, description, comments, annotations, created_at, is_public)
        select title, url, tags, description, comments, annotations, created_at, is_public
from note
        where not exists (
            select 1 from other.note
            where other.note.title = note.title
            and other.note.url = note.url
            and other.note.tags = note.tags
            and other.note.description = note.description
            and other.note.comments = note.comments
            and other.note.annotations= note.annotations
            and other.note.created_at = note.created_at
            and other.note.is_public = note.is_public
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

    // let r: Result<String> =
    //     conn.query_row("SELECT * FROM other.note", NO_PARAMS, |row| {
    //         row.get(2)
    //     });
    // match r {
    //     Ok(f) => {
    //         eprintln!("Ok {:?}", f);
    //         format!(r#"{{"sync-via-attach": "{}"}}"#, uri)
    //     }
    //     Err(err) => {
    //         eprintln!("Err {:?}", err);
    //         format!(r#"{{"error": "{}"}}"#, err.to_string())
    //     }
    // }
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

pub fn search_count(conn: &Connection, query: &str) -> u32 {
    let re1 = Regex::new(r"\s+").unwrap();
    let s1 = re1.replace_all(query, " ");
    let words = s1
        .trim()
        .split(" ")
        .map(|w| format!("%{}%", w))
        .collect::<Vec<String>>();
    if words.len() == 1 && words.get(0).unwrap().is_empty() {
        return select_count(conn);
    }
    let num_words = words.len();
    eprintln!("{} words {:?}", num_words, words);

    let r: Vec<String> = (0..num_words)
        .map(|i| {
            format!(
                "(
        title like :w{}
        or url like :w{}
        or tags like :w{}
        or description like :w{}
        )",
                i.to_string(),
                i.to_string(),
                i.to_string(),
                i.to_string()
            )
        })
        .collect();

    let sql = format!(
        "SELECT count(1)
        FROM note where
        {}",
        r.join(" and ")
    );

    eprintln!("sql {}", sql);

    let mut stmt = conn.prepare(&sql).unwrap();

    let keys: Vec<String> = (0..num_words)
        .map(|i| ":w".to_string() + &i.to_string())
        .collect();

    let mut params: Vec<(&str, &ToSql)> = vec![];
    for i in 0..num_words {
        params.push((&keys.get(i).unwrap(), words.get(i).unwrap() as &ToSql));
    }

    eprintln!("params {:?}", params.len());

    let rs = stmt.query_map_named(&params, |row| row.get(0)).unwrap();
    let mut c: u32 = 0;
    for r in rs {
        c = r.unwrap();
    }
    c
}

pub fn search(conn: &Connection, query: &str, limit: &u32, offset: &u32) -> String {
    let re1 = Regex::new(r"\s+").unwrap();
    let s1 = re1.replace_all(query, " ");
    let words = s1
        .trim()
        .split(" ")
        .map(|w| format!("%{}%", w))
        .collect::<Vec<String>>();
    if words.len() == 1 && words.get(0).unwrap().is_empty() {
        return select(conn, limit, offset);
    }
    let num_words = words.len();
    eprintln!("{} words {:?}", num_words, words);

    let r: Vec<String> = (0..num_words)
        .map(|i| {
            format!(
                "(
        title like :w{}
        or url like :w{}
        or tags like :w{}
        or description like :w{}
        )",
                i.to_string(),
                i.to_string(),
                i.to_string(),
                i.to_string()
            )
        })
        .collect();

    let sql = format!(
        "SELECT rowid, title, url, tags, description, comments, annotations, created_at, is_public
        FROM note where
        {}
        order by created_at desc limit :limit offset :offset",
        r.join(" and ")
    );

    eprintln!("sql {}", sql);

    let mut stmt = conn.prepare(&sql).unwrap();

    let keys: Vec<String> = (0..num_words)
        .map(|i| ":w".to_string() + &i.to_string())
        .collect();

    let mut params: Vec<(&str, &ToSql)> =
        vec![(":limit", limit as &ToSql), (":offset", offset as &ToSql)];

    for i in 0..num_words {
        params.push((&keys.get(i).unwrap(), words.get(i).unwrap() as &ToSql));
    }

    eprintln!("params {:?}", params.len());

    let note_iter = stmt
        .query_map_named(&params, |row| Note {
            rowid: row.get(0),
            title: row.get(1),
            url: row.get(2),
            tags: row.get(3),
            description: row.get(4),
            comments: row.get(5),
            annotations: "".to_string(), //row.get(6),
            created_at: row.get(7),
            is_public: row.get(8),
        })
        .unwrap();

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

pub fn select_count(conn: &Connection) -> u32 {
    let mut stmt = conn.prepare("SELECT count(1) FROM note").unwrap();
    let rs = stmt.query_row(NO_PARAMS, |row| row.get(0)).unwrap();
    rs
}

pub fn select(conn: &Connection, limit: &u32, offset: &u32) -> String {
    let mut stmt = conn.prepare(
        "SELECT rowid, title, url, tags, description, comments, annotations, created_at, is_public
        FROM note
        order by created_at desc limit :limit offset :offset"
        ).unwrap();
    let note_iter = stmt
        .query_map_named(
            &[(":limit", limit as &ToSql), (":offset", offset as &ToSql)],
            |row| Note {
                rowid: row.get(0),
                title: row.get(1),
                url: row.get(2),
                tags: row.get(3),
                description: row.get(4),
                comments: row.get(5),
                annotations: "".to_string(), //row.get(6),
                created_at: row.get(7),
                is_public: row.get(8),
            },
        )
        .unwrap();

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
