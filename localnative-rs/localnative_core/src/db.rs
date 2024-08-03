use base64::Engine;
use chrono::Utc;
use futures::future::BoxFuture;
use regex::Regex;
use semver::Version;
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use sqlx::sqlite::SqlitePool;
use sqlx::Executor as _;
use sqlx::Row as _;
use std::collections::HashMap;
use std::collections::HashSet;
use thiserror::Error;
use tokio::fs;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, FromRow)]
pub struct KVStringI64 {
    pub k: String,
    pub v: i64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Tags {
    pub tag: String,
    pub count: i64,
}

#[derive(Serialize, Deserialize, Debug, FromRow, Clone)]
pub struct Note {
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
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

pub type DbResult<T> = Result<T, DbError>;

async fn sqlite3_db_location() -> DbResult<String> {
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

pub async fn init_db() -> DbResult<SqlitePool> {
    let db_url = sqlite3_db_location().await?;
    let pool = SqlitePool::connect(&db_url).await?;

    #[cfg(target_os = "android")]
    sqlx::query("PRAGMA temp_store_directory = '/data/data/app.localnative/cache'")
        .execute(&pool)
        .await?;

    upgrade(&pool).await?;

    Ok(pool)
}

impl CmdInsert {
    pub async fn process(&self, pool: &SqlitePool) -> DbResult<Note> {
        insert_note(pool, self, &self.annotations.as_bytes()).await
    }
}

impl CmdDelete {
    pub async fn process(&self, pool: &SqlitePool) -> DbResult<()> {
        sqlx::query("DELETE FROM note WHERE rowid = ?")
            .bind(self.rowid)
            .execute(pool)
            .await?;
        Ok(())
    }
}

impl CmdSelect {
    pub async fn process(&self, pool: &SqlitePool) -> DbResult<QueryResult> {
        do_select(pool, self.limit, self.offset).await
    }
}

impl CmdSearch {
    pub async fn process(&self, pool: &SqlitePool) -> DbResult<QueryResult> {
        do_search(pool, &self.query, self.limit, self.offset).await
    }
}

impl CmdFilter {
    pub async fn process(&self, pool: &SqlitePool) -> DbResult<QueryResult> {
        do_filter(
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

impl CmdSyncViaAttach {
    pub async fn process(&self, pool: &SqlitePool) -> DbResult<()> {
        let mut tx = pool.begin().await?;
        tx.execute(sqlx::query("ATTACH ? AS other").bind(&self.uri))
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
}

pub async fn process_cmd(cmd: Cmd, pool: &SqlitePool) -> DbResult<String> {
    match cmd {
        Cmd::Insert(ref insert) => {
            insert.process(&pool).await?;
            let select_result = do_select(&pool, insert.limit, insert.offset).await?;
            Ok(serde_json::to_string(&select_result)?)
        }
        Cmd::InsertImage(ref insert) => {
            insert_image(&pool, insert).await?;
            let select_result = do_select(&pool, insert.limit, insert.offset).await?;
            Ok(serde_json::to_string(&select_result)?)
        }
        Cmd::Delete(ref delete) => {
            delete.process(&pool).await?;
            let search_result =
                do_search(&pool, &delete.query, delete.limit, delete.offset).await?;
            Ok(serde_json::to_string(&search_result)?)
        }
        Cmd::Select(ref select) => {
            let select_result = select.process(&pool).await?;
            Ok(serde_json::to_string(&select_result)?)
        }
        Cmd::Search(ref search) => {
            let search_result = search.process(&pool).await?;
            Ok(serde_json::to_string(&search_result)?)
        }
        Cmd::Filter(ref filter) => {
            let filter_result = filter.process(&pool).await?;
            Ok(serde_json::to_string(&filter_result)?)
        }
        Cmd::Upgrade => {
            upgrade(&pool).await?;
            Ok(serde_json::to_string(&"Upgrade completed")?)
        }
        Cmd::SyncViaAttach(ref sync) => {
            sync.process(&pool).await?;
            Ok(serde_json::to_string(&"Sync via attach completed")?)
        }
    }
}

async fn insert_image(pool: &SqlitePool, insert: &CmdInsert) -> DbResult<Note> {
    let data64 = insert.annotations.replace("data:image/png;base64,", "");
    let decoded = base64::engine::general_purpose::STANDARD.decode(&data64)?;
    insert_note(pool, insert, &decoded).await
}

async fn insert_note(pool: &SqlitePool, insert: &CmdInsert, annotations: &[u8]) -> DbResult<Note> {
    let uuid4 = Uuid::new_v4().to_string();
    let created_at = Utc::now().to_string();
    sqlx::query(
        "INSERT INTO note (uuid4, title, url, tags, description, comments, annotations, created_at, is_public)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(uuid4.clone())
    .bind(&insert.title)
    .bind(&insert.url)
    .bind(&insert.tags)
    .bind(&insert.description)
    .bind(&insert.comments)
    .bind(annotations)
    .bind(created_at.clone())
    .bind(insert.is_public)
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

const MIGRATIONS: &[(Version, fn(&SqlitePool) -> BoxFuture<'_, DbResult<()>>)] = &[
    (Version::new(0, 4, 0), |pool| Box::pin(migrate_schema(pool))),
    (Version::new(0, 4, 1), |pool| Box::pin(migrate_note(pool))),
    (Version::new(0, 5, 0), |pool| Box::pin(drop_ssb_table(pool))),
    (Version::new(0, 6, 0), |_| Box::pin(async { Ok(()) })),
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

fn make_words(query: &str) -> Vec<String> {
    let re = Regex::new(r"\s+").unwrap();
    re.replace_all(query.trim(), " ")
        .split(' ')
        .map(|w| format!("%{}%", w))
        .collect()
}

fn where_clause(words: &[String]) -> String {
    words
        .iter()
        .enumerate()
        .map(|(i, _)| {
            format!(
                "(title LIKE ?{} OR url LIKE ?{} OR tags LIKE ?{} OR description LIKE ?{})",
                i + 1,
                i + 1,
                i + 1,
                i + 1
            )
        })
        .collect::<Vec<_>>()
        .join(" AND ")
}

pub async fn filter_by_tag(
    pool: &SqlitePool,
    query: &str,
    from: &str,
    to: &str,
) -> DbResult<Vec<Tags>> {
    let words = make_words(query);
    if words.len() == 1 && words[0].is_empty() {
        return select_by_tag(pool).await;
    }

    let sql = format!(
        "SELECT tags
        FROM note
        WHERE substr(created_at, 1, 10) >= ?
        AND substr(created_at, 1, 10) <= ?
        AND {}",
        where_clause(&words)
    );

    let mut tag_count_map = HashMap::new();

    let mut query = sqlx::query(&sql).bind(from).bind(to);

    for word in &words {
        query = query.bind(word);
    }

    query
        .map(|row: sqlx::sqlite::SqliteRow| row.get::<String, _>("tags"))
        .fetch_all(pool)
        .await?
        .into_iter()
        .flat_map(|tags| tags.split(',').map(&str::to_lowercase).collect::<Vec<_>>())
        .for_each(|tag| {
            *tag_count_map.entry(tag).or_insert(0) += 1;
        });

    let tags = tag_count_map
        .into_iter()
        .map(|(tag, count)| Tags { tag, count })
        .collect();

    Ok(tags)
}

pub async fn filter_count(pool: &SqlitePool, query: &str, from: &str, to: &str) -> DbResult<u32> {
    let words = make_words(query);
    if words.len() == 1 && words[0].is_empty() {
        return select_count(pool).await;
    }

    let sql = format!(
        "SELECT COUNT(1)
        FROM note
        WHERE substr(created_at, 1, 10) >= ?
        AND substr(created_at, 1, 10) <= ?
        AND {}",
        where_clause(&words)
    );

    let mut query = sqlx::query_scalar(&sql).bind(from).bind(to);

    for word in &words {
        query = query.bind(word);
    }

    let count = query.fetch_one(pool).await?;

    Ok(count)
}

pub async fn filter(
    pool: &SqlitePool,
    query: &str,
    from: &str,
    to: &str,
    limit: u32,
    offset: u32,
) -> DbResult<Vec<Note>> {
    let words = make_words(query);
    if words.len() == 1 && words[0].is_empty() {
        return select(pool, limit, offset).await;
    }

    let sql = format!(
        "SELECT rowid, uuid4, title, url, tags, description, comments,
         hex(annotations) as annotations, created_at, is_public
         FROM note
         WHERE substr(created_at, 1, 10) >= ?
         AND substr(created_at, 1, 10) <= ?
         AND {}
         ORDER BY created_at DESC
         LIMIT ? OFFSET ?",
        where_clause(&words)
    );

    let mut query = sqlx::query_as::<_, Note>(&sql).bind(from).bind(to);

    for word in &words {
        query = query.bind(word);
    }

    let notes = query.bind(limit).bind(offset).fetch_all(pool).await?;

    Ok(notes)
}

pub async fn select_by_day(pool: &SqlitePool) -> DbResult<Vec<KVStringI64>> {
    let days = sqlx::query_as::<_, KVStringI64>(
        "SELECT substr(created_at, 1, 10) as k, COUNT(1) as v
        FROM note
        GROUP BY k
        ORDER BY k",
    )
    .fetch_all(pool)
    .await?;

    Ok(days)
}

pub async fn select_by_tag(pool: &SqlitePool) -> DbResult<Vec<Tags>> {
    let mut tag_count_map = HashMap::new();

    sqlx::query("SELECT tags FROM note")
        .map(|row: sqlx::sqlite::SqliteRow| row.get::<String, _>("tags"))
        .fetch_all(pool)
        .await?
        .into_iter()
        .flat_map(|tags| tags.split(',').map(&str::to_lowercase).collect::<Vec<_>>())
        .for_each(|tag| {
            *tag_count_map.entry(tag).or_insert(0) += 1;
        });

    let tags = tag_count_map
        .into_iter()
        .map(|(tag, count)| Tags { tag, count })
        .collect();

    Ok(tags)
}

pub async fn select_count(pool: &SqlitePool) -> DbResult<u32> {
    let count = sqlx::query_scalar("SELECT COUNT(1) FROM note")
        .fetch_one(pool)
        .await?;

    Ok(count)
}

pub async fn select(pool: &SqlitePool, limit: u32, offset: u32) -> DbResult<Vec<Note>> {
    let notes = sqlx::query_as::<_, Note>(
        "SELECT rowid, uuid4, title, url, tags, description, comments,
         hex(annotations) as annotations, created_at, is_public
         FROM note
         ORDER BY created_at DESC
         LIMIT ? OFFSET ?",
    )
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await?;

    Ok(notes)
}

pub async fn search_by_tag(pool: &SqlitePool, query: &str) -> DbResult<Vec<Tags>> {
    if query.is_empty() {
        return select_by_tag(pool).await;
    }

    let words = make_words(query);
    let sql = format!(
        "SELECT tags
        FROM note
        WHERE {}",
        where_clause(&words)
    );

    let mut tag_count_map = HashMap::new();

    let mut query = sqlx::query(&sql);

    for word in &words {
        query = query.bind(word);
    }

    query
        .map(|row: sqlx::sqlite::SqliteRow| row.get::<String, _>("tags"))
        .fetch_all(pool)
        .await?
        .into_iter()
        .flat_map(|tags| tags.split(',').map(&str::to_lowercase).collect::<Vec<_>>())
        .for_each(|tag| {
            *tag_count_map.entry(tag).or_insert(0) += 1;
        });

    let tags = tag_count_map
        .into_iter()
        .map(|(tag, count)| Tags { tag, count })
        .collect();

    Ok(tags)
}

pub async fn search_by_day(pool: &SqlitePool, query: &str) -> DbResult<Vec<KVStringI64>> {
    if query.is_empty() {
        return select_by_day(pool).await;
    }

    let words = make_words(query);
    let sql = format!(
        "SELECT substr(created_at, 1, 10) as dt, COUNT(1) as n
        FROM note
        WHERE {}
        GROUP BY dt
        ORDER BY dt",
        where_clause(&words)
    );

    let mut query = sqlx::query_as::<_, KVStringI64>(&sql);

    for word in &words {
        query = query.bind(word);
    }

    let days = query.fetch_all(pool).await?;

    Ok(days)
}

pub async fn search_count(pool: &SqlitePool, query: &str) -> DbResult<u32> {
    if query.is_empty() {
        return select_count(pool).await;
    }

    let words = make_words(query);
    let sql = format!(
        "SELECT COUNT(1)
        FROM note
        WHERE {}",
        where_clause(&words)
    );

    let mut query = sqlx::query_scalar(&sql);

    for word in &words {
        query = query.bind(word);
    }

    let count = query.fetch_one(pool).await?;

    Ok(count)
}

pub async fn search(
    pool: &SqlitePool,
    query: &str,
    limit: u32,
    offset: u32,
) -> DbResult<Vec<Note>> {
    if query.is_empty() {
        return select(pool, limit, offset).await;
    }

    let words = make_words(query);
    let sql = format!(
        "SELECT rowid, uuid4, title, url, tags, description, comments,
         hex(annotations) as annotations, created_at, is_public
         FROM note
         WHERE {}
         ORDER BY created_at DESC
         LIMIT ? OFFSET ?",
        where_clause(&words)
    );

    let mut query = sqlx::query_as::<_, Note>(&sql);

    for word in &words {
        query = query.bind(word);
    }

    let notes = query.bind(limit).bind(offset).fetch_all(pool).await?;

    Ok(notes)
}

#[derive(Serialize)]
pub struct QueryResult {
    pub count: u32,
    pub notes: Vec<Note>,
    pub days: Vec<KVStringI64>,
    pub tags: Vec<Tags>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    #[test]
    fn test_query_result_serialization() {
        let query_result = QueryResult {
            count: 1,
            notes: vec![Note {
                rowid: 1,
                uuid4: "uuid4".to_string(),
                title: "title".to_string(),
                url: "url".to_string(),
                tags: "tags".to_string(),
                description: "description".to_string(),
                comments: "comments".to_string(),
                annotations: "annotations".to_string(),
                created_at: "created_at".to_string(),
                is_public: true,
            }],
            days: vec![KVStringI64 {
                k: "k".to_string(),
                v: 1,
            }],
            tags: vec![
                Tags {
                    tag: "hi".to_string(),
                    count: 10,
                },
                Tags {
                    tag: "hiss".to_string(),
                    count: 10,
                },
                Tags {
                    tag: "hidad".to_string(),
                    count: 1330,
                },
            ],
        };

        let serialized = serde_json::to_string(&query_result).unwrap();
        println!("{}", serialized);
    }
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
        let exists: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM note WHERE uuid4 = ?)")
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
