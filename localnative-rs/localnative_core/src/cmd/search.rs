extern crate linked_hash_set;
extern crate regex;
extern crate rusqlite;
extern crate serde;
extern crate serde_json;
use self::regex::Regex;
use self::rusqlite::types::ToSql;
use super::Note;
use self::rusqlite::{Connection};
use super::select::{select, select_count};
use super::make_tags;

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
