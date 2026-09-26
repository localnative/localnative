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

//! Peer-to-peer sync over the LAN.
//!
//! One TCP connection per sync, everything inside a Noise-encrypted channel
//! ([`crate::secure`]). After a handshake that authenticates both devices,
//! the session is: exchange hello → compare 256 bucket hashes → exchange
//! versions only for buckets that differ → push and pull notes in
//! byte-budgeted batches, each applied in one transaction. Two devices that
//! are already in sync exchange a few kilobytes regardless of library size.
//!
//! There is no unauthenticated entry point: a connection that neither
//! completes pairing nor presents a trusted key never reaches a database
//! query, and there is no remote "stop" — stopping a server is local.

use crate::db::{self, Pool, peers, sync as dbsync};
use crate::error::{SyncError, ValidationError};
use crate::secure::{ActiveCode, BATCH_BUDGET, CODE_LEN, MAX_PAYLOAD, SecureChannel};
use crate::wire::{self, Applied, Diff, Hello, NoteV1, PROTOCOL_VERSION, Verdict, WireError};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex, OnceLock};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio_util::sync::CancellationToken;

const HANDSHAKE_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(30);
const MESSAGE_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(60);
/// Annotations larger than one message are streamed in chunks of this size.
const BLOB_CHUNK: usize = 24 * 1024;
/// Conservative ceiling for one `Notes` message (serialized, encrypted,
/// framed — all bounded by `MAX_PAYLOAD`).
const NOTES_MESSAGE_BUDGET: usize = 48 * 1024;

// ── Wire messages ─────────────────────────────────────────────────────────

/// One slice of a note's annotations, sent after the note itself.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Blob {
    pub uuid4: String,
    pub bytes: Vec<u8>,
}

#[derive(Serialize, Deserialize, Debug)]
enum ClientMsg {
    Hello(Hello),
    /// This client's bucket hashes.
    Hashes(Vec<[u8; 32]>),
    /// Versions of this client's notes in the buckets whose hashes differed.
    DiffReq {
        buckets: Vec<u16>,
        versions: Vec<(String, String)>,
    },
    /// Notes being pushed; annotations beyond the first chunk arrive as
    /// [`ClientMsg::Blob`]. Applied when [`ClientMsg::NotesEnd`] arrives.
    Notes(Vec<NoteV1>),
    Blob(Blob),
    NotesEnd,
    /// UUIDs to pull, in order; the server answers with Notes/Blob/NotesEnd.
    Pull(Vec<String>),
    Done,
}

#[derive(Serialize, Deserialize, Debug)]
enum ServerMsg {
    Verdict(Verdict),
    Hello(Hello),
    Hashes(Vec<[u8; 32]>),
    Diff(Diff),
    Notes(Vec<NoteV1>),
    Blob(Blob),
    NotesEnd {
        consumed: u32,
    },
    /// Result of applying a pushed batch.
    Ack(Applied),
    Err(WireError),
}

fn protocol(e: impl std::fmt::Display) -> SyncError {
    SyncError::Protocol(e.to_string())
}

async fn send_msg<IO, M>(channel: &mut SecureChannel, io: &mut IO, msg: &M) -> Result<(), SyncError>
where
    IO: AsyncWriteExt + Unpin,
    M: Serialize,
{
    let bytes = bincode::serialize(msg).map_err(protocol)?;
    if bytes.len() > MAX_PAYLOAD {
        return Err(protocol(
            "outgoing message exceeds frame size (batching bug)",
        ));
    }
    channel.send(io, &bytes).await
}

async fn recv_msg<IO, M>(channel: &mut SecureChannel, io: &mut IO) -> Result<M, SyncError>
where
    IO: AsyncReadExt + Unpin,
    M: for<'de> Deserialize<'de>,
{
    let bytes = tokio::time::timeout(MESSAGE_TIMEOUT, channel.recv(io))
        .await
        .map_err(|_| SyncError::Timeout("peer stopped responding"))??;
    bincode::deserialize(&bytes).map_err(protocol)
}

// ── Batched note transfer ─────────────────────────────────────────────────

