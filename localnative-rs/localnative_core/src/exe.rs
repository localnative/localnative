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
use crate::Note;
use rusqlite::Connection;
use std::fs;
use std::path::Path;
use time::macros::format_description;
use uuid::Uuid;

use serde::Serialize;
use thiserror::Error;

#[derive(Error, Debug, Serialize)]
#[serde(rename(serialize = "error", deserialize = "error"))]
pub enum ProcessError {
    #[error("cmd upgrade error: {0}")]
    UpgradeFailure(String),
    #[error("cmd json error: {0}")]
    JsonParseFailure(String),
    #[error("start server failure:{0}")]
    StartServerFailure(String),
    #[error("client-sync error:{0}")]
    ClientSyncFailure(String),
    #[error("client-stop-server error: {0}")]
    ClientStopServerFailure(String),
    #[error("unknown error: {0} from {1}")]
    Unknown(String, String),
}

pub fn get_sqlite_connection() -> Connection {
    let p = sqlite3_db_location();
    let path = Path::new(&p);
    Connection::open(path).unwrap()
}

fn sqlite3_db_location() -> String {
    if cfg!(target_os = "android") {
        let path = "sdcard/LocalNative";
        if let Err(e) = fs::create_dir_all(path) {
            panic!("{}", e);
        };
        return format!("{}/localnative.sqlite3", path);
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
        match process(cmd).map_err(|err| serde_json::to_string(&err).unwrap()) {
            Ok(rs) => rs,
            Err(err) => err,
        }
    } else {
        let err = ProcessError::JsonParseFailure(text.into());
        serde_json::to_string(&err).unwrap()
    }
}

fn process(cmd: Cmd) -> anyhow::Result<String, ProcessError> {
    eprintln!("process cmd {:?}", cmd);
    let conn = get_sqlite_connection();
    create(&conn).map_err(|err| ProcessError::Unknown(err.to_string(), "create conn".into()))?;

    // always run upgrade first
    if let Ok(version) = upgrade::upgrade(&conn) {
        eprintln!(r#"{{"upgrade-done": "{}"}}"#, version)
    } else {
        return Err(ProcessError::UpgradeFailure("init".into()));
    }

    match cmd {
        Cmd::Server(s) => match crate::rpc::server::start(&s.addr) {
            Ok(_) => Ok(r#"{"server": "started"}"#.to_string()),
            Err(e) => Err(ProcessError::StartServerFailure(e.into())),
        },
        Cmd::ClientSync(s) => {
            eprintln!(r#"{{"client": "starting"}}"#);
            match crate::rpc::client::sync(&s.addr) {
                Ok(resp) => Ok(format!(r#"{{"client-sync": "{}"}}"#, resp)),
                Err(err) => Err(ProcessError::ClientSyncFailure(err.to_string())),
            }
        }
        Cmd::ClientStopServer(s) => {
            eprintln!(r#"{{"client": "starting"}}"#);
            match crate::rpc::client::stop_server(&s.addr) {
                Ok(resp) => Ok(format!(r#"{{"client-stop-server": "{}"}}"#, resp)),
                Err(err) => Err(ProcessError::ClientStopServerFailure(err.to_string())),
            }
        }
        Cmd::Upgrade => upgrade::upgrade(&conn)
            .map(|version| format!(r#"{{"upgrade-done": "{}"}}"#, version))
            .map_err(|err| ProcessError::UpgradeFailure(err.to_string())),
        Cmd::SyncViaAttach(s) => Ok(sync_via_attach(&conn, &s.uri)),
        Cmd::InsertImage(i) => {
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
            cmd::image::insert_image(note)
                .map_err(|err| ProcessError::Unknown(err.to_string(), "insert image op".into()))?;
            if i.is_public {
                eprintln!("is_public")
            }
            do_select(&conn, i.limit, i.offset)
                .map_err(|err| ProcessError::Unknown(err.to_string(), "insert image".into()))
        }
        Cmd::Insert(i) => {
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
            insert(note)
                .map_err(|err| ProcessError::Unknown(err.to_string(), "insert op".into()))?;
            if i.is_public {
                eprintln!("is_public")
            }
            do_select(&conn, i.limit, i.offset)
                .map_err(|err| ProcessError::Unknown(err.to_string(), "insert".into()))
        }
        Cmd::Delete(s) => {
            delete(&conn, s.rowid)
                .map_err(|err| ProcessError::Unknown(err.to_string(), "delete op".into()))?;
            do_search(&conn, &s.query, s.limit, s.offset)
                .map_err(|err| ProcessError::Unknown(err.to_string(), "delete".into()))
        }
        Cmd::Select(s) => do_select(&conn, s.limit, s.offset)
            .map_err(|err| ProcessError::Unknown(err.to_string(), "select".into())),
        Cmd::Search(s) => do_search(&conn, &s.query, s.limit, s.offset)
            .map_err(|err| ProcessError::Unknown(err.to_string(), "search".into())),
        Cmd::Filter(s) => do_filter(&conn, &s.query, s.limit, s.offset, &s.from, &s.to)
            .map_err(|err| ProcessError::Unknown(err.to_string(), "filter".into())),
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

pub fn do_search(
    conn: &Connection,
    query: &str,
    limit: u32,
    offset: u32,
) -> anyhow::Result<String> {
    let c = search_count(conn, query)?;
    let j = search(conn, query, limit, offset)?;
    let d = search_by_day(conn, query)?;
    let t = search_by_tag(conn, query)?;
    let msg = format!(
        r#"{{"count": {}, "notes":{}, "days": {}, "tags": {} }}"#,
        c, j, d, t
    );
    // eprintln!("msg {}", msg);
    Ok(msg)
}

fn do_select(conn: &Connection, limit: u32, offset: u32) -> anyhow::Result<String> {
    let c = select_count(conn)?;
    let j = select(conn, limit, offset)?;
    let d = select_by_day(conn)?;
    let t = select_by_tag(conn)?;
    let msg = format!(
        r#"{{"count": {}, "notes":{}, "days": {}, "tags": {} }}"#,
        c, j, d, t
    );
    // eprintln!("msg {}", msg);
    Ok(msg)
}

pub fn do_filter(
    conn: &Connection,
    query: &str,
    limit: u32,
    offset: u32,
    from: &str,
    to: &str,
) -> anyhow::Result<String> {
    let c = filter_count(conn, query, from, to)?;
    let j = filter(conn, query, from, to, limit, offset)?;
    let d = search_by_day(conn, query)?;
    let t = filter_by_tag(conn, query, from, to)?;
    let msg = format!(
        r#"{{"count": {}, "notes":{},"days": {}, "tags": {} }}"#,
        c, j, d, t
    );
    // eprintln!("msg {}", msg);
    Ok(msg)
}
