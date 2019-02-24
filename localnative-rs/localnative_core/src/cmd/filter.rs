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
extern crate regex;
extern crate rusqlite;
extern crate serde;
extern crate serde_json;
use self::regex::Regex;
use self::rusqlite::types::ToSql;
use self::rusqlite::Connection;
use super::make_tags;
use super::select::{select, select_by_day, select_by_tag, select_count};
use std::collections::HashMap;
use {KVStringI64, Note, Tags};

pub fn filter_by_tag(conn: &Connection, query: &str, from: &str, to: &str) -> String {
    let words = make_words(query);
    if words.len() == 1 && words.get(0).unwrap().is_empty() {
        return select_by_tag(conn);
    }
    let num_words = words.len();
    let r: Vec<String> = where_vec(num_words);
    let sql = format!(
        "SELECT tags
        FROM note where
        substr(created_at, 0, 11) >= :from
        and substr(created_at, 0, 11) <= :to
        and {}",
        r.join(" and ")
    );

    eprintln!("sql {}", sql);

    let mut stmt = conn.prepare(&sql).unwrap();
    let keys: Vec<String> = make_keys(num_words);

    let mut params: Vec<(&str, &ToSql)> = vec![(":from", &from as &ToSql), (":to", &to as &ToSql)];
    for i in 0..num_words {
        params.push((&keys.get(i).unwrap(), words.get(i).unwrap() as &ToSql));
    }

    eprintln!("params {:?}", params.len());

    let mut tag_count_map: HashMap<String, i64> = HashMap::new();

    let result_iter = stmt
        .query_map_named(&params, |row| Tags { tags: row.get(0) })
        .unwrap();

    for r in result_iter {
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
        d.push_str(&serde_json::to_string(&item).unwrap());
        d.push_str(",");
    }
    d.pop();
    d.push_str("]");
    d
}

pub fn filter_count(conn: &Connection, query: &str, from: &str, to: &str) -> u32 {
    let words = make_words(query);
    if words.len() == 1 && words.get(0).unwrap().is_empty() {
        return select_count(conn);
    }
    let num_words = words.len();
    eprintln!("{} words {:?}", num_words, words);

    let r: Vec<String> = where_vec(num_words);
    let sql = format!(
        "SELECT count(1)
        FROM note where
        substr(created_at, 0, 11) >= :from
        and substr(created_at, 0, 11) <= :to
        and {}",
        r.join(" and ")
    );

    eprintln!("sql {}", sql);

    let mut stmt = conn.prepare(&sql).unwrap();
    let keys: Vec<String> = make_keys(num_words);

    let mut params: Vec<(&str, &ToSql)> = vec![(":from", &from as &ToSql), (":to", &to as &ToSql)];
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

pub fn filter(
    conn: &Connection,
    query: &str,
    from: &str,
    to: &str,
    limit: &u32,
    offset: &u32,
) -> String {
    let words = make_words(query);
    if words.len() == 1 && words.get(0).unwrap().is_empty() {
        return select(conn, limit, offset);
    }
    let num_words = words.len();
    eprintln!("{} words {:?}", num_words, words);

    let r: Vec<String> = where_vec(num_words);
    let sql = format!(
        "SELECT rowid, title, url, tags, description, comments, annotations, created_at, is_public
        FROM note where
        substr(created_at, 0, 11) >= :from
        and substr(created_at, 0, 11) <= :to
        and {}
        order by created_at desc limit :limit offset :offset",
        r.join(" and ")
    );

    eprintln!("sql {}", sql);

    let mut stmt = conn.prepare(&sql).unwrap();
    let keys: Vec<String> = make_keys(num_words);

    let mut params: Vec<(&str, &ToSql)> = vec![
        (":from", &from as &ToSql),
        (":to", &to as &ToSql),
        (":limit", limit as &ToSql),
        (":offset", offset as &ToSql),
    ];

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

fn make_words(query: &str) -> Vec<String> {
    let re1 = Regex::new(r"\s+").unwrap();
    let s1 = re1.replace_all(query, " ");
    s1.trim()
        .split(" ")
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