/// Send `notes` as a series of `Notes`/`Blob` messages, every message within
/// the frame budget. The caller sends the end marker (which differs by
/// direction) when the whole batch is out.
async fn send_notes<IO, C>(
    channel: &mut SecureChannel,
    io: &mut IO,
    notes: Vec<NoteV1>,
) -> Result<(), SyncError>
where
    IO: AsyncReadExt + AsyncWriteExt + Unpin,
    C: NotesCarrier,
{
    let mut batch: Vec<NoteV1> = Vec::new();
    let mut budget = NOTES_MESSAGE_BUDGET;
    for mut note in notes {
        let mut blobs = Vec::new();
        if note.annotations.len() > BLOB_CHUNK {
            let rest = note.annotations.split_off(BLOB_CHUNK);
            blobs = rest
                .chunks(BLOB_CHUNK)
                .map(|bytes| Blob {
                    uuid4: note.uuid4.clone(),
                    bytes: bytes.to_vec(),
                })
                .collect();
        }
        let size = note.encoded_len() + 96;
        if budget < size {
            C::send_notes_message(channel, io, std::mem::take(&mut batch)).await?;
            budget = NOTES_MESSAGE_BUDGET;
        }
        budget = budget.saturating_sub(size);
        batch.push(note);
        if !blobs.is_empty() {
            C::send_notes_message(channel, io, std::mem::take(&mut batch)).await?;
            budget = NOTES_MESSAGE_BUDGET;
            for blob in blobs {
                C::send_blob(channel, io, blob).await?;
            }
        }
    }
    C::send_notes_message(channel, io, std::mem::take(&mut batch)).await?;
    Ok(())
}

/// The two directions a note transfer can flow, abstracted so the batching
/// code above is written once.
trait NotesCarrier {
    async fn send_notes_message<IO>(
        channel: &mut SecureChannel,
        io: &mut IO,
        notes: Vec<NoteV1>,
    ) -> Result<(), SyncError>
    where
        IO: AsyncReadExt + AsyncWriteExt + Unpin;
    async fn send_blob<IO>(
        channel: &mut SecureChannel,
        io: &mut IO,
        blob: Blob,
    ) -> Result<(), SyncError>
    where
        IO: AsyncReadExt + AsyncWriteExt + Unpin;
}

enum Client {}
enum Server {}

impl NotesCarrier for Client {
    async fn send_notes_message<IO>(
        channel: &mut SecureChannel,
        io: &mut IO,
        notes: Vec<NoteV1>,
    ) -> Result<(), SyncError>
    where
        IO: AsyncReadExt + AsyncWriteExt + Unpin,
    {
        send_msg(channel, io, &ClientMsg::Notes(notes)).await
    }
    async fn send_blob<IO>(
        channel: &mut SecureChannel,
        io: &mut IO,
        blob: Blob,
    ) -> Result<(), SyncError>
    where
        IO: AsyncReadExt + AsyncWriteExt + Unpin,
    {
        send_msg(channel, io, &ClientMsg::Blob(blob)).await
    }
}

impl NotesCarrier for Server {
    async fn send_notes_message<IO>(
        channel: &mut SecureChannel,
        io: &mut IO,
        notes: Vec<NoteV1>,
    ) -> Result<(), SyncError>
    where
        IO: AsyncReadExt + AsyncWriteExt + Unpin,
    {
        send_msg(channel, io, &ServerMsg::Notes(notes)).await
    }
    async fn send_blob<IO>(
        channel: &mut SecureChannel,
        io: &mut IO,
        blob: Blob,
    ) -> Result<(), SyncError>
    where
        IO: AsyncReadExt + AsyncWriteExt + Unpin,
    {
        send_msg(channel, io, &ServerMsg::Blob(blob)).await
    }
}

