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
