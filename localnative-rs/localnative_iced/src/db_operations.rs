use localnative_core::db::models::{CmdDelete, Note, QueryResult};
use localnative_core::db::{migrations, queries, sync};
use rusqlite::Connection;
use std::sync::{Arc, Mutex};

pub async fn delete(
    pool: Arc<Mutex<Connection>>,
    query: String,
    limit: u32,
    offset: u32,
    rowid: i64,
) -> Option<QueryResult> {
    tokio::task::spawn_blocking(move || {
        let conn = pool.lock().unwrap();
        let delete_cmd = CmdDelete {
            query: query.clone(),
            rowid,
            limit,
            offset,
        };
        if let Err(e) = delete_cmd.process(&conn) {
            eprintln!("Error deleting note: {}", e);
            return None;
        }
        select_inner(&conn, query, limit, offset)
    })
    .await
    .unwrap_or(None)
}

pub async fn upgrade(
    pool: Arc<Mutex<Connection>>,
    query: String,
    limit: u32,
    offset: u32,
) -> Option<QueryResult> {
    tokio::task::spawn_blocking(move || {
        let conn = pool.lock().unwrap();
        if let Err(e) = migrations::upgrade(&conn) {
            eprintln!("Error upgrading database: {}", e);
            return None;
        }
        select_inner(&conn, query, limit, offset)
    })
    .await
    .unwrap_or(None)
}

pub async fn insert(
    pool: Arc<Mutex<Connection>>,
    query: String,
    limit: u32,
    offset: u32,
    note: Note,
) -> Option<QueryResult> {
    tokio::task::spawn_blocking(move || {
        let conn = pool.lock().unwrap();
        if let Err(e) = sync::insert(&conn, &note) {
            eprintln!("Error inserting note: {}", e);
            return None;
        }
        select_inner(&conn, query, limit, offset)
    })
    .await
    .unwrap_or(None)
}

pub async fn select(
    pool: Arc<Mutex<Connection>>,
    query: String,
    limit: u32,
    offset: u32,
) -> Option<QueryResult> {
    tokio::task::spawn_blocking(move || {
        let conn = pool.lock().unwrap();
        select_inner(&conn, query, limit, offset)
    })
    .await
    .unwrap_or(None)
}

pub async fn filter(
    pool: Arc<Mutex<Connection>>,
    query: String,
    limit: u32,
    offset: u32,
    from: chrono::NaiveDate,
    to: chrono::NaiveDate,
) -> Option<QueryResult> {
    tokio::task::spawn_blocking(move || {
        let conn = pool.lock().unwrap();
        filter_inner(
            &conn,
            &query,
            limit,
            offset,
            &from.to_string(),
            &to.to_string(),
        )
    })
    .await
    .unwrap_or(None)
}

pub async fn someday(
    pool: Arc<Mutex<Connection>>,
    query: String,
    limit: u32,
    offset: u32,
    day: String,
) -> Option<QueryResult> {
    tokio::task::spawn_blocking(move || {
        let conn = pool.lock().unwrap();
        filter_inner(&conn, &query, limit, offset, &day, &day)
    })
    .await
    .unwrap_or(None)
}

fn select_inner(
    conn: &Connection,
    query: String,
    limit: u32,
    offset: u32,
) -> Option<QueryResult> {
    match queries::do_search(conn, &query, limit, offset) {
        Ok(search_result) => Some(search_result),
        Err(e) => {
            eprintln!("Error searching notes: {}", e);
            None
        }
    }
}

fn filter_inner(
    conn: &Connection,
    query: &str,
    limit: u32,
    offset: u32,
    from: &str,
    to: &str,
) -> Option<QueryResult> {
    match queries::do_filter(conn, query, limit, offset, from, to) {
        Ok(filter_result) => Some(filter_result),
        Err(e) => {
            eprintln!("Error filtering notes: {}", e);
            None
        }
    }
}