/// Collect `Notes`/`Blob` messages until `NotesEnd`, reassembling chunked
/// annotations. `from_server` selects which enum to read.
async fn recv_notes<IO>(
    channel: &mut SecureChannel,
    io: &mut IO,
    from_server: bool,
) -> Result<(Vec<NoteV1>, u32), SyncError>
where
    IO: AsyncReadExt + Unpin,
{
    let mut notes: Vec<NoteV1> = Vec::new();
    let mut by_uuid: HashMap<String, usize> = HashMap::new();
    let mut consumed = 0u32;
    if from_server {
        loop {
            match recv_msg::<_, ServerMsg>(channel, io).await? {
                ServerMsg::Notes(batch) => {
                    consumed =
                        consumed.saturating_add(u32::try_from(batch.len()).unwrap_or(u32::MAX));
                    for note in batch {
                        by_uuid.insert(note.uuid4.clone(), notes.len());
                        notes.push(note);
                    }
                }
                ServerMsg::Blob(blob) => append_blob(&mut notes, &by_uuid, blob),
                ServerMsg::NotesEnd { consumed: c } => return Ok((notes, c.max(consumed))),
                ServerMsg::Err(e) => {
                    return Err(SyncError::Remote {
                        code: e.code,
                        message: e.message,
                    });
                }
                other => {
                    return Err(protocol(format!(
                        "unexpected message during pull: {other:?}"
                    )));
                }
            }
        }
    }
    loop {
        match recv_msg::<_, ClientMsg>(channel, io).await? {
            ClientMsg::Notes(batch) => {
                for note in batch {
                    by_uuid.insert(note.uuid4.clone(), notes.len());
                    notes.push(note);
                }
            }
            ClientMsg::Blob(blob) => append_blob(&mut notes, &by_uuid, blob),
            ClientMsg::NotesEnd => return Ok((notes, consumed)),
            other => {
                return Err(protocol(format!(
                    "unexpected message during push: {other:?}"
                )));
            }
        }
    }
}

fn append_blob(notes: &mut [NoteV1], by_uuid: &HashMap<String, usize>, blob: Blob) {
    if let Some(&i) = by_uuid.get(&blob.uuid4)
        && let Some(note) = notes.get_mut(i)
    {
        note.annotations.extend_from_slice(&blob.bytes);
    }
}

// ── Server ────────────────────────────────────────────────────────────────

struct ServerHandle {
    stop: CancellationToken,
    pairing: Arc<Mutex<Option<ActiveCode>>>,
}

fn registry() -> &'static Mutex<HashMap<SocketAddr, ServerHandle>> {
    static REGISTRY: OnceLock<Mutex<HashMap<SocketAddr, ServerHandle>>> = OnceLock::new();
    REGISTRY.get_or_init(|| Mutex::new(HashMap::new()))
}

fn lock<T>(lock: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    lock.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
}

fn active_pairing(pairing: &Arc<Mutex<Option<ActiveCode>>>) -> Option<ActiveCode> {
    lock(pairing)
        .as_ref()
        .filter(|code| !code.expired())
        .cloned()
}

fn this_device(conn: &rusqlite::Connection) -> Result<Hello, SyncError> {
    Ok(Hello {
        protocol: PROTOCOL_VERSION,
        min_protocol: wire::MIN_PROTOCOL_VERSION,
        node_id: db::node_id(conn)?,
        name: hostname::get()
            .unwrap_or_default()
            .to_string_lossy()
            .trim_end_matches(".local")
            .to_string(),
        schema: db::migrations::get_meta_version(conn)?,
    })
}

/// Bind `addr` and serve sync sessions until `stop_token` fires.
pub async fn setup_server(
    addr: SocketAddr,
    pool: Pool,
    stop_token: CancellationToken,
    pairing: Arc<Mutex<Option<ActiveCode>>>,
) -> Result<(), SyncError> {
    let listener = TcpListener::bind(addr).await?;
    let bound = listener.local_addr()?;
    let node = pool
        .get()
        .ok()
        .and_then(|conn| db::node_id(&conn).ok())
        .unwrap_or_default();
    let mdns = crate::discovery::start_advertising(bound.port(), &node).ok();
    tracing::info!(%bound, "sync server listening");

    loop {
        tokio::select! {
            _ = stop_token.cancelled() => break,
            accepted = listener.accept() => {
                let Ok((stream, _peer)) = accepted else { continue };
                let pool = pool.clone();
                let pairing = pairing.clone();
                tokio::spawn(async move {
                    if let Err(e) = serve_connection(stream, pool, pairing).await {
                        tracing::info!(%e, "sync session ended with error");
                    }
                });
            }
        }
    }

    if let Some(daemon) = mdns {
        crate::discovery::stop_advertising(daemon);
    }
    tracing::info!(%bound, "sync server stopped");
    Ok(())
}

