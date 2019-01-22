extern crate linked_hash_set;
extern crate regex;
extern crate rusqlite;
extern crate serde;
extern crate serde_json;
use self::rusqlite::types::ToSql;
use super::Note;
use self::rusqlite::{Connection, NO_PARAMS};
use super::make_tags;

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
