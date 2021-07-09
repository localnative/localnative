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
use crate::cmd;
use crate::cmd::{
    create, delete, filter, filter_by_tag, filter_count, insert, search, search_by_day,
    search_by_tag, search_count, select, select_by_day, select_by_tag, select_count,
    sync_via_attach,
};
use crate::upgrade;
use crate::Cmd;
use crate::CmdDelete;
use crate::CmdFilter;
use crate::CmdInsert;
use crate::CmdRpcClient;
use crate::CmdRpcServer;
use crate::CmdSearch;
use crate::CmdSelect;
use crate::CmdSyncViaAttach;
use crate::Note;
use rusqlite::Connection;
use std::fs;
use std::path::Path;
use time::macros::format_description;
use uuid::Uuid;

pub fn get_sqlite_connection() -> Connection {
    let p = sqlite3_db_location();
    let path = Path::new(&p);
    Connection::open(path).unwrap()
}

fn sqlite3_db_location() -> String {
    if cfg!(target_os = "android") {
        fs::create_dir_all("/sdcard/LocalNative").unwrap();
        return "/sdcard/LocalNative/localnative.sqlite3".to_string();
    }
    let mut dir_name = "LocalNative";
    if cfg!(target_os = "ios") {
        dir_name = "Documents";
    }
    let dir = format!(
        "{}/{}",
        dirs::home_dir().unwrap().to_str().unwrap(),
        dir_name
    );
    eprintln!("db dir location: {}", dir);
    fs::create_dir_all(&dir).unwrap();
    format!("{}/localnative.sqlite3", dir)
}

pub fn run(text: &str) -> String {
    if let Ok(cmd) = serde_json::from_str::<Cmd>(text) {
        process(cmd, text)
    } else {
        r#"{"error": "cmd json error"}"#.to_string()
    }
}

