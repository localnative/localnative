use super::cmd::{create, delete, insert, select};
use rusqlite::Connection;

use super::ssb;
use super::Cmd;
use super::CmdDelete;
use super::CmdInsert;
use super::CmdSearch;
use super::CmdSelect;
use super::Note;

extern crate serde_json;
extern crate time;

pub fn run(text: &str) -> String {
    if let Ok(cmd) = serde_json::from_str::<Cmd>(text) {
        process(cmd, text)
    } else {
        r#"{"error": "cmd json error"}"#.to_string()
    }
}

fn process(cmd: Cmd, text: &str) -> String {
    eprintln!("process cmd {:?}", cmd);
    let conn = ssb::get_sqlite_connection();
    create(&conn);

    match cmd.action.as_ref() {
        "insert" => {
            if let Ok(i) = serde_json::from_str::<CmdInsert>(text) {
                let created_at =
                    time::strftime("%Y-%m-%d %H:%M:%S:%f UTC", &time::now_utc()).unwrap();
                //eprintln!("created_at {}", created_at);
                let note = Note {
                    rowid: 0i64,
                    title: i.title,
                    url: i.url,
                    tags: i.tags,
                    description: i.description,
                    comments: i.comments,
                    annotations: i.annotations,
                    created_at,
                };
                insert(note);
                do_select(&conn, "")
            } else {
                r#"{"error":"cmd insert json error"}"#.to_string()
            }
        }
        "delete" => {
            if let Ok(s) = serde_json::from_str::<CmdDelete>(text) {
                delete(&conn, s.rowid);
                do_select(&conn, &s.query)
            } else {
                r#"{"error":"cmd delete json error"}"#.to_string()
            }
        }
        "select" => {
            if let Ok(s) = serde_json::from_str::<CmdSelect>(text) {
                do_select(&conn, "")
            } else {
                r#"{"error":"cmd select json error"}"#.to_string()
            }
        }
        "search" => {
            if let Ok(s) = serde_json::from_str::<CmdSearch>(text) {
                do_select(&conn, &s.query)
            } else {
                r#"{"error":"cmd search json error"}"#.to_string()
            }
        }
        _ => r#"{"error": "cmd no match"}"#.to_string(),
    }
}

fn do_select(conn: &Connection, query: &str) -> String {
    let j = select(&conn, query);
    let msg = format!("{{\"notes\":{}}}", j);
    eprintln!("msg {}", msg);
    msg
}
