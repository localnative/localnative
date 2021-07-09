use localnative_core::{
    cmd::{create, delete, insert},
    rusqlite::Connection,
    Note,
};
use serde::{Deserialize, Serialize};

use crate::{days::Day, tags::Tag, Conn};

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
pub struct MiddleDate {
    pub count: u32,
    pub notes: Vec<Note>,
    pub days: Option<Vec<Day>>,
    pub tags: Vec<Tag>,
}

impl MiddleDate {
    pub async fn delete(
        conn: Conn,
        query: String,
        limit: u32,
        offset: u32,
        rowid: i64,
    ) -> Option<Self> {
        let conn = &*conn.lock().await;
        delete(conn, rowid);
        Self::from_select_inner(conn, query, limit, offset)
    }
    pub async fn upgrade(
        conn: Conn,
        query: String,
        limit: u32,
        offset: u32,
        is_created_db: bool,
    ) -> Option<Self> {
        let conn = &*conn.lock().await;
        if !is_created_db {
            create(conn);
        }
        if let Ok(version) = localnative_core::upgrade::upgrade(conn) {
            println!("upgrade done:{}", version);
        } else {
            println!("upgrade error");
        }
        Self::from_select_inner(conn, query, limit, offset)
    }
    pub async fn insert(
        conn: Conn,
        query: String,
        limit: u32,
        offset: u32,
        rowid: i64,
        note: Note,
    ) -> Option<Self> {
        let conn = &*conn.lock().await;
        delete(conn, rowid);
        insert(note);
        Self::from_select_inner(conn, query, limit, offset)
    }
    pub async fn from_select(conn: Conn, query: String, limit: u32, offset: u32) -> Option<Self> {
        let conn = &*conn.lock().await;
        Self::from_select_inner(conn, query, limit, offset)
    }
    pub async fn from_filter(
        conn: Conn,
        query: String,
        limit: u32,
        offset: u32,
        from: time::Date,
        to: time::Date,
    ) -> Option<Self> {
        let conn = &*conn.lock().await;
        let from = from.to_string();
        let to = to.to_string();
        Self::from_filter_inner(conn, &query, &limit, &offset, &from, &to)
    }
    pub async fn from_someday(
        conn: Conn,
        query: String,
        limit: u32,
        offset: u32,
        day: time::Date,
    ) -> Option<Self> {
        let conn = &*conn.lock().await;
        let day = day.to_string();
        Self::from_filter_inner(conn, &query, &limit, &offset, &day, &day)
    }
    fn from_select_inner(
        conn: &Connection,
        query: String,
        limit: u32,
        offset: u32,
    ) -> Option<Self> {
        let search_result = localnative_core::exe::do_search(conn, &query, &limit, &offset);

        serde_json::from_str::<Self>(&search_result).ok()
    }
    fn from_filter_inner(
        conn: &Connection,
        query: &str,
        limit: &u32,
        offset: &u32,
        from: &str,
        to: &str,
    ) -> Option<Self> {
        let filter_result = localnative_core::exe::do_filter(conn, query, limit, offset, from, to);

        serde_json::from_str::<Self>(&filter_result).ok()
    }
}
