use localnative_core::db::models::{CmdDelete, Note, QueryResult};
use localnative_core::db::{migrations, queries, sync};
use sqlx::SqlitePool;
use tracing::error;

pub async fn delete(
    pool: SqlitePool,
    query: String,
    limit: u32,
    offset: u32,
    rowid: i64,
) -> Option<QueryResult> {
    let delete_cmd = CmdDelete {
        query,
        rowid,
        limit,
        offset,
    };
    if let Err(e) = delete_cmd.process(&pool).await {
        error!("Error deleting note: {}", e);
        return None;
    }

    select_inner(&pool, delete_cmd.query, limit, offset).await
}

pub async fn upgrade(
    pool: SqlitePool,
    query: String,
    limit: u32,
    offset: u32,
) -> Option<QueryResult> {
    if let Err(e) = migrations::upgrade(&pool).await {
        error!("Error upgrading database: {}", e);
        return None;
    }

    select_inner(&pool, query, limit, offset).await
}

pub async fn insert(
    pool: &SqlitePool,
    query: String,
    limit: u32,
    offset: u32,
    note: Note,
) -> Option<QueryResult> {
    if let Err(e) = sync::insert(pool, note).await {
        error!("Error inserting note: {}", e);
        return None;
    }

    select_inner(pool, query, limit, offset).await
}

pub async fn select(
    pool: SqlitePool,
    query: String,
    limit: u32,
    offset: u32,
) -> Option<QueryResult> {
    select_inner(&pool, query, limit, offset).await
}

pub async fn filter(
    pool: SqlitePool,
    query: String,
    limit: u32,
    offset: u32,
    from: chrono::NaiveDate,
    to: chrono::NaiveDate,
) -> Option<QueryResult> {
    filter_inner(
        &pool,
        &query,
        limit,
        offset,
        &from.to_string(),
        &to.to_string(),
    )
    .await
}

pub async fn someday(
    pool: &SqlitePool,
    query: String,
    limit: u32,
    offset: u32,
    day: String,
) -> Option<QueryResult> {
    filter_inner(pool, &query, limit, offset, &day, &day).await
}

async fn select_inner(
    pool: &SqlitePool,
    query: String,
    limit: u32,
    offset: u32,
) -> Option<QueryResult> {
    match queries::do_search(pool, &query, limit, offset).await {
        Ok(search_result) => Some(search_result),
        Err(e) => {
            error!("Error searching notes: {}", e);
            None
        }
    }
}

async fn filter_inner(
    pool: &SqlitePool,
    query: &str,
    limit: u32,
    offset: u32,
    from: &str,
    to: &str,
) -> Option<QueryResult> {
    match queries::do_filter(pool, query, limit, offset, from, to).await {
        Ok(filter_result) => Some(filter_result),
        Err(e) => {
            error!("Error filtering notes: {}", e);
            None
        }
    }
}
