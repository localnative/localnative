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
use super::make_tags;
use crate::{KVStringI64, Note, OneString, Tags};
use rusqlite::types::ToSql;
use rusqlite::{Connection, NO_PARAMS};
use std::collections::HashMap;

//client
pub fn get_note_by_uuid4(conn: &Connection, uuid4: &str) -> Note {
    let mut stmt = conn.prepare(
"select uuid4, title, url, tags, description, comments, annotations, created_at FROM note where uuid4 = ? "
    ).unwrap();
    let note = stmt
        .query_row(&[uuid4], |row| {
            Ok(Note {
                rowid: 0,
                uuid4: row.get(0)?,
                title: row.get(1)?,
                url: row.get(2)?,
                tags: row.get(3)?,
                description: row.get(4)?,
                comments: row.get(5)?,
                annotations: row.get(6)?,
                created_at: row.get(7)?,
                is_public: false,
            })
        })
        .unwrap();
    note
}

pub fn next_uuid4_candidates(conn: &Connection) -> Vec<String> {
    let mut r = Vec::new();
    let mut stmt = conn
        .prepare("select uuid4 FROM note order by rowid")
        .unwrap();
    let iter = stmt
        .query_map(NO_PARAMS, |row| Ok(OneString { s: row.get(0)? }))
        .unwrap();
    for i in iter {
        if let Ok(uuid4) = i {
            r.push(uuid4.s)
        }
    }
    r
}

// server
pub fn diff_uuid4(conn: &Connection, candidates: Vec<String>) -> Vec<String> {
    let mut r = Vec::new();
    let mut stmt = conn.prepare("select 1 FROM note where uuid4 = ? ").unwrap();
    for uuid4 in candidates {
        if !(stmt.exists(&[&uuid4]).unwrap()) {
            r.push(uuid4);
        }
    }
    r
}
