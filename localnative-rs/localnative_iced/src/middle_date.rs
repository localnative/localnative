use crate::{days::Day, tags::Tag};
use localnative_core::db::{self, CmdDelete, Note};
use sqlx::SqlitePool;

use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
pub struct MiddleDate {
    pub count: u32,
    pub notes: Vec<Note>,
    pub days: Option<Vec<Day>>,
    pub tags: Vec<Tag>,
}

impl MiddleDate {
    pub async fn delete(
        pool: &SqlitePool,
        query: String,
        limit: u32,
        offset: u32,
        rowid: i64,
    ) -> Option<Self> {
        let delete = CmdDelete {
            query,
            rowid,
            limit,
            offset,
        };
        if let Err(e) = delete.process(pool).await {
            eprintln!("Error deleting note: {}", e);
            return None;
        }

        Self::from_select_inner(pool, delete.query, limit, offset).await
    }

    pub async fn upgrade(
        pool: &SqlitePool,
        query: String,
        limit: u32,
        offset: u32,
    ) -> Option<Self> {
        if let Err(e) = db::upgrade(pool).await {
            eprintln!("Error upgrading database: {}", e);
            return None;
        }

        Self::from_select_inner(pool, query, limit, offset).await
    }

    pub async fn insert(
        pool: &SqlitePool,
        query: String,
        limit: u32,
        offset: u32,
        note: Note,
    ) -> Option<Self> {
        if let Err(e) = db::insert(pool, note).await {
            eprintln!("Error inserting note: {}", e);
            return None;
        }

        Self::from_select_inner(pool, query, limit, offset).await
    }

    pub async fn from_select(
        pool: &SqlitePool,
        query: String,
        limit: u32,
        offset: u32,
    ) -> Option<Self> {
        Self::from_select_inner(pool, query, limit, offset).await
    }

    pub async fn from_filter(
        pool: &SqlitePool,
        query: String,
        limit: u32,
        offset: u32,
        from: String,
        to: String,
    ) -> Option<Self> {
        Self::from_filter_inner(pool, &query, limit, offset, &from, &to).await
    }

    pub async fn from_someday(
        pool: &SqlitePool,
        query: String,
        limit: u32,
        offset: u32,
        day: String,
    ) -> Option<Self> {
        Self::from_filter_inner(pool, &query, limit, offset, &day, &day).await
    }

    async fn from_select_inner(
        pool: &SqlitePool,
        query: String,
        limit: u32,
        offset: u32,
    ) -> Option<Self> {
        match db::do_search(pool, &query, limit, offset).await {
            Ok(search_result) => serde_json::from_str::<Self>(&search_result).ok(),
            Err(e) => {
                eprintln!("Error searching notes: {}", e);
                None
            }
        }
    }

    async fn from_filter_inner(
        pool: &SqlitePool,
        query: &str,
        limit: u32,
        offset: u32,
        from: &str,
        to: &str,
    ) -> Option<Self> {
        match db::do_filter(pool, query, limit, offset, from, to).await {
            Ok(filter_result) => serde_json::from_str::<Self>(&filter_result).ok(),
            Err(e) => {
                eprintln!("Error filtering notes: {}", e);
                None
            }
        }
    }
}