async fn serve_connection(
    mut stream: TcpStream,
    pool: Pool,
    pairing: Arc<Mutex<Option<ActiveCode>>>,
) -> Result<(), SyncError> {
    // Pairing is only offered while a code is active; sync mode is always on.
    // The pooled connection is only held for synchronous spans, never across
    // an await — rusqlite connections are not `Sync`, so holding one would
    // make this task unspawnable.
    let private_key = {
        let conn = pool
            .get()
            .map_err(|e| SyncError::PoolError(e.to_string()))?;
        crate::secure::static_private_key(&conn)?
    };
    let accepting = active_pairing(&pairing);
    let handshake = tokio::time::timeout(
        HANDSHAKE_TIMEOUT,
        SecureChannel::accept(&private_key, &mut stream, accepting.as_ref()),
    )
    .await
    .map_err(|_| SyncError::Timeout("handshake"))??;

    let remote_key = handshake
        .remote_static()
        .map(|k| k.to_vec())
        .ok_or_else(|| SyncError::Handshake("peer presented no static key".to_string()))?;
    let pairing_proved = handshake.pairing_used();
    let mut channel = handshake.into_transport()?;

    // Gate on trust before anything is exchanged: a sync handshake with a
    // key we never paired is refused outright — completing Noise_XX alone
    // proves nothing. A pairing handshake already proved the one-time code,
    // so the (until now unknown) key can be recorded.
    if !pairing_proved {
        let trusted = {
            let conn = pool
                .get()
                .map_err(|e| SyncError::PoolError(e.to_string()))?;
            peers::is_trusted(&conn, &remote_key)?
        };
        if !trusted {
            tracing::warn!("refusing sync session from an unpaired device");
            send_msg(
                &mut channel,
                &mut stream,
                &ServerMsg::Verdict(Verdict {
                    accepted: false,
                    reason: Some("not-paired".to_string()),
                }),
            )
            .await?;
            let _ = stream.shutdown().await;
            return Ok(());
        }
    }

    let client_hello: Hello = match recv_msg::<_, ClientMsg>(&mut channel, &mut stream).await? {
        ClientMsg::Hello(hello) => hello,
        other => return Err(protocol(format!("expected hello, got {other:?}"))),
    };
    let compatible = {
        let conn = pool
            .get()
            .map_err(|e| SyncError::PoolError(e.to_string()))?;
        client_hello.compatible_with(&this_device(&conn)?)
    };
    if !compatible {
        send_msg(
            &mut channel,
            &mut stream,
            &ServerMsg::Verdict(Verdict {
                accepted: false,
                reason: Some("incompatible-protocol".to_string()),
            }),
        )
        .await?;
        return Err(SyncError::Incompatible {
            local: PROTOCOL_VERSION,
            remote: client_hello.protocol,
        });
    }

    if pairing_proved {
        // First contact through a valid code: record the device.
        let conn = pool
            .get()
            .map_err(|e| SyncError::PoolError(e.to_string()))?;
        peers::trust(
            &conn,
            &remote_key,
            &client_hello.node_id,
            &client_hello.name,
            &stream
                .peer_addr()
                .map(|a| a.to_string())
                .unwrap_or_default(),
        )?;
    }
    send_msg(
        &mut channel,
        &mut stream,
        &ServerMsg::Verdict(Verdict {
            accepted: true,
            reason: None,
        }),
    )
    .await?;
    let server_hello = {
        let conn = pool
            .get()
            .map_err(|e| SyncError::PoolError(e.to_string()))?;
        this_device(&conn)?
    };
    send_msg(&mut channel, &mut stream, &ServerMsg::Hello(server_hello)).await?;

    // Session: reconciliation, then note transfer in both directions.
    let mut push_buffer: Vec<NoteV1> = Vec::new();
    let mut push_index: HashMap<String, usize> = HashMap::new();
    loop {
        match recv_msg::<_, ClientMsg>(&mut channel, &mut stream).await? {
            ClientMsg::Hashes(_) => {
                let hashes = {
                    let conn = pool
                        .get()
                        .map_err(|e| SyncError::PoolError(e.to_string()))?;
                    let versions = dbsync::note_versions(&conn)?;
                    dbsync::bucket_hashes(&versions)
                };
                send_msg(&mut channel, &mut stream, &ServerMsg::Hashes(hashes)).await?;
            }
            ClientMsg::DiffReq { buckets, versions } => {
                let diff = {
                    let conn = pool
                        .get()
                        .map_err(|e| SyncError::PoolError(e.to_string()))?;
                    dbsync::diff_versions(&conn, &buckets, versions)?
                };
                send_msg(&mut channel, &mut stream, &ServerMsg::Diff(diff)).await?;
            }
            ClientMsg::Notes(batch) => {
                for note in batch {
                    push_index.insert(note.uuid4.clone(), push_buffer.len());
                    push_buffer.push(note);
                }
            }
            ClientMsg::Blob(blob) => {
                append_blob(&mut push_buffer, &push_index, blob);
            }
            ClientMsg::NotesEnd => {
                let applied = {
                    let conn = pool
                        .get()
                        .map_err(|e| SyncError::PoolError(e.to_string()))?;
                    dbsync::apply_notes(&conn, &push_buffer)?
                };
                push_buffer.clear();
                push_index.clear();
                send_msg(&mut channel, &mut stream, &ServerMsg::Ack(applied)).await?;
            }
            ClientMsg::Pull(uuids) => {
                let (notes, consumed) = {
                    let conn = pool
                        .get()
                        .map_err(|e| SyncError::PoolError(e.to_string()))?;
                    dbsync::load_notes(&conn, &uuids, BATCH_BUDGET)?
                };
                send_notes::<_, Server>(&mut channel, &mut stream, notes).await?;
                send_msg(
                    &mut channel,
                    &mut stream,
                    &ServerMsg::NotesEnd {
                        consumed: u32::try_from(consumed).unwrap_or(u32::MAX),
                    },
                )
                .await?;
            }
            ClientMsg::Done => {
                {
                    let conn = pool
                        .get()
                        .map_err(|e| SyncError::PoolError(e.to_string()))?;
                    peers::touch(&conn, &remote_key)?;
                }
                let _ = stream.shutdown().await;
                return Ok(());
            }
            other => return Err(protocol(format!("unexpected session message: {other:?}"))),
        }
    }
}

