use crate::db::{
    diff_uuid4_from_server, diff_uuid4_to_server, get_meta_version, get_note_by_uuid4, insert,
    next_uuid4_candidates, DbError, Note,
};
use futures::{future, FutureExt, StreamExt};
use sqlx::SqlitePool;
use std::net::SocketAddr;
use std::process;
use tarpc::client;
use tarpc::server::incoming::Incoming as _;
use tarpc::server::Channel as _;
use tarpc::{context, serde_transport::tcp, server::BaseChannel};
use thiserror::Error;
use tokio_serde::formats::Bincode;
use tokio_util::sync::CancellationToken;

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

#[derive(Clone)]
struct LocalNativeServer {
    pool: SqlitePool,
    stop_token: Option<CancellationToken>,
}

impl LocalNative for LocalNativeServer {
    async fn is_version_match(
        self,
        _: context::Context,
        version: String,
    ) -> Result<bool, RpcError> {
        let meta_version = get_meta_version(&self.pool).await?;
        Ok(version == meta_version)
    }

    async fn diff_uuid4_to_server(
        self,
        _: context::Context,
        candidates: Vec<String>,
    ) -> Result<Vec<String>, RpcError> {
        let diff_uuid4 = diff_uuid4_to_server(&self.pool, candidates).await?;
        Ok(diff_uuid4)
    }

    async fn diff_uuid4_from_server(
        self,
        _: context::Context,
        candidates: Vec<String>,
    ) -> Result<Vec<String>, RpcError> {
        let diff_uuid4 = diff_uuid4_from_server(&self.pool, candidates).await?;
        Ok(diff_uuid4)
    }

    async fn send_note(self, _: context::Context, note: Note) -> Result<bool, RpcError> {
        // Implement note sending logic here
        insert(&self.pool, note).await?;
        Ok(true)
    }

    async fn receive_note(self, _: context::Context, uuid4: String) -> Result<Note, RpcError> {
        let note = get_note_by_uuid4(&self.pool, &uuid4).await?;
        Ok(note)
    }

    async fn stop(self, _: context::Context) -> Result<(), RpcError> {
        if let Some(stop_tx) = self.stop_token {
            stop_tx.cancel();
        } else {
            process::exit(0)
        }

        Ok(())
    }
}

async fn setup_server(
    addr: SocketAddr,
    pool: SqlitePool,
    stop_token: Option<CancellationToken>,
) -> Result<(), RpcError> {
    let listener = tcp::listen(addr, tarpc::tokio_serde::formats::Bincode::default).await?;
    let stop_token_clone = stop_token.clone();

    tokio::spawn(async move {
        tokio::select! {
            _ = listener
                .filter_map(|r| future::ready(r.ok()))
                .map(BaseChannel::with_defaults)
                .max_channels_per_key(2, |t| t.as_ref().peer_addr().unwrap().ip())
                .map(|channel| {
                    let server = LocalNativeServer {
                        pool: pool.clone(),
                        stop_token: stop_token_clone.clone(),
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

pub fn get_server_addr() -> String {
    get_if_addrs::get_if_addrs()
        .unwrap_or_default()
        .into_iter()
        .find(|iface| !iface.is_loopback())
        .map(|iface| format!("{}:3456", iface.addr.ip()))
        .unwrap_or_default()
}

async fn check_version_match(
    client: &LocalNativeClient,
    pool: &SqlitePool,
) -> Result<bool, RpcError> {
    let version = get_meta_version(pool).await?;
    let is_version_match = client
        .is_version_match(context::current(), version)
        .await??;
    eprintln!("is_version_match: {}", is_version_match);
    if !is_version_match {
        return Err(RpcError::VersionMismatch);
    }
    Ok(is_version_match)
}

pub async fn run_sync_to_server(addr: &SocketAddr, pool: &SqlitePool) -> Result<(), RpcError> {
    let transport = tarpc::serde_transport::tcp::connect(addr, Bincode::default).await?;
    let client = LocalNativeClient::new(client::Config::default(), transport).spawn();

    check_version_match(&client, pool).await?;

    let candidates = next_uuid4_candidates(pool).await?;
    let diff_uuid4 = client
        .diff_uuid4_to_server(context::current(), candidates)
        .await??;
    eprintln!("diff_uuid4_to_server len: {:?}", diff_uuid4.len());

    for u in diff_uuid4 {
        let uuid4 = get_note_by_uuid4(pool, &u).await?;
        client.send_note(context::current(), uuid4).await??;
    }
    eprintln!("send_note done");

    Ok(())
}

pub async fn run_sync_from_server(addr: &SocketAddr, pool: &SqlitePool) -> Result<(), RpcError> {
    let transport = tarpc::serde_transport::tcp::connect(addr, Bincode::default).await?;
    let client = LocalNativeClient::new(client::Config::default(), transport).spawn();

    check_version_match(&client, pool).await?;

    let candidates = next_uuid4_candidates(pool).await?;
    let diff_uuid4 = client
        .diff_uuid4_from_server(context::current(), candidates)
        .await??;
    eprintln!("diff_uuid4_from_server len: {:?}", diff_uuid4.len());

    for u in diff_uuid4 {
        client.receive_note(context::current(), u).await??;
    }
    eprintln!("receive_note done");

    Ok(())
}

pub async fn sync(addr: &str, pool: &SqlitePool) -> Result<String, RpcError> {
    let server_addr: SocketAddr = addr.parse()?;

    tokio::try_join!(
        run_sync_to_server(&server_addr, pool),
        run_sync_from_server(&server_addr, pool)
    )?;

    Ok("sync ok".to_string())
}

pub async fn run_stop_server(addr: &SocketAddr, pool: &SqlitePool) -> Result<(), RpcError> {
    let transport = tarpc::serde_transport::tcp::connect(addr, Bincode::default).await?;
    let client = LocalNativeClient::new(client::Config::default(), transport).spawn();

    check_version_match(&client, pool).await?;

    client.stop(context::current()).await??;
    Ok(())
}

pub async fn stop_server(addr: &str, pool: &SqlitePool) -> Result<String, RpcError> {
    let server_addr: SocketAddr = addr.parse()?;
    run_stop_server(&server_addr, pool).await?;
    Ok("stop ok".to_string())
}

pub async fn start(addr: &str, pool: &SqlitePool) -> Result<(), RpcError> {
    let server_addr: SocketAddr = addr.parse()?;

    setup_server(server_addr, pool.clone(), Some(CancellationToken::new())).await?;

    Ok(())
}
