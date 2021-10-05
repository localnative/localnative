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
use crate::{KVStringI64, Note, Tags};
use rusqlite::types::ToSql;
use rusqlite::Connection;
use std::collections::HashMap;

pub fn select_by_day(conn: &Connection) -> anyhow::Result<String> {
    let mut stmt = conn.prepare(
        "SELECT substr(created_at, 0, 11) as dt, count(1) as n
        FROM note
        group by dt
        order by dt",
    )?;
    let result_iter = stmt.query_map([], |row| {
        Ok(KVStringI64 {
            k: row.get(0)?,
            v: row.get(1)?,
        })
    })?;

    let mut d = "[ ".to_owned();
    for r in result_iter.filter(|r| r.is_ok()) {
        let r = r.unwrap();
        d.push_str(&serde_json::to_string(&r)?);
        d.push(',');
    }
    d.pop();
    d.push(']');
    Ok(d)
}

// fn make_json_array_string<T>(result_iter : MappedRows<T> ) -> String {
//     let mut d = "[ ".to_owned();
//     for r in result_iter {
//         let mut r = r.unwrap();
//         d.push_str(&serde_json::to_string(&r).unwrap());
//         d.push_str(",");
//     }
//     d.pop();
//     d.push_str("]");
//     d
// }

pub fn select_by_tag(conn: &Connection) -> anyhow::Result<String> {
    let mut stmt = conn.prepare(
        "SELECT tags
        FROM note",
    )?;

    let mut tag_count_map: HashMap<String, i64> = HashMap::new();

    let result_iter = stmt.query_map([], |row| Ok(Tags { tags: row.get(0)? }))?;

    for r in result_iter.filter(|r| r.is_ok()) {
        let r = r.unwrap();
        let tag_set = r.tags.split(',');
        for t in tag_set {
            if let Some(&v) = &tag_count_map.get(t) {
                tag_count_map.insert(t.to_lowercase(), v + 1);
            } else {
                tag_count_map.insert(t.to_lowercase(), 1);
            }
        }
    }

    let mut d = "[ ".to_owned();
    for (tag, &count) in &tag_count_map {
        let item = KVStringI64 {
            k: tag.to_string(),
            v: count,
        };
        d.push_str(&serde_json::to_string(&item)?);
        d.push(',');
    }
    d.pop();
    d.push(']');
    Ok(d)
}

pub fn select_count(conn: &Connection) -> anyhow::Result<u32> {
    let mut stmt = conn.prepare("SELECT count(1) FROM note")?;
    let count = stmt.query_row([], |row| row.get(0))?;
    Ok(count)
}

pub fn select(conn: &Connection, limit: u32, offset: u32) -> anyhow::Result<String> {
    let mut stmt = conn.prepare(
        "SELECT rowid, uuid4, title, url, tags, description, comments
        , hex(annotations)
        , created_at, is_public
        FROM note
        order by created_at desc limit :limit offset :offset",
    )?;
    let note_iter = stmt.query_map(
        &[
            (":limit", &limit as &dyn ToSql),
            (":offset", &offset as &dyn ToSql),
        ],
        |row| {
            Ok(Note {
                rowid: row.get(0)?,
                uuid4: row.get(1)?,
                title: row.get(2)?,
                url: row.get(3)?,
                tags: row.get(4)?,
                description: row.get(5)?,
                comments: row.get(6)?,
                annotations: super::utils::make_data_url(row).unwrap_or_else(|_| "".into()),
                created_at: row.get(8)?,
                is_public: row.get(9)?,
            })
        },
    )?;

    let mut j = "[ ".to_owned();
    for note in note_iter {
        let mut note = note?;
        note.tags = make_tags(&note.tags);
        //#[cfg(not(feature = "no_print"))]
        //eprintln!("Found note {:?}", note);
        j.push_str(&serde_json::to_string(&note)?);
        j.push(',');
    }
    j.pop();
    j.push(']');
    Ok(j)
}
