// db.rs
pub use error::{DbError, DbResult};
use models::Cmd;
use sqlx::sqlite::SqlitePool;
use tracing::debug;

pub async fn init_db() -> DbResult<SqlitePool> {
    let db_url = utils::sqlite3_db_location().await?;
    debug!("db_url: {db_url}");
    let pool = SqlitePool::connect(&db_url).await?;

    #[cfg(target_os = "android")]
    sqlx::query("PRAGMA temp_store_directory = '/data/data/app.localnative/cache'")
        .execute(&pool)
        .await?;

    migrations::upgrade(&pool).await?;

    Ok(pool)
}

pub async fn process_cmd(cmd: Cmd, pool: &SqlitePool) -> DbResult<String> {
    match cmd {
        Cmd::Insert(ref insert) => {
            insert.process(pool).await?;
            let select_result = queries::do_select(pool, insert.limit, insert.offset).await?;
            Ok(serde_json::to_string(&select_result)?)
        }
        Cmd::InsertImage(ref insert) => {
            insert.process_image(pool).await?;
            let select_result = queries::do_select(pool, insert.limit, insert.offset).await?;
            Ok(serde_json::to_string(&select_result)?)
        }
        Cmd::Delete(ref delete) => {
            delete.process(pool).await?;
            let search_result =
                queries::do_search(pool, &delete.query, delete.limit, delete.offset).await?;
            Ok(serde_json::to_string(&search_result)?)
        }
        Cmd::Select(ref select) => {
            let select_result = select.process(pool).await?;
            Ok(serde_json::to_string(&select_result)?)
        }
        Cmd::Search(ref search) => {
            let search_result = search.process(pool).await?;
            Ok(serde_json::to_string(&search_result)?)
        }
        Cmd::Filter(ref filter) => {
            let filter_result = filter.process(pool).await?;
            Ok(serde_json::to_string(&filter_result)?)
        }
        Cmd::Upgrade => {
            migrations::upgrade(pool).await?;
            Ok(serde_json::to_string(&"Upgrade completed")?)
        }
        Cmd::SyncViaAttach(ref sync) => {
            sync.process(pool).await?;
            Ok(serde_json::to_string(&"Sync via attach completed")?)
        }
    }
}

pub mod models {
    use chrono::{NaiveDate, NaiveDateTime};
    use serde::{Deserialize, Serialize};
    use sqlx::prelude::FromRow;

    #[derive(Serialize, Deserialize, Debug, FromRow, Clone, Default)]
    pub struct Day {
        pub date: NaiveDate,
        pub count: i64,
    }

    #[derive(Debug, Default, Deserialize, Serialize, Clone)]
    pub struct Tags {
        pub tag: String,
        pub count: i64,
    }

    #[derive(Serialize, Deserialize, Debug, FromRow, Default, Clone)]
    pub struct Note {
        pub rowid: i64,
        pub uuid4: String,
        pub title: String,
        pub url: String,
        pub tags: String,
        pub description: String,
        pub comments: String,
        pub annotations: Vec<u8>,
        pub created_at: NaiveDateTime,
        pub is_public: bool,
    }

