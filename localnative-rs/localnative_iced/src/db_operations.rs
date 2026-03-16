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
        let conn = pool.lock().unwrap_or_else(|e| e.into_inner());
        let delete_cmd = CmdDelete {
            query: query.clone(),
            rowid,
            limit,
            offset,
        };
        if let Err(e) = delete_cmd.process(&conn) {
            tracing::error!(%e, "failed to delete note");
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
        let conn = pool.lock().unwrap_or_else(|e| e.into_inner());
        if let Err(e) = migrations::upgrade(&conn) {
            tracing::error!(%e, "failed to upgrade database");
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
        let conn = pool.lock().unwrap_or_else(|e| e.into_inner());
        if let Err(e) = sync::insert(&conn, &note) {
            tracing::error!(%e, "failed to insert note");
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
        let conn = pool.lock().unwrap_or_else(|e| e.into_inner());
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
        let conn = pool.lock().unwrap_or_else(|e| e.into_inner());
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
        let conn = pool.lock().unwrap_or_else(|e| e.into_inner());
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
            tracing::error!(%e, "failed to search notes");
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
            tracing::error!(%e, "failed to filter notes");
            None
        }
    }
}