// ── Address helpers ───────────────────────────────────────────────────────

/// This device's non-loopback LAN addresses as `"<ip>:<port>"`, for display.
pub fn server_addresses(port: u16) -> Vec<String> {
    if_addrs::get_if_addrs()
        .unwrap_or_default()
        .into_iter()
        .filter(|iface| !iface.is_loopback())
        .map(|iface| format!("{}:{}", iface.ip(), port))
        .collect()
}

fn validate_client_addr(addr: &SocketAddr) -> Result<(), SyncError> {
    if addr.ip().is_unspecified() {
        return Err(ValidationError::Other(
            "Cannot connect to unspecified address (0.0.0.0)".to_string(),
        )
        .into());
    }
    if addr.port() == 0 {
        return Err(ValidationError::Other("Port must not be 0".to_string()).into());
    }
    Ok(())
}

fn validate_server_addr(addr: &SocketAddr) -> Result<(), SyncError> {
    if addr.port() == 0 {
        return Err(ValidationError::Other("Server port must not be 0".to_string()).into());
    }
    if addr.ip().is_unspecified() {
        tracing::warn!(%addr, "server binding to all interfaces -- ensure this is intentional");
    }
    Ok(())
}

// ── JSON-dispatch entry points ────────────────────────────────────────────

/// Start (or confirm) a sync server on `addr`, accepting no new pairings.
pub async fn start(addr: &str, pool: &Pool) -> Result<String, SyncError> {
    let addr: SocketAddr = addr.parse()?;
    validate_server_addr(&addr)?;
    let handle = ensure_server(addr, pool.clone()).await?;
    *lock(&handle.pairing) = None;
    Ok("started".to_string())
}

/// Start the server (if needed) and accept one new pairing for the next few
/// minutes. Returns the code to show on the other device.
pub async fn start_pairing(addr: &str, pool: &Pool) -> Result<String, SyncError> {
    let addr: SocketAddr = addr.parse()?;
    validate_server_addr(&addr)?;
    let handle = ensure_server(addr, pool.clone()).await?;
    let code = ActiveCode::fresh();
    let display = code.code.clone();
    *lock(&handle.pairing) = Some(code);
    Ok(display)
}

async fn ensure_server(addr: SocketAddr, pool: Pool) -> Result<Arc<ServerHandle>, SyncError> {
    // Register (or find) the server in one synchronous span; the lock is
    // never held across an await.
    let handle = {
        let mut servers = registry()
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if let Some(existing) = servers.get(&addr) {
            return Ok(Arc::new(ServerHandle {
                stop: existing.stop.clone(),
                pairing: existing.pairing.clone(),
            }));
        }
        let stop = CancellationToken::new();
        let pairing: Arc<Mutex<Option<ActiveCode>>> = Arc::new(Mutex::new(None));
        servers.insert(
            addr,
            ServerHandle {
                stop: stop.clone(),
                pairing: pairing.clone(),
            },
        );
        Arc::new(ServerHandle { stop, pairing })
    };
    let (stop, pairing) = (handle.stop.clone(), handle.pairing.clone());
    tokio::spawn(async move {
        // Surface "address in use" asynchronously; the registry entry is
        // removed so a later start can retry.
        if let Err(e) = setup_server(addr, pool, stop, pairing).await {
            tracing::error!(%e, "sync server failed to start");
            registry()
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .remove(&addr);
        }
    });
    // Give the bind a moment to fail fast (e.g. port already taken elsewhere).
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    Ok(handle)
}