fn process(cmd: Cmd, text: &str) -> String {
    eprintln!("process cmd {:?}", cmd);
    let conn = get_sqlite_connection();
    create(&conn);

    // always run upgrade first
    if let Ok(version) = upgrade::upgrade(&conn) {
        eprintln!(r#"{{"upgrade-done": "{}"}}"#, version)
    } else {
        return r#"{"error":"upgrade error"}"#.to_string();
    }

    match cmd.action.as_ref() {
        "server" => {
            eprintln!(r#"{{"server": "starting"}}"#);
            if let Ok(s) = serde_json::from_str::<CmdRpcServer>(text) {
                if crate::rpc::server::start(&s.addr).is_ok() {
                    r#"{"server": "started"}"#.to_string()
                } else {
                    r#"{"error":"server error"}"#.to_string()
                }
            } else {
                r#"{"error":"cmd server error"}"#.to_string()
            }
        }
        "client-sync" => {
            eprintln!(r#"{{"client": "starting"}}"#);
            if let Ok(s) = serde_json::from_str::<CmdRpcClient>(text) {
                if let Ok(resp) = crate::rpc::client::sync(&s.addr) {
                    format!(r#"{{"client-sync": "{}"}}"#, resp)
                } else {
                    r#"{"error":"client-sync error"}"#.to_string()
                }
            } else {
                r#"{"error":"cmd client-sync error"}"#.to_string()
            }
        }
        "client-stop-server" => {
            eprintln!(r#"{{"client": "starting"}}"#);
            if let Ok(s) = serde_json::from_str::<CmdRpcClient>(text) {
                if let Ok(resp) = crate::rpc::client::stop_server(&s.addr) {
                    format!(r#"{{"client-stop-server": "{}"}}"#, resp)
                } else {
                    r#"{"error":"client-stop-server error"}"#.to_string()
                }
            } else {
                r#"{"error":"cmd client-stop-server error"}"#.to_string()
            }
        }
        "upgrade" => {
            if let Ok(version) = upgrade::upgrade(&conn) {
                format!(r#"{{"upgrade-done": "{}"}}"#, version)
            } else {
                r#"{"error":"cmd upgrade error"}"#.to_string()
            }
        }
        "sync-via-attach" => {
            if let Ok(s) = serde_json::from_str::<CmdSyncViaAttach>(text) {
                sync_via_attach(&conn, &s.uri)
            } else {
                r#"{"error":"cmd sync-via-attach error"}"#.to_string()
            }
        }
        "insert-image" => {
            if let Ok(i) = serde_json::from_str::<CmdInsert>(text) {
                let created_at = created_time();
                let note = Note {
                    rowid: 0i64,
                    uuid4: Uuid::new_v4().to_string(),
                    title: i.title,
                    url: i.url,
                    tags: i.tags,
                    description: i.description,
                    comments: i.comments,
                    annotations: i.annotations,
                    created_at,
                    is_public: i.is_public,
                };
                cmd::image::insert_image(note);
                if i.is_public {
                    eprintln!("is_public")
                }
                do_select(&conn, &i.limit, &i.offset)
            } else {
                r#"{"error":"cmd insert json error"}"#.to_string()
            }
        }
        "insert" => {
            if let Ok(i) = serde_json::from_str::<CmdInsert>(text) {
                let created_at = created_time();
                eprintln!("created_at {}", &created_at);
                let note = Note {
                    rowid: 0i64,
                    uuid4: Uuid::new_v4().to_string(),
                    title: i.title,
                    url: i.url,
                    tags: i.tags,
                    description: i.description,
                    comments: i.comments,
                    annotations: i.annotations,
                    created_at,
                    is_public: i.is_public,
                };
                insert(note);
                if i.is_public {
                    eprintln!("is_public")
                }
                do_select(&conn, &i.limit, &i.offset)
            } else {
                r#"{"error":"cmd insert json error"}"#.to_string()
            }
        }
        "delete" => {
            if let Ok(s) = serde_json::from_str::<CmdDelete>(text) {
                delete(&conn, s.rowid);
                do_search(&conn, &s.query, &s.limit, &s.offset)
            } else {
                r#"{"error":"cmd delete json error"}"#.to_string()
            }
        }
        "select" => {
            if let Ok(s) = serde_json::from_str::<CmdSelect>(text) {
                do_select(&conn, &s.limit, &s.offset)
            } else {
                r#"{"error":"cmd select json error"}"#.to_string()
            }
        }
        "search" => {
            if let Ok(s) = serde_json::from_str::<CmdSearch>(text) {
                do_search(&conn, &s.query, &s.limit, &s.offset)
            } else {
                r#"{"error":"cmd search json error"}"#.to_string()
            }
        }
        "filter" => {
            if let Ok(s) = serde_json::from_str::<CmdFilter>(text) {
                do_filter(&conn, &s.query, &s.limit, &s.offset, &s.from, &s.to)
            } else {
                r#"{"error":"cmd filter json error"}"#.to_string()
            }
        }
        _ => r#"{"error": "cmd no match"}"#.to_string(),
    }
}

fn created_time() -> String {
    let created_at = time::OffsetDateTime::now_utc();
    created_at
        .format(&format_description!(
            "[year]-[month]-[day] [hour]:[minute]:[second]:"
        ))
        .unwrap()
        + created_at.nanosecond().to_string().as_str()
        + " UTC"
}

pub fn do_search(conn: &Connection, query: &str, limit: &u32, offset: &u32) -> String {
    let c = search_count(&conn, query);
    let j = search(&conn, query, limit, offset);
    let d = search_by_day(&conn, query);
    let t = search_by_tag(&conn, query);
    let msg = format!(
        r#"{{"count": {}, "notes":{}, "days": {}, "tags": {} }}"#,
        c, j, d, t
    );
    // eprintln!("msg {}", msg);
    msg
}

fn do_select(conn: &Connection, limit: &u32, offset: &u32) -> String {
    let c = select_count(&conn);
    let j = select(&conn, limit, offset);
    let d = select_by_day(&conn);
    let t = select_by_tag(&conn);
    let msg = format!(
        r#"{{"count": {}, "notes":{}, "days": {}, "tags": {} }}"#,
        c, j, d, t
    );
    // eprintln!("msg {}", msg);
    msg
}

pub fn do_filter(
    conn: &Connection,
    query: &str,
    limit: &u32,
    offset: &u32,
    from: &str,
    to: &str,
) -> String {
    let c = filter_count(&conn, query, from, to);
    let j = filter(&conn, query, from, to, limit, offset);
    let d = search_by_day(&conn, query);
    let t = filter_by_tag(&conn, query, from, to);
    let msg = format!(
        r#"{{"count": {}, "notes":{},"days": {}, "tags": {} }}"#,
        c, j, d, t
    );
    // eprintln!("msg {}", msg);
    msg
}