    #[derive(Serialize, Deserialize, Debug)]
    #[serde(tag = "action", rename_all = "kebab-case")]
    pub enum Cmd {
        Insert(CmdInsert),
        InsertImage(CmdInsert),
        Delete(CmdDelete),
        Select(CmdSelect),
        Search(CmdSearch),
        Filter(CmdFilter),
        Upgrade,
        SyncViaAttach(CmdSyncViaAttach),
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub struct CmdInsert {
        pub title: String,
        pub url: String,
        pub tags: String,
        pub description: String,
        pub comments: String,
        pub annotations: String,
        pub limit: u32,
        pub offset: u32,
        pub is_public: bool,
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub struct CmdFilter {
        pub query: String,
        pub limit: u32,
        pub offset: u32,
        pub from: String,
        pub to: String,
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub struct CmdSearch {
        pub query: String,
        pub limit: u32,
        pub offset: u32,
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub struct CmdDelete {
        pub query: String,
        pub rowid: i64,
        pub limit: u32,
        pub offset: u32,
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub struct CmdSelect {
        pub limit: u32,
        pub offset: u32,
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub struct CmdSyncViaAttach {
        pub uri: String,
    }

    #[derive(Debug, Default, Deserialize, Serialize, Clone)]
    pub struct QueryResult {
        pub count: u32,
        pub notes: Vec<Note>,
        pub days: Vec<Day>,
        pub tags: Vec<Tags>,
    }
}

mod error {
    use thiserror::Error;

    #[derive(Error, Debug)]
    pub enum DbError {
        #[error("SQLx error: {0}")]
        SqlxError(#[from] sqlx::Error),
        #[error("Serialization error: {0}")]
        SerdeError(#[from] serde_json::Error),
        #[error("Base64 decoding error: {0}")]
        Base64Error(#[from] base64::DecodeError),
        #[error("Semver parsing error: {0}")]
        SemverError(#[from] semver::Error),
        #[error("Chrono parse error: {0}")]
        ChronoParseError(#[from] chrono::ParseError),
        #[error("Invalid created_at format")]
        InvalidFormat,
        #[error("IO error: {0}")]
        IoError(#[from] std::io::Error),
    }

    pub type DbResult<T> = Result<T, DbError>;
}

mod utils {
    use super::*;
    use tokio::fs;

    pub async fn sqlite3_db_location() -> DbResult<String> {
        let dir_name = if cfg!(target_os = "android") {
            "/data/data/app.localnative/files"
        } else if cfg!(target_os = "ios") {
            "Documents"
        } else {
            "LocalNative"
        };

        let home_dir = dirs::home_dir().ok_or(DbError::IoError(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Failed to get home directory",
        )))?;
        let dir = format!(
            "{}/{}",
            home_dir
                .to_str()
                .ok_or(DbError::IoError(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "Invalid home directory path",
                )))?,
            dir_name
        );
        eprintln!("db dir location: {}", dir);
        fs::create_dir_all(&dir).await?;
        Ok(format!("{}/localnative.sqlite3", dir))
    }
}

mod commands {
    use super::*;
    use base64::engine::general_purpose::STANDARD;
    use base64::Engine as _;
    use models::{
        CmdDelete, CmdFilter, CmdInsert, CmdSearch, CmdSelect, CmdSyncViaAttach, Note, QueryResult,
    };

    use sqlx::sqlite::SqlitePool;

    impl CmdFilter {
        pub async fn process(&self, pool: &SqlitePool) -> DbResult<QueryResult> {
            queries::do_filter(
                pool,
                &self.query,
                self.limit,
                self.offset,
                &self.from,
                &self.to,
            )
            .await
        }
    }

    impl CmdInsert {
        pub async fn process(&self, pool: &SqlitePool) -> DbResult<Note> {
            queries::insert_note(
                pool,
                &self.title,
                &self.url,
                &self.tags,
                &self.description,
                &self.comments,
                &self.annotations.as_bytes(),
                self.is_public,
            )
            .await
        }

        pub async fn process_image(&self, pool: &SqlitePool) -> DbResult<Note> {
            let data64 = self.annotations.replace("data:image/png;base64,", "");
            let decoded = STANDARD.decode(&data64)?;
            queries::insert_note(
                pool,
                &self.title,
                &self.url,
                &self.tags,
                &self.description,
                &self.comments,
                &decoded,
                self.is_public,
            )
            .await
        }
    }

    impl CmdDelete {
        pub async fn process(&self, pool: &SqlitePool) -> DbResult<()> {
            queries::delete_note(pool, self.rowid).await
        }
    }

    impl CmdSyncViaAttach {
        pub async fn process(&self, pool: &SqlitePool) -> DbResult<()> {
            queries::sync_via_attach(pool, &self.uri).await
        }
    }

    impl CmdSelect {
        pub async fn process(&self, pool: &SqlitePool) -> DbResult<QueryResult> {
            queries::do_select(pool, self.limit, self.offset).await
        }
    }

    impl CmdSearch {
        pub async fn process(&self, pool: &SqlitePool) -> DbResult<QueryResult> {
            queries::do_search(pool, &self.query, self.limit, self.offset).await
        }
    }
}

pub mod queries {
    use super::*;
    use chrono::Utc;
    use models::{Day, Note, QueryResult, Tags};
    use regex::Regex;
    use sqlx::sqlite::SqlitePool;
    use sqlx::{Executor as _, Row as _};
    use std::collections::HashMap;
    use tracing::{debug, error};
    use uuid::Uuid;

    pub async fn insert_note(
        pool: &SqlitePool,
        title: &str,
        url: &str,
        tags: &str,
        description: &str,
        comments: &str,
        annotations: &[u8],
        is_public: bool,
    ) -> DbResult<Note> {
        let uuid4 = Uuid::new_v4().to_string();
        let created_at = Utc::now().to_string();
        sqlx::query(
            "INSERT INTO note (uuid4, title, url, tags, description, comments, annotations, created_at, is_public)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(uuid4.clone())
        .bind(title)
        .bind(url)
        .bind(tags)
        .bind(description)
        .bind(comments)
        .bind(annotations)
        .bind(created_at.clone())
        .bind(is_public)
        .execute(pool)
        .await?;

        // Fetch the inserted note
        let note = sqlx::query_as::<_, Note>(
            "SELECT rowid, uuid4, title, url, tags, description, comments, annotations, created_at, is_public FROM note
            WHERE uuid4 = ?",
        )
        .bind(uuid4)
        .fetch_one(pool)
        .await?;

        Ok(note)
    }
    pub async fn delete_note(pool: &SqlitePool, rowid: i64) -> DbResult<()> {
        sqlx::query("DELETE FROM note WHERE rowid = ?")
            .bind(rowid)
            .execute(pool)
            .await?;
        Ok(())
    }

    pub async fn sync_via_attach(pool: &SqlitePool, uri: &str) -> DbResult<()> {
        let mut tx = pool.begin().await?;
        tx.execute(sqlx::query("ATTACH ? AS other").bind(uri))
            .await?;
        tx.execute(
            sqlx::query(
                "BEGIN;
                INSERT INTO main.note (uuid4, title, url, tags, description, comments, annotations, created_at, is_public)
                SELECT uuid4, title, url, tags, description, comments, annotations, created_at, is_public
                FROM other.note
                WHERE NOT EXISTS (
                    SELECT 1 FROM main.note
                    WHERE main.note.uuid4 = other.note.uuid4
                ) ORDER BY created_at;

                INSERT INTO other.note (uuid4, title, url, tags, description, comments, annotations, created_at, is_public)
                SELECT uuid4, title, url, tags, description, comments, annotations, created_at, is_public
                FROM main.note
                WHERE NOT EXISTS (
                    SELECT 1 FROM other.note
                    WHERE other.note.uuid4 = main.note.uuid4
                ) ORDER BY created_at;
                COMMIT;
                DETACH DATABASE other;",
            )
        )
        .await?;

        tx.commit().await?;
        Ok(())
    }

    pub async fn do_select(pool: &SqlitePool, limit: u32, offset: u32) -> DbResult<QueryResult> {
        let count = select_count(pool).await?;
        let notes = select(pool, limit, offset).await?;
        let days = select_by_day(pool).await?;
        let tags = select_by_tag(pool).await?;

        Ok(QueryResult {
            count,
            notes,
            days,
            tags,
        })
    }

    pub async fn do_search(
        pool: &SqlitePool,
        query: &str,
        limit: u32,
        offset: u32,
    ) -> DbResult<QueryResult> {
        let count = search_count(pool, query).await?;
        let notes = search(pool, query, limit, offset).await?;
        let days = search_by_day(pool, query).await?;
        let tags = search_by_tag(pool, query).await?;

        Ok(QueryResult {
            count,
            notes,
            days,
            tags,
        })
    }

    pub async fn do_filter(
        pool: &SqlitePool,
        query: &str,
        limit: u32,
        offset: u32,
        from: &str,
        to: &str,
    ) -> DbResult<QueryResult> {
        let count = filter_count(pool, query, from, to).await?;
        let notes = filter(pool, query, from, to, limit, offset).await?;
        let days = search_by_day(pool, query).await?;
        let tags = filter_by_tag(pool, query, from, to).await?;

        Ok(QueryResult {
            count,
            notes,
            days,
            tags,
        })
    }

    async fn select_count(pool: &SqlitePool) -> DbResult<u32> {
        debug!("Starting select_count");
        let count_result = sqlx::query_scalar("SELECT COUNT(1) FROM note")
            .fetch_one(pool)
            .await;

        match count_result {
            Ok(count) => {
                debug!("Fetched count: {}", count);
                Ok(count)
            }
            Err(e) => {
                error!("Error fetching count: {:?}", e);
                Err(DbError::from(e))
            }
        }
    }

    async fn select(pool: &SqlitePool, limit: u32, offset: u32) -> DbResult<Vec<Note>> {
        debug!("Starting select with limit: {}, offset: {}", limit, offset);
        let notes_result = sqlx::query_as::<_, Note>(
            "SELECT rowid, uuid4, title, url, tags, description, comments,
             hex(annotations) as annotations, created_at, is_public
             FROM note
             ORDER BY created_at DESC
             LIMIT ? OFFSET ?",
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await;

        match notes_result {
            Ok(notes) => {
                debug!("Fetched {} notes from the database", notes.len());
                Ok(notes)
            }
            Err(e) => {
                error!("Error fetching notes: {:?}", e);
                Err(DbError::from(e))
            }
        }
    }

    async fn select_by_day(pool: &SqlitePool) -> DbResult<Vec<Day>> {
        debug!("Starting select_by_day");
        let days_result = sqlx::query_as::<_, Day>(
            "SELECT DATE(substr(created_at, 1, 10)) as date, COUNT(1) as count
            FROM note
            GROUP BY date
            ORDER BY date",
        )
        .fetch_all(pool)
        .await;

        match days_result {
            Ok(days) => {
                debug!("Fetched {} days from the database", days.len());
                Ok(days)
            }
            Err(e) => {
                error!("Error fetching days: {:?}", e);
                Err(DbError::from(e))
            }
        }
    }

    async fn select_by_tag(pool: &SqlitePool) -> DbResult<Vec<Tags>> {
        debug!("Starting select_by_tag");
        let mut tag_count_map = HashMap::new();

        let tags_result = sqlx::query("SELECT tags FROM note")
            .map(|row: sqlx::sqlite::SqliteRow| row.get::<String, _>("tags"))
            .fetch_all(pool)
            .await;

        match tags_result {
            Ok(tags) => {
                debug!("Fetched {} tags from the database", tags.len());
                tags.into_iter()
                    .flat_map(|tags| tags.split(',').map(&str::to_lowercase).collect::<Vec<_>>())
                    .for_each(|tag| {
                        *tag_count_map.entry(tag).or_insert(0) += 1;
                    });
            }
            Err(e) => {
                error!("Error fetching tags: {:?}", e);
                return Err(DbError::from(e));
            }
        }

        let tags = tag_count_map
            .into_iter()
            .map(|(tag, count)| Tags { tag, count })
            .collect::<Vec<_>>();

        debug!("Completed select_by_tag with {} tags", tags.len());
        Ok(tags)
    }

    async fn search_count(pool: &SqlitePool, query: &str) -> DbResult<u32> {
        debug!("Starting search_count with query: {}", query);
        if query.is_empty() {
            debug!("Query is empty, falling back to select_count");
            return select_count(pool).await;
        }

        let words = make_words(query);
        let sql = format!(
            "SELECT COUNT(1)
            FROM note
            WHERE {}",
            where_clause(&words, 1)
        );

        let mut query = sqlx::query_scalar(&sql);

        for word in &words {
            query = query.bind(word);
        }

        let count_result = query.fetch_one(pool).await;

        match count_result {
            Ok(count) => {
                debug!("Fetched count: {}", count);
                Ok(count)
            }
            Err(e) => {
                error!("Error fetching count: {:?}", e);
                Err(DbError::from(e))
            }
        }
    }

    async fn search(
        pool: &SqlitePool,
        query: &str,
        limit: u32,
        offset: u32,
    ) -> DbResult<Vec<Note>> {
        debug!(
            "Starting search with query: {}, limit: {}, offset: {}",
            query, limit, offset
        );
        if query.is_empty() {
            debug!("Query is empty, falling back to select");
            return select(pool, limit, offset).await;
        }

        let words = make_words(query);
        let sql = format!(
            "SELECT rowid, uuid4, title, url, tags, description, comments,
             hex(annotations) as annotations, created_at, is_public
             FROM note
             WHERE {}
             ORDER BY created_at DESC
             LIMIT ?1 OFFSET ?2",
            where_clause(&words, 3),
        );

        debug!("{sql}");

        let mut query = sqlx::query_as::<_, Note>(&sql).bind(limit).bind(offset);

        for word in &words {
            query = query.bind(word);
        }

        let notes_result = query.fetch_all(pool).await;
        match notes_result {
            Ok(notes) => {
                debug!("Fetched {} notes from the database", notes.len());
                Ok(notes)
            }
            Err(e) => {
                error!("Error fetching notes: {:?}", e);
                Err(DbError::from(e))
            }
        }
    }

    async fn search_by_day(pool: &SqlitePool, query: &str) -> DbResult<Vec<Day>> {
        debug!("Starting search_by_day with query: {}", query);
        if query.is_empty() {
            debug!("Query is empty, falling back to select_by_day");
            return select_by_day(pool).await;
        }

        let words = make_words(query);
        let sql = format!(
            "SELECT DATE(substr(created_at, 1, 10)) as date, COUNT(1) as count
            FROM note
            WHERE {}
            GROUP BY date
            ORDER BY date",
            where_clause(&words, 1)
        );

        let mut query = sqlx::query_as::<_, Day>(&sql);

        for word in &words {
            query = query.bind(word);
        }

        let days_result = query.fetch_all(pool).await;

        match days_result {
            Ok(days) => {
                debug!("Fetched {} days from the database", days.len());
                Ok(days)
            }
            Err(e) => {
                error!("Error fetching days: {:?}", e);
                Err(DbError::from(e))
            }
        }
    }

    async fn search_by_tag(pool: &SqlitePool, query: &str) -> DbResult<Vec<Tags>> {
        debug!("Starting search_by_tag with query: {}", query);
        if query.is_empty() {
            debug!("Query is empty, falling back to select_by_tag");
            return select_by_tag(pool).await;
        }

        let words = make_words(query);
        let sql = format!(
            "SELECT tags
            FROM note
            WHERE {}",
            where_clause(&words, 1)
        );

        let mut tag_count_map = HashMap::new();

        let mut query = sqlx::query(&sql);

        for word in &words {
            query = query.bind(word);
        }

        let tags_result = query
            .map(|row: sqlx::sqlite::SqliteRow| row.get::<String, _>("tags"))
            .fetch_all(pool)
            .await;

        match tags_result {
            Ok(tags) => {
                debug!("Fetched {} tags from the database", tags.len());
                tags.into_iter()
                    .flat_map(|tags| tags.split(',').map(&str::to_lowercase).collect::<Vec<_>>())
                    .for_each(|tag| {
                        *tag_count_map.entry(tag).or_insert(0) += 1;
                    });
            }
            Err(e) => {
                error!("Error fetching tags: {:?}", e);
                return Err(DbError::from(e));
            }
        }

        let tags = tag_count_map
            .into_iter()
            .map(|(tag, count)| Tags { tag, count })
            .collect::<Vec<_>>();

        debug!("Completed search_by_tag with {} tags", tags.len());
        Ok(tags)
    }

    async fn filter_count(pool: &SqlitePool, query: &str, from: &str, to: &str) -> DbResult<u32> {
        debug!(
            "Starting filter_count with query: {}, from: {}, to: {}",
            query, from, to
        );
        let words = make_words(query);
        if words.len() == 1 && words[0].is_empty() {
            debug!("Query is empty, falling back to select_count");
            return select_count(pool).await;
        }

        let sql = format!(
            "SELECT COUNT(1)
            FROM note
            WHERE substr(created_at, 1, 10) >= ?1
            AND substr(created_at, 1, 10) <= ?2
            AND {}",
            where_clause(&words, 3)
        );

        let mut query = sqlx::query_scalar(&sql).bind(from).bind(to);

        for word in &words {
            query = query.bind(word);
        }

        let count_result = query.fetch_one(pool).await;

        match count_result {
            Ok(count) => {
                debug!("Fetched count: {}", count);
                Ok(count)
            }
            Err(e) => {
                error!("Error fetching count: {:?}", e);
                Err(DbError::from(e))
            }
        }
    }

    async fn filter(
        pool: &SqlitePool,
        query: &str,
        from: &str,
        to: &str,
        limit: u32,
        offset: u32,
    ) -> DbResult<Vec<Note>> {
        debug!(
            "Starting filter with query: {}, from: {}, to: {}, limit: {}, offset: {}",
            query, from, to, limit, offset
        );
        let words = make_words(query);
        if words.len() == 1 && words[0].is_empty() {
            debug!("Query is empty, falling back to select");
            return select(pool, limit, offset).await;
        }

        let sql = format!(
            "SELECT rowid, uuid4, title, url, tags, description, comments,
             hex(annotations) as annotations, created_at, is_public
             FROM note
             WHERE substr(created_at, 1, 10) >= ?1
             AND substr(created_at, 1, 10) <= ?2
             AND {}
             ORDER BY created_at DESC
             LIMIT ?3 OFFSET ?4",
            where_clause(&words, 5)
        );

        let mut query = sqlx::query_as::<_, Note>(&sql)
            .bind(from)
            .bind(to)
            .bind(limit)
            .bind(offset);

        for word in &words {
            query = query.bind(word);
        }

        let notes_result = query.fetch_all(pool).await;

        match notes_result {
            Ok(notes) => {
                debug!("Fetched {} notes from the database", notes.len());
                Ok(notes)
            }
            Err(e) => {
                error!("Error fetching notes: {:?}", e);
                Err(DbError::from(e))
            }
        }
    }

    async fn filter_by_tag(
        pool: &SqlitePool,
        query: &str,
        from: &str,
        to: &str,
    ) -> DbResult<Vec<Tags>> {
        debug!(
            "Starting filter_by_tag with query: {}, from: {}, to: {}",
            query, from, to
        );
        let words = make_words(query);

        if words.len() == 1 && words[0].is_empty() {
            debug!("Query is empty, falling back to select_by_tag");
            return select_by_tag(pool).await;
        }

        let sql = format!(
            "SELECT tags
            FROM note
            WHERE substr(created_at, 1, 10) >= ?1
            AND substr(created_at, 1, 10) <= ?2
            AND {}",
            where_clause(&words, 3)
        );

        let mut tag_count_map = HashMap::new();

        let mut query = sqlx::query(&sql).bind(from).bind(to);

        for word in &words {
            query = query.bind(word);
        }

        let tags_result = query
            .map(|row: sqlx::sqlite::SqliteRow| row.get::<String, _>("tags"))
            .fetch_all(pool)
            .await;

        match tags_result {
            Ok(tags) => {
                debug!("Fetched {} tags from the database", tags.len());
                tags.into_iter()
                    .flat_map(|tags| tags.split(',').map(&str::to_lowercase).collect::<Vec<_>>())
                    .for_each(|tag| {
                        *tag_count_map.entry(tag).or_insert(0) += 1;
                    });
            }
            Err(e) => {
                error!("Error fetching tags: {:?}", e);
                return Err(DbError::from(e));
            }
        }

        let tags = tag_count_map
            .into_iter()
            .map(|(tag, count)| Tags { tag, count })
            .collect::<Vec<_>>();

        debug!("Completed filter_by_tag with {} tags", tags.len());
        Ok(tags)
    }

    fn make_words(query: &str) -> Vec<String> {
        let re = Regex::new(r"\s+").unwrap();
        re.replace_all(query.trim(), " ")
            .split(' ')
            .map(|w| format!("%{}%", w))
            .collect()
    }

    fn where_clause(words: &[String], start: usize) -> String {
        words
            .iter()
            .enumerate()
            .map(|(i, _)| {
                format!(
                    "(title LIKE ?{0} OR url LIKE ?{0} OR tags LIKE ?{0} OR description LIKE ?{0})",
                    i + start
                )
            })
            .collect::<Vec<_>>()
            .join(" AND ")
    }
}
pub mod migrations {
    use super::*;
    use chrono::{NaiveDateTime, Timelike as _};
    use futures::future::BoxFuture;
    use models::Note;
    use semver::Version;
    use serde::{Deserialize, Serialize};
    use sqlx::{sqlite::SqlitePool, FromRow};
    use uuid::Uuid;

    const MIGRATIONS: &[(Version, fn(&SqlitePool) -> BoxFuture<'_, DbResult<()>>)] = &[
        (Version::new(0, 4, 0), |pool| Box::pin(migrate_schema(pool))),
        (Version::new(0, 4, 1), |pool| Box::pin(migrate_note(pool))),
        (Version::new(0, 5, 0), |pool| Box::pin(drop_ssb_table(pool))),
        (Version::new(0, 6, 0), |pool| {
            Box::pin(migrate_created_at(pool))
        }),
    ];

    pub async fn upgrade(pool: &SqlitePool) -> DbResult<()> {
        if !table_exists(pool).await? {
            return init_db_schema(pool).await;
        }

        let current_version = Version::parse(&get_meta_version(pool).await?)?;

        for (version, migrate) in MIGRATIONS {
            if current_version < *version {
                migrate(pool).await?;
                set_meta_version(pool, &version.to_string()).await?;
            }
        }

        Ok(())
    }

    async fn table_exists(pool: &SqlitePool) -> DbResult<bool> {
        let table_exists: bool = sqlx::query_scalar(
            "SELECT EXISTS (SELECT 1 FROM sqlite_master WHERE type='table' AND name='note')",
        )
        .fetch_one(pool)
        .await?;

        Ok(table_exists)
    }

    pub async fn get_meta_version(pool: &SqlitePool) -> DbResult<String> {
        let row: Option<String> =
            sqlx::query_scalar("SELECT meta_value FROM meta WHERE meta_key = 'version'")
                .fetch_optional(pool)
                .await?;
        Ok(row.unwrap_or_else(|| "0.3.10".to_string()))
    }

    async fn set_meta_version(pool: &SqlitePool, version: &str) -> DbResult<()> {
        sqlx::query("INSERT OR REPLACE INTO meta (meta_key, meta_value) VALUES ('version', ?)")
            .bind(version)
            .execute(pool)
            .await?;

        Ok(())
    }

    async fn init_db_schema(pool: &SqlitePool) -> DbResult<()> {
        sqlx::query(
            "CREATE TABLE note (
                 rowid INTEGER PRIMARY KEY AUTOINCREMENT,
                 uuid4 TEXT NOT NULL UNIQUE,
                 title TEXT NOT NULL,
                 url TEXT NOT NULL,
                 tags TEXT NOT NULL,
                 description TEXT NOT NULL,
                 comments TEXT NOT NULL,
                 annotations TEXT NOT NULL,
                 created_at TEXT NOT NULL,
                 is_public BOOLEAN NOT NULL DEFAULT 0
             );
             CREATE TABLE IF NOT EXISTS meta (
                 meta_key TEXT PRIMARY KEY,
                 meta_value TEXT NOT NULL
             );
             INSERT INTO meta (meta_key, meta_value) VALUES ('version', '0.5.1');",
        )
        .execute(pool)
        .await?;
        Ok(())
    }

    async fn migrate_schema(pool: &SqlitePool) -> DbResult<()> {
        sqlx::query(
            "ALTER TABLE note RENAME TO _note_0_3;
             CREATE TABLE note (
                 rowid INTEGER PRIMARY KEY AUTOINCREMENT,
                 uuid4 TEXT NOT NULL UNIQUE,
                 title TEXT NOT NULL,
                 url TEXT NOT NULL,
                 tags TEXT NOT NULL,
                 description TEXT NOT NULL,
                 comments TEXT NOT NULL,
                 annotations TEXT NOT NULL,
                 created_at TEXT NOT NULL,
                 is_public BOOLEAN NOT NULL DEFAULT 0
             );
             CREATE TABLE IF NOT EXISTS meta (
                 meta_key TEXT PRIMARY KEY,
                 meta_value TEXT NOT NULL
             );
             INSERT INTO meta (meta_key, meta_value) VALUES ('version', '0.4.0'), ('is_upgrading', '1');",
        )
        .execute(pool)
        .await?;
        Ok(())
    }

    async fn migrate_note(pool: &SqlitePool) -> DbResult<()> {
        let notes = sqlx::query_as::<_, Note>(
            "SELECT rowid, title, url, tags, description, comments, annotations, created_at, is_public FROM _note_0_3 ORDER BY rowid",
        )
        .fetch_all(pool)
        .await?;

        for note in notes {
            let uuid4 = Uuid::new_v4().to_string();
            sqlx::query(
                "INSERT INTO note (uuid4, title, url, tags, description, comments, annotations, created_at, is_public)
                 VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
            )
            .bind(uuid4)
            .bind(&note.title)
            .bind(&note.url)
            .bind(&note.tags)
            .bind(&note.description)
            .bind(&note.comments)
            .bind(&note.annotations)
            .bind(&note.created_at)
            .bind(note.is_public)
            .execute(pool)
            .await?;
        }

        sqlx::query(
            "DROP TABLE _note_0_3; UPDATE meta SET meta_value = '0' WHERE meta_key = 'is_upgrading';",
        )
        .execute(pool)
        .await?;
        Ok(())
    }

    async fn drop_ssb_table(pool: &SqlitePool) -> DbResult<()> {
        sqlx::query("DROP TABLE IF EXISTS ssb;")
            .execute(pool)
            .await?;
        Ok(())
    }

    async fn migrate_created_at(pool: &SqlitePool) -> DbResult<()> {
        #[derive(Serialize, Deserialize, Debug, FromRow, Default, Clone)]
        struct OldNote {
            pub rowid: i64,
            pub uuid4: String,
            pub title: String,
            pub url: String,
            pub tags: String,
            pub description: String,
            pub comments: String,
            pub annotations: String,
            pub created_at: String,
            pub is_public: bool,
        }

        let notes = sqlx::query_as::<_, OldNote>(
            "SELECT rowid, uuid4, title, url, tags, description, comments, annotations, created_at, is_public FROM note"
        )
        .fetch_all(pool)
        .await?;

        for note in notes {
            let created_at = parse_old_created_at(&note.created_at)?;
            sqlx::query("UPDATE note SET created_at = ? WHERE rowid = ?")
                .bind(created_at)
                .bind(note.rowid)
                .execute(pool)
                .await?;
        }

        Ok(())
    }

    fn parse_old_created_at(created_at: &str) -> DbResult<NaiveDateTime> {
        // 移除末尾的" UTC"
        let created_at = created_at.trim_end_matches(" UTC");

        // 分割日期时间部分和纳秒部分
        let parts: Vec<&str> = created_at.split(':').collect();
        if parts.len() != 4 {
            return Err(DbError::InvalidFormat);
        }

        // 解析日期时间部分
        let date_time_part = parts[..3].join(":");
        let naive_date_time = NaiveDateTime::parse_from_str(&date_time_part, "%Y-%m-%d %H:%M:%S")?;

        // 解析纳秒部分
        let nanos: u32 = parts[3].parse().map_err(|_| DbError::InvalidFormat)?;

        // 组合日期时间和纳秒
        Ok(naive_date_time
            .with_nanosecond(nanos)
            .unwrap_or(naive_date_time))
    }
}

pub mod sync {
    use super::*;
    use error::{DbError, DbResult};
    use models::Note;
    use sqlx::sqlite::SqlitePool;
    use std::collections::HashSet;

    pub async fn get_meta_version(pool: &SqlitePool) -> DbResult<String> {
        let row: Option<String> =
            sqlx::query_scalar("SELECT meta_value FROM meta WHERE meta_key = 'version'")
                .fetch_optional(pool)
                .await?;
        Ok(row.unwrap_or_else(|| "0.3.10".to_string()))
    }

    pub async fn get_note_by_uuid4(pool: &SqlitePool, uuid4: &str) -> DbResult<Note> {
        sqlx::query_as::<_, Note>(
            "SELECT rowid, uuid4, title, url, tags, description, comments, annotations, created_at, is_public FROM note WHERE uuid4 = ?"
        )
        .bind(uuid4)
        .fetch_one(pool)
        .await
        .map_err(DbError::from)
    }

    pub async fn next_uuid4_candidates(pool: &SqlitePool) -> DbResult<Vec<String>> {
        sqlx::query_scalar("SELECT uuid4 FROM note ORDER BY rowid")
            .fetch_all(pool)
            .await
            .map_err(DbError::from)
    }

    // to server
    pub async fn diff_uuid4_to_server(
        pool: &SqlitePool,
        candidates: Vec<String>,
    ) -> DbResult<Vec<String>> {
        let mut r = Vec::new();
        for uuid4 in candidates {
            let exists: bool =
                sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM note WHERE uuid4 = ?)")
                    .bind(&uuid4)
                    .fetch_one(pool)
                    .await
                    .map_err(DbError::from)?;
            if !exists {
                r.push(uuid4);
            }
        }
        Ok(r)
    }

    // from server
    pub async fn diff_uuid4_from_server(
        pool: &SqlitePool,
        candidates: Vec<String>,
    ) -> DbResult<Vec<String>> {
        let candidates: HashSet<_> = candidates.iter().collect();
        let mut r = Vec::new();
        let uuid4s = sqlx::query_scalar("SELECT uuid4 FROM note")
            .fetch_all(pool)
            .await
            .map_err(DbError::from)?;
        for uuid4 in uuid4s {
            if !candidates.contains(&uuid4) {
                r.push(uuid4);
            }
        }
        Ok(r)
    }

    pub async fn insert(pool: &SqlitePool, note: Note) -> DbResult<()> {
        sqlx::query(
            "INSERT INTO note (uuid4, title, url, tags, description, comments, annotations, created_at, is_public)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&note.uuid4)
        .bind(&note.title)
        .bind(&note.url)
        .bind(&note.tags)
        .bind(&note.description)
        .bind(&note.comments)
        .bind(&note.annotations)
        .bind(&note.created_at)
        .bind(note.is_public)
        .execute(pool)
        .await?;

        Ok(())
    }
}