/// Stop the server this process runs on `addr` (a local action — there is
/// deliberately no remote stop).
pub async fn stop_local(addr: &str) -> Result<String, SyncError> {
    let addr: SocketAddr = addr.parse()?;
    let mut servers = registry()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let handle = servers
        .remove(&addr)
        .ok_or_else(|| SyncError::ServerConfigError(format!("no sync server running on {addr}")))?;
    handle.stop.cancel();
    Ok("stopped".to_string())
}

/// Run one synchronous database operation on a pooled connection.
fn with_conn<T>(
    pool: &Pool,
    f: impl FnOnce(&rusqlite::Connection) -> Result<T, SyncError>,
) -> Result<T, SyncError> {
    let conn = pool
        .get()
        .map_err(|e| SyncError::PoolError(e.to_string()))?;
    f(&conn)
}

/// Sync with the peer at `addr`. `code` pairs when this device has never
/// synced with that peer before. Returns a human-readable summary.
pub async fn sync(addr: &str, code: Option<&str>, pool: &Pool) -> Result<String, SyncError> {
    let addr: SocketAddr = addr.parse()?;
    validate_client_addr(&addr)?;

    if let Some(code) = code {
        let normalized: String = code.chars().filter(|c| c.is_ascii_alphanumeric()).collect();
        if normalized.len() != CODE_LEN {
            return Err(SyncError::InvalidPairingCode);
        }
    }

    let stored_key = with_conn(pool, |conn| {
        peers::find_by_addr(conn, &addr.to_string()).map_err(SyncError::from)
    })?;
    if code.is_none() && stored_key.is_none() {
        return Err(SyncError::NotPaired);
    }

    let mut stream = TcpStream::connect(addr).await?;
    let mode = if code.is_some() { b'P' } else { b'S' };
    stream.write_all(&[mode]).await?;

    let private_key = with_conn(pool, |conn| {
        crate::secure::static_private_key(conn).map_err(SyncError::from)
    })?;
    let handshake = tokio::time::timeout(
        HANDSHAKE_TIMEOUT,
        SecureChannel::connect(&private_key, &mut stream, code, stored_key.as_deref()),
    )
    .await
    .map_err(|_| SyncError::Timeout("handshake"))??;
    let remote_key = handshake
        .remote_static()
        .map(|k| k.to_vec())
        .ok_or_else(|| SyncError::Handshake("peer presented no static key".to_string()))?;
    let mut channel = handshake.into_transport()?;

    let my_hello = with_conn(pool, this_device)?;
    send_msg(
        &mut channel,
        &mut stream,
        &ClientMsg::Hello(my_hello.clone()),
    )
    .await?;

    match recv_msg::<_, ServerMsg>(&mut channel, &mut stream).await? {
        ServerMsg::Verdict(Verdict { accepted: true, .. }) => {}
        ServerMsg::Verdict(Verdict {
            accepted: false,
            reason,
        }) => {
            return Err(match reason.as_deref() {
                Some("not-paired") => SyncError::NotPaired,
                Some("incompatible-protocol") => SyncError::Incompatible {
                    local: PROTOCOL_VERSION,
                    remote: 0,
                },
                other => SyncError::Rejected(other.unwrap_or("declined").to_string()),
            });
        }
        ServerMsg::Err(e) => {
            return Err(SyncError::Remote {
                code: e.code,
                message: e.message,
            });
        }
        other => return Err(protocol(format!("expected verdict, got {other:?}"))),
    }
    let server_hello: Hello = match recv_msg::<_, ServerMsg>(&mut channel, &mut stream).await? {
        ServerMsg::Hello(hello) => hello,
        ServerMsg::Err(e) => {
            return Err(SyncError::Remote {
                code: e.code,
                message: e.message,
            });
        }
        other => return Err(protocol(format!("expected hello, got {other:?}"))),
    };
    if !my_hello.compatible_with(&server_hello) {
        return Err(SyncError::Incompatible {
            local: PROTOCOL_VERSION,
            remote: server_hello.protocol,
        });
    }

    // Both sides are authenticated; remember this peer by address.
    with_conn(pool, |conn| {
        peers::trust(
            conn,
            &remote_key,
            &server_hello.node_id,
            &server_hello.name,
            &addr.to_string(),
        )
        .map_err(SyncError::from)
    })?;

    // Reconcile: compare bucket hashes, then versions where they differ.
    let (versions, mine_hashes) = with_conn(pool, |conn| {
        let versions = dbsync::note_versions(conn).map_err(SyncError::from)?;
        Ok(dbsync::bucket_hashes_pair(versions))
    })?;
    send_msg(
        &mut channel,
        &mut stream,
        &ClientMsg::Hashes(mine_hashes.clone()),
    )
    .await?;
    let their_hashes: Vec<[u8; 32]> =
        match recv_msg::<_, ServerMsg>(&mut channel, &mut stream).await? {
            ServerMsg::Hashes(h) => h,
            ServerMsg::Err(e) => {
                return Err(SyncError::Remote {
                    code: e.code,
                    message: e.message,
                });
            }
            other => return Err(protocol(format!("expected hashes, got {other:?}"))),
        };
    let differing = dbsync::differing_buckets(&mine_hashes, &their_hashes);
    send_msg(
        &mut channel,
        &mut stream,
        &ClientMsg::DiffReq {
            buckets: differing.clone(),
            versions: dbsync::versions_in(&versions, &differing),
        },
    )
    .await?;
    let diff: Diff = match recv_msg::<_, ServerMsg>(&mut channel, &mut stream).await? {
        ServerMsg::Diff(d) => d,
        ServerMsg::Err(e) => {
            return Err(SyncError::Remote {
                code: e.code,
                message: e.message,
            });
        }
        other => return Err(protocol(format!("expected diff, got {other:?}"))),
    };

    // Push in batches, waiting for each to be applied.
    let mut sent: u32 = 0;
    let mut to_push = diff.push.clone();
    while !to_push.is_empty() {
        let (notes, consumed) = with_conn(pool, |conn| {
            dbsync::load_notes(conn, &to_push, BATCH_BUDGET).map_err(SyncError::from)
        })?;
        if consumed == 0 {
            tracing::warn!("a note exceeded the transfer budget and was skipped");
            to_push.remove(0);
            continue;
        }
        sent = sent.saturating_add(u32::try_from(consumed).unwrap_or(u32::MAX));
        to_push = to_push[consumed..].to_vec();
        send_notes::<_, Client>(&mut channel, &mut stream, notes).await?;
        send_msg(&mut channel, &mut stream, &ClientMsg::NotesEnd).await?;
        match recv_msg::<_, ServerMsg>(&mut channel, &mut stream).await? {
            ServerMsg::Ack(_) => {}
            ServerMsg::Err(e) => {
                return Err(SyncError::Remote {
                    code: e.code,
                    message: e.message,
                });
            }
            other => return Err(protocol(format!("expected ack, got {other:?}"))),
        }
    }

    // Pull in batches until the server has nothing newer left.
    let mut received: u32 = 0;
    let mut to_pull = diff.pull.clone();
    while !to_pull.is_empty() {
        send_msg(&mut channel, &mut stream, &ClientMsg::Pull(to_pull.clone())).await?;
        let (notes, consumed) = recv_notes(&mut channel, &mut stream, true).await?;
        if consumed == 0 {
            break;
        }
        let consumed = usize::try_from(consumed).unwrap_or(usize::MAX);
        to_pull = to_pull[consumed.min(to_pull.len())..].to_vec();
        let applied = with_conn(pool, |conn| {
            dbsync::apply_notes(conn, &notes).map_err(SyncError::from)
        })?;
        received += applied.applied;
    }

    send_msg(&mut channel, &mut stream, &ClientMsg::Done).await?;
    with_conn(pool, |conn| {
        peers::touch(conn, &remote_key).map_err(SyncError::from)
    })?;
    let _ = stream.shutdown().await;

    Ok(format!("sync ok: sent {sent}, received {received} note(s)"))
}

#[cfg(test)]
mod tests {
    use super::*;

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
