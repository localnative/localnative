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
use super::select::{select, select_by_day, select_by_tag, select_count};
use crate::{KVStringI64, Note, Tags};
use regex::Regex;
use rusqlite::types::ToSql;
use rusqlite::Connection;
use std::collections::HashMap;

pub fn search_by_tag(conn: &Connection, query: &str) -> anyhow::Result<String> {
    let words = make_words(query);
    if words.len() == 1 && words.get(0).unwrap().is_empty() {
        return select_by_tag(conn);
    }
    let num_words = words.len();
    let r: Vec<String> = where_vec(num_words);
    let sql = format!(
        "SELECT tags
        FROM note where
        {}",
        r.join(" and ")
    );
    #[cfg(not(feature = "no_print"))]
    eprintln!("sql {}", sql);

    let mut stmt = conn.prepare(&sql)?;
    let keys: Vec<String> = make_keys(num_words);

    let mut params: Vec<(&str, &dyn ToSql)> = vec![];
    for i in 0..num_words {
        params.push((
            &keys.get(i).ok_or(anyhow::anyhow!("keys is empty"))?,
            words.get(i).ok_or(anyhow::anyhow!("words is empty"))? as &dyn ToSql,
        ));
    }

    let mut tag_count_map: HashMap<String, i64> = HashMap::new();

    let result_iter = stmt.query_map(&params[..], |row| Ok(Tags { tags: row.get(0)? }))?;

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

pub fn search_by_day(conn: &Connection, query: &str) -> anyhow::Result<String> {
    let words = make_words(query);
    if words.len() == 1 && words.get(0).unwrap().is_empty() {
        return select_by_day(conn);
    }
    let num_words = words.len();
    let r: Vec<String> = where_vec(num_words);
    let sql = format!(
        "SELECT substr(created_at, 0, 11) as dt, count(1) as n
        FROM note where
        {}
        group by dt
        order by dt",
        r.join(" and ")
    );
    #[cfg(not(feature = "no_print"))]
    eprintln!("sql {}", sql);

    let mut stmt = conn.prepare(&sql)?;
    let keys: Vec<String> = make_keys(num_words);

    let mut params: Vec<(&str, &dyn ToSql)> = vec![];
    for i in 0..num_words {
        params.push((
            &keys.get(i).ok_or(anyhow::anyhow!("keys is empty"))?,
            words.get(i).ok_or(anyhow::anyhow!("words is empty"))? as &dyn ToSql,
        ));
    }

    let result_iter = stmt.query_map(&params[..], |row| {
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

pub fn search_count(conn: &Connection, query: &str) -> anyhow::Result<u32> {
    let words = make_words(query);
    if words.len() == 1 && words.get(0).unwrap().is_empty() {
        return select_count(conn);
    }
    let num_words = words.len();
    #[cfg(not(feature = "no_print"))]
    eprintln!("{} words {:?}", num_words, words);

    let r: Vec<String> = where_vec(num_words);
    let sql = format!(
        "SELECT count(1)
        FROM note where
        {}",
        r.join(" and ")
    );
    #[cfg(not(feature = "no_print"))]
    eprintln!("sql {}", sql);

    let mut stmt = match conn.prepare(&sql) {
        Ok(s) => s,
        Err(e) => {
            if let rusqlite::Error::SqliteFailure(_, _) = e {
                crate::cmd::create(conn)?;
            }
            conn.prepare(&sql)?
        }
    };
    let keys: Vec<String> = make_keys(num_words);

    let mut params: Vec<(&str, &dyn ToSql)> = vec![];
    for i in 0..num_words {
        params.push((
            &keys.get(i).ok_or(anyhow::anyhow!("keys is empty"))?,
            words.get(i).ok_or(anyhow::anyhow!("words is empty"))? as &dyn ToSql,
        ));
    }
    #[cfg(not(feature = "no_print"))]
    eprintln!("params {:?}", params.len());

    let rs = stmt.query_map(&params[..], |row| row.get(0))?;
    let mut c: u32 = 0;
    for r in rs {
        c = r?;
    }
    Ok(c)
}

pub fn search(conn: &Connection, query: &str, limit: &u32, offset: &u32) -> anyhow::Result<String> {
    let words = make_words(query);
    if words.len() == 1
        && words
            .get(0)
            .ok_or(anyhow::anyhow!("words is empty"))?
            .is_empty()
    {
        return select(conn, limit, offset);
    }
    let num_words = words.len();
    #[cfg(not(feature = "no_print"))]
    eprintln!("{} words {:?}", num_words, words);

    let r: Vec<String> = where_vec(num_words);
    let sql = format!(
        "SELECT rowid, uuid4, title, url, tags, description, comments
        , hex(annotations)
        , created_at, is_public
        FROM note where
        {}
        order by created_at desc limit :limit offset :offset",
        r.join(" and ")
    );
    #[cfg(not(feature = "no_print"))]
    eprintln!("sql {}", sql);

    let mut stmt = conn.prepare(&sql)?;
    let keys: Vec<String> = make_keys(num_words);

    let mut params: Vec<(&str, &dyn ToSql)> = vec![
        (":limit", limit as &dyn ToSql),
        (":offset", offset as &dyn ToSql),
    ];

    for i in 0..num_words {
        params.push((
            &keys.get(i).ok_or(anyhow::anyhow!("keys is empty"))?,
            words.get(i).ok_or(anyhow::anyhow!("words is empty"))? as &dyn ToSql,
        ));
    }
    #[cfg(not(feature = "no_print"))]
    eprintln!("params {:?}", params.len());

    let note_iter = stmt.query_map(&params[..], |row| {
        Ok(Note {
            rowid: row.get(0)?,
            uuid4: row.get(1)?,
            title: row.get(2)?,
            url: row.get(3)?,
            tags: row.get(4)?,
            description: row.get(5)?,
            comments: row.get(6)?,
            annotations: super::utils::make_data_url(row).unwrap_or("".into()),
            created_at: row.get(8)?,
            is_public: row.get(9)?,
        })
    })?;

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

fn make_words(query: &str) -> Vec<String> {
    let re1 = Regex::new(r"\s+").unwrap();
    let s1 = re1.replace_all(query, " ");
    s1.trim()
        .split(' ')
        .map(|w| format!("%{}%", w))
        .collect::<Vec<String>>()
}

fn make_keys(num_words: usize) -> Vec<String> {
    (0..num_words)
        .map(|i| ":w".to_string() + &i.to_string())
        .collect()
}

fn where_vec(num_words: usize) -> Vec<String> {
    (0..num_words)
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
        .collect()
}
