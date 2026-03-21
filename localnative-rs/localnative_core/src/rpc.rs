use crate::db::{
    models::Note,
    sync::{
        diff_uuid4_from_server, diff_uuid4_to_server, get_meta_version, get_note_by_uuid4, insert,
        next_uuid4_candidates,
    },
    DbError,
};
use futures::{future, FutureExt, StreamExt};
use governor::{Quota, RateLimiter};
use rusqlite::Connection;
use std::net::SocketAddr;
use std::num::NonZeroU32;
use std::sync::{Arc, Mutex};
use tarpc::client;
use tarpc::server::incoming::Incoming as _;
use tarpc::server::Channel as _;
use tarpc::{context, serde_transport::tcp, server::BaseChannel};
use thiserror::Error;
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum RpcError {
    #[error("Database error: {0}")]
    DbError(#[from] DbError),
    #[error("RPC error: {0}")]
    RpcError(#[from] tarpc::client::RpcError),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Address parse error: {0}")]
    AddrParseError(#[from] std::net::AddrParseError),
    #[error("Version mismatch")]
    VersionMismatch,
    #[error("Rpc error (serialized): {0}")]
    SerializedErr(String),
    #[error("Input validation error: {0}")]
    InputValidation(String),
    #[error("Server configuration error: {0}")]
    ServerConfigError(String),
    #[error("Rate limited: too many requests")]
    RateLimited,
}

/// Maximum allowed size for individual note text fields (1 MB).
const MAX_NOTE_FIELD_SIZE: usize = 1_048_576;
/// Maximum allowed size for note annotations field (10 MB).
const MAX_ANNOTATION_SIZE: usize = 10_485_760;

fn validate_uuid4(uuid4: &str) -> Result<(), RpcError> {
    Uuid::parse_str(uuid4)
        .map_err(|_| RpcError::InputValidation("Invalid UUID4 format".to_string()))?;
    Ok(())
}

fn validate_note(note: &Note) -> Result<(), RpcError> {
    validate_uuid4(&note.uuid4)?;
    if note.title.len() > MAX_NOTE_FIELD_SIZE {
        return Err(RpcError::InputValidation(
            "Title exceeds maximum size".to_string(),
        ));
    }
    if note.url.len() > MAX_NOTE_FIELD_SIZE {
        return Err(RpcError::InputValidation(
            "URL exceeds maximum size".to_string(),
        ));
    }
    if note.tags.len() > MAX_NOTE_FIELD_SIZE {
        return Err(RpcError::InputValidation(
            "Tags field exceeds maximum size".to_string(),
        ));
    }
    if note.description.len() > MAX_NOTE_FIELD_SIZE {
        return Err(RpcError::InputValidation(
            "Description field exceeds maximum size".to_string(),
        ));
    }
    if note.comments.len() > MAX_NOTE_FIELD_SIZE {
        return Err(RpcError::InputValidation(
            "Comments field exceeds maximum size".to_string(),
        ));
    }
    if note.annotations.len() > MAX_ANNOTATION_SIZE {
        return Err(RpcError::InputValidation(
            "Annotations field exceeds maximum size".to_string(),
        ));
    }
    Ok(())
}

#[tarpc::service]
pub trait LocalNative {
    async fn is_version_match(version: String) -> Result<bool, RpcError>;
    async fn diff_uuid4_to_server(candidates: Vec<String>) -> Result<Vec<String>, RpcError>;
    async fn diff_uuid4_from_server(candidates: Vec<String>) -> Result<Vec<String>, RpcError>;
    async fn send_note(note: Note) -> Result<bool, RpcError>;
    async fn receive_note(uuid4: String) -> Result<Note, RpcError>;
    async fn stop() -> Result<(), RpcError>;
}

type SharedRateLimiter = Arc<
    RateLimiter<
        governor::state::NotKeyed,
        governor::state::InMemoryState,
        governor::clock::DefaultClock,
    >,
>;

#[derive(Clone)]
struct LocalNativeServer {
    pool: Arc<Mutex<Connection>>,
    stop_token: Option<CancellationToken>,
    /// General rate limiter: 100 requests per second across all methods.
    general_limiter: SharedRateLimiter,
    /// Stricter rate limiter for data-intensive operations (send_note, receive_note): 20 req/sec.
    data_limiter: SharedRateLimiter,
}

impl LocalNativeServer {
    fn check_general_limit(&self) -> Result<(), RpcError> {
        self.general_limiter
            .check()
            .map_err(|_| RpcError::RateLimited)
    }

    fn check_data_limit(&self) -> Result<(), RpcError> {
        self.general_limiter
            .check()
            .map_err(|_| RpcError::RateLimited)?;
        self.data_limiter.check().map_err(|_| RpcError::RateLimited)
    }
}

impl LocalNative for LocalNativeServer {
    async fn is_version_match(
        self,
        _: context::Context,
        version: String,
    ) -> Result<bool, RpcError> {
        self.check_general_limit()?;
        let meta_version = {
            let conn = self.pool.lock().unwrap_or_else(|e| e.into_inner());
            get_meta_version(&conn)?
        };
        Ok(version == meta_version)
    }

    async fn diff_uuid4_to_server(
        self,
        _: context::Context,
        candidates: Vec<String>,
    ) -> Result<Vec<String>, RpcError> {
        self.check_general_limit()?;
        let diff_uuid4 = {
            let conn = self.pool.lock().unwrap_or_else(|e| e.into_inner());
            diff_uuid4_to_server(&conn, candidates)?
        };
        Ok(diff_uuid4)
    }

    async fn diff_uuid4_from_server(
        self,
        _: context::Context,
        candidates: Vec<String>,
    ) -> Result<Vec<String>, RpcError> {
        self.check_general_limit()?;
        let diff_uuid4 = {
            let conn = self.pool.lock().unwrap_or_else(|e| e.into_inner());
            diff_uuid4_from_server(&conn, candidates)?
        };
        Ok(diff_uuid4)
    }

    async fn send_note(self, _: context::Context, note: Note) -> Result<bool, RpcError> {
        self.check_data_limit()?;
        validate_note(&note)?;
        let conn = self.pool.lock().unwrap_or_else(|e| e.into_inner());
        insert(&conn, &note)?;
        Ok(true)
    }

    async fn receive_note(self, _: context::Context, uuid4: String) -> Result<Note, RpcError> {
        self.check_data_limit()?;
        validate_uuid4(&uuid4)?;
        let note = {
            let conn = self.pool.lock().unwrap_or_else(|e| e.into_inner());
            get_note_by_uuid4(&conn, &uuid4)?
        };
        Ok(note)
    }

    async fn stop(self, _: context::Context) -> Result<(), RpcError> {
        self.check_general_limit()?;
        if let Some(stop_tx) = self.stop_token {
            stop_tx.cancel();
        } else {
            return Err(RpcError::ServerConfigError(
                "Server was not started with a stop token".to_string(),
            ));
        }

        Ok(())
    }
}

/// Bind a TCP listener on `addr` and spawn the RPC server task in the background.
/// Cancel the returned future by triggering the `stop_token` (if provided).
pub async fn setup_server(
    addr: SocketAddr,
    pool: Arc<Mutex<Connection>>,
    stop_token: Option<CancellationToken>,
) -> Result<(), RpcError> {
    let listener = tcp::listen(addr, tarpc::tokio_serde::formats::Bincode::default).await?;
    let stop_token_clone = stop_token.clone();

    // 100 requests/sec general limit, 20 requests/sec for data-intensive operations
    let general_limiter: SharedRateLimiter = Arc::new(RateLimiter::direct(Quota::per_second(
        NonZeroU32::new(100).unwrap(),
    )));
    let data_limiter: SharedRateLimiter = Arc::new(RateLimiter::direct(Quota::per_second(
        NonZeroU32::new(20).unwrap(),
    )));

    tokio::spawn(async move {
        tokio::select! {
            _ = listener
                .filter_map(|r| future::ready(r.ok()))
                .map(BaseChannel::with_defaults)
                .max_channels_per_key(2, |t| {
                    t.as_ref()
                        .peer_addr()
                        .map(|a| a.ip())
                        .unwrap_or(std::net::IpAddr::V4(std::net::Ipv4Addr::UNSPECIFIED))
                })
                .map(|channel| {
                    let server = LocalNativeServer {
                        pool: pool.clone(),
                        stop_token: stop_token_clone.clone(),
                        general_limiter: general_limiter.clone(),
                        data_limiter: data_limiter.clone(),
                    };
                    channel.execute(server.serve()).boxed()
                })
                .flatten_unordered(10)
                .buffer_unordered(10)
                .for_each(|_| future::ready(())) => {
                // Server loop completed
            }
            _ = stop_token.as_ref().map(|token| token.cancelled().boxed()).unwrap_or(future::pending().boxed()) => {
                // Stop signal received
            }
        }
    });

    Ok(())
}

/// Return the preferred non-loopback IP address of this host formatted as `"ip:3456"`,
/// or an empty string if no suitable interface is found.
pub fn get_server_addr() -> String {
    get_if_addrs::get_if_addrs()
        .unwrap_or_default()
        .into_iter()
        .find(|iface| !iface.is_loopback())
        .map(|iface| format!("{}:3456", iface.addr.ip()))
        .unwrap_or_default()
}

fn validate_client_addr(addr: &SocketAddr) -> Result<(), RpcError> {
    if addr.ip().is_unspecified() {
        return Err(RpcError::InputValidation(
            "Cannot connect to unspecified address (0.0.0.0)".to_string(),
        ));
    }
    if addr.port() == 0 {
        return Err(RpcError::InputValidation("Port must not be 0".to_string()));
    }
    Ok(())
}

fn validate_server_addr(addr: &SocketAddr) -> Result<(), RpcError> {
    if addr.port() == 0 {
        return Err(RpcError::InputValidation(
            "Server port must not be 0".to_string(),
        ));
    }
    if addr.ip().is_unspecified() {
        tracing::warn!(%addr, "server binding to all interfaces — ensure this is intentional");
    }
    Ok(())
}

async fn check_version_match(
    client: &LocalNativeClient,
    pool: &Arc<Mutex<Connection>>,
) -> Result<bool, RpcError> {
    let version = {
        let conn = pool.lock().unwrap_or_else(|e| e.into_inner());
        get_meta_version(&conn)?
    };
    let is_version_match = client
        .is_version_match(context::current(), version)
        .await??;
    tracing::debug!(is_version_match, "version check result");
    if !is_version_match {
        return Err(RpcError::VersionMismatch);
    }
    Ok(is_version_match)
}

/// Push local notes that the server does not yet have.
pub async fn run_sync_to_server(
    addr: &SocketAddr,
    pool: &Arc<Mutex<Connection>>,
) -> Result<(), RpcError> {
    let transport =
        tarpc::serde_transport::tcp::connect(addr, tarpc::tokio_serde::formats::Bincode::default)
            .await?;
    let client = LocalNativeClient::new(client::Config::default(), transport).spawn();

    check_version_match(&client, pool).await?;

    let candidates = {
        let conn = pool.lock().unwrap_or_else(|e| e.into_inner());
        next_uuid4_candidates(&conn)?
    };
    let diff_uuid4 = client
        .diff_uuid4_to_server(context::current(), candidates)
        .await??;
    tracing::info!(count = diff_uuid4.len(), "notes to send to server");

    for u in diff_uuid4 {
        let note = {
            let conn = pool.lock().unwrap_or_else(|e| e.into_inner());
            get_note_by_uuid4(&conn, &u)?
        };
        client.send_note(context::current(), note).await??;
    }
    tracing::info!("sync to server complete");

    Ok(())
}

/// Pull notes from the server that this client does not yet have.
pub async fn run_sync_from_server(
    addr: &SocketAddr,
    pool: &Arc<Mutex<Connection>>,
) -> Result<(), RpcError> {
    let transport =
        tarpc::serde_transport::tcp::connect(addr, tarpc::tokio_serde::formats::Bincode::default)
            .await?;
    let client = LocalNativeClient::new(client::Config::default(), transport).spawn();

    check_version_match(&client, pool).await?;

    let candidates = {
        let conn = pool.lock().unwrap_or_else(|e| e.into_inner());
        next_uuid4_candidates(&conn)?
    };
    let diff_uuid4 = client
        .diff_uuid4_from_server(context::current(), candidates)
        .await??;
    tracing::info!(count = diff_uuid4.len(), "notes to receive from server");

    for u in diff_uuid4 {
        client.receive_note(context::current(), u).await??;
    }
    tracing::info!("sync from server complete");

    Ok(())
}

/// Bidirectional sync with the server at `addr`: push local-only notes and pull server-only notes
/// concurrently. Returns `"sync ok"` on success.
pub async fn sync(addr: &str, pool: &Arc<Mutex<Connection>>) -> Result<String, RpcError> {
    let server_addr: SocketAddr = addr.parse()?;
    validate_client_addr(&server_addr)?;

    tokio::try_join!(
        run_sync_to_server(&server_addr, pool),
        run_sync_from_server(&server_addr, pool)
    )?;

    Ok("sync ok".to_string())
}

pub async fn run_stop_server(
    addr: &SocketAddr,
    pool: &Arc<Mutex<Connection>>,
) -> Result<(), RpcError> {
    let transport =
        tarpc::serde_transport::tcp::connect(addr, tarpc::tokio_serde::formats::Bincode::default)
            .await?;
    let client = LocalNativeClient::new(client::Config::default(), transport).spawn();

    check_version_match(&client, pool).await?;

    client.stop(context::current()).await??;
    Ok(())
}

/// Send a stop signal to the server at `addr`. Returns `"stop ok"` on success.
pub async fn stop_server(addr: &str, pool: &Arc<Mutex<Connection>>) -> Result<String, RpcError> {
    let server_addr: SocketAddr = addr.parse()?;
    validate_client_addr(&server_addr)?;
    run_stop_server(&server_addr, pool).await?;
    Ok("stop ok".to_string())
}

/// Start the RPC server bound to `addr` with a fresh cancellation token.
pub async fn start(addr: &str, pool: &Arc<Mutex<Connection>>) -> Result<(), RpcError> {
    let server_addr: SocketAddr = addr.parse()?;
    validate_server_addr(&server_addr)?;

    setup_server(server_addr, pool.clone(), Some(CancellationToken::new())).await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_uuid4_valid() {
        assert!(validate_uuid4("550e8400-e29b-41d4-a716-446655440000").is_ok());
    }

    #[test]
    fn test_validate_uuid4_invalid() {
        assert!(validate_uuid4("not-a-uuid").is_err());
        assert!(validate_uuid4("").is_err());
        assert!(validate_uuid4("550e8400-e29b-41d4-a716").is_err());
    }

    #[test]
    fn test_validate_note_valid() {
        let note = Note {
            rowid: 1,
            uuid4: "550e8400-e29b-41d4-a716-446655440000".to_string(),
            title: "Test".to_string(),
            url: "https://example.com".to_string(),
            tags: "tag1,tag2".to_string(),
            description: "desc".to_string(),
            comments: "comment".to_string(),
            annotations: "abcd".to_string(),
            created_at: "2024-01-01 00:00:00".to_string(),
            is_public: false,
        };
        assert!(validate_note(&note).is_ok());
    }

    #[test]
    fn test_validate_note_invalid_uuid() {
        let note = Note {
            rowid: 1,
            uuid4: "invalid-uuid".to_string(),
            title: "Test".to_string(),
            url: "https://example.com".to_string(),
            tags: "".to_string(),
            description: "".to_string(),
            comments: "".to_string(),
            annotations: "".to_string(),
            created_at: "2024-01-01 00:00:00".to_string(),
            is_public: false,
        };
        assert!(validate_note(&note).is_err());
    }

    #[test]
    fn test_validate_note_oversized_field() {
        let oversized = "x".repeat(MAX_NOTE_FIELD_SIZE + 1);
        let note = Note {
            rowid: 1,
            uuid4: "550e8400-e29b-41d4-a716-446655440000".to_string(),
            title: oversized,
            url: "https://example.com".to_string(),
            tags: "".to_string(),
            description: "".to_string(),
            comments: "".to_string(),
            annotations: "".to_string(),
            created_at: "2024-01-01 00:00:00".to_string(),
            is_public: false,
        };
        assert!(validate_note(&note).is_err());
    }

    #[test]
    fn test_validate_client_addr_unspecified() {
        let addr: SocketAddr = "0.0.0.0:2345".parse().unwrap();
        assert!(validate_client_addr(&addr).is_err());
    }

    #[test]
    fn test_validate_client_addr_zero_port() {
        let addr: SocketAddr = "127.0.0.1:0".parse().unwrap();
        assert!(validate_client_addr(&addr).is_err());
    }

    #[test]
    fn test_validate_client_addr_valid() {
        let addr: SocketAddr = "192.168.1.1:2345".parse().unwrap();
        assert!(validate_client_addr(&addr).is_ok());
    }

    #[test]
    fn test_validate_server_addr_zero_port() {
        let addr: SocketAddr = "127.0.0.1:0".parse().unwrap();
        assert!(validate_server_addr(&addr).is_err());
    }

    #[test]
    fn test_validate_server_addr_valid() {
        let addr: SocketAddr = "127.0.0.1:2345".parse().unwrap();
        assert!(validate_server_addr(&addr).is_ok());
    }
}
