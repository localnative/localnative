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

//! Encrypted, paired sync transport.
//!
//! Two modes, both Noise (`Noise_XX_25519_ChaChaPoly_BLAKE2s` family):
//!
//! * **Sync** — both devices already paired: Noise_XX with each side's static
//!   key known in advance, mutual authentication with no secret to type.
//! * **Pairing** — first contact: the device running the server shows a
//!   one-time code, the client types it, and it becomes a pre-shared key
//!   (`Noise_XXpsk3`). A code is valid for a few minutes and usable once; an
//!   attacker without the code cannot complete the handshake, and the
//!   handshake never confirms whether a code was right — it just fails.
//!
//! Static keys live in the `meta` table and never leave the device except in
//! the (stripped) public half exchanged during the handshake; [`crate::db`]
//! drops them from exported database copies.

use crate::db::{DbResult, meta_get, meta_set};
use blake2::{Blake2s256, Digest};
use rusqlite::Connection;
use snow::{Builder, HandshakeState, Keypair, TransportState};
use std::time::{Duration, Instant};

/// Noise pattern for syncing with an already-paired device.
const PATTERN_SYNC: &str = "Noise_XX_25519_ChaChaPoly_BLAKE2s";
/// Noise pattern for the first contact, pairing code as pre-shared key.
const PATTERN_PAIRING: &str = "Noise_XXpsk3_25519_ChaChaPoly_BLAKE2s";

/// How long a pairing code stays valid.
pub const PAIRING_WINDOW: Duration = Duration::from_secs(5 * 60);

/// Pairing codes use 16 characters from an alphabet without look-alikes
/// (no 0/O, 1/I), grouped for typing: `ABCD-EFGH-JKMN-PQRS`.
const CODE_ALPHABET: &[u8] = b"ABCDEFGHJKLMNPQRSTUVWXYZ23456789";
pub const CODE_LEN: usize = 16;

/// Bytes of plaintext a Noise message may carry (snow caps messages at 64 KiB;
/// leave room for the auth tag and length prefix).
pub const MAX_PAYLOAD: usize = 60_000;
/// Largest note batch sent in one message.
pub const BATCH_BUDGET: usize = 512 * 1024;

// ── Static keys ───────────────────────────────────────────────────────────

fn snow_db_error(e: snow::Error) -> crate::error::DatabaseError {
    crate::error::DatabaseError::IoError(std::io::Error::other(e.to_string()))
}

/// This device's Noise static keypair, created on first use.
fn keypair(conn: &Connection) -> DbResult<Keypair> {
    if let (Some(private), Some(public)) = (
        meta_get(conn, "sync_private_key")?,
        meta_get(conn, "sync_public_key")?,
    ) && let (Ok(private), Ok(public)) = (hex::decode(private), hex::decode(public))
    {
        return Ok(Keypair { private, public });
    }
    let params: snow::params::NoiseParams = PATTERN_SYNC.parse().expect("valid noise pattern");
    let pair = Builder::new(params)
        .generate_keypair()
        .map_err(snow_db_error)?;
    meta_set(conn, "sync_private_key", &hex::encode(&pair.private))?;
    meta_set(conn, "sync_public_key", &hex::encode(&pair.public))?;
    Ok(pair)
}

/// The public half of this device's static keypair, hex-encoded.
pub fn public_key_hex(conn: &Connection) -> DbResult<String> {
    Ok(hex::encode(keypair(conn)?.public))
}

/// The private half of this device's static keypair. Callers load it in a
/// synchronous span and keep only the bytes across any await.
pub(crate) fn static_private_key(conn: &Connection) -> DbResult<Vec<u8>> {
    Ok(keypair(conn)?.private)
}

// ── Pairing codes ─────────────────────────────────────────────────────────

/// Normalize a code as typed (case-insensitive, ignore spaces and dashes).
fn normalize_code(code: &str) -> String {
    code.chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .map(|c| c.to_ascii_uppercase())
        .collect()
}

/// A freshly generated pairing code, formatted for display.
pub fn generate_code() -> String {
    let mut raw = [0u8; CODE_LEN];
    getrandom::fill(&mut raw).expect("system randomness is available");
    let code: String = raw
        .iter()
        .map(|b| char::from(CODE_ALPHABET[usize::from(*b) % CODE_ALPHABET.len()]))
        .collect();
    format!(
        "{}-{}-{}-{}",
        &code[0..4],
        &code[4..8],
        &code[8..12],
        &code[12..16]
    )
}

/// Derive the pre-shared key from a pairing code. Includes a domain-separating
/// prefix so the PSK can never collide with a key derived another way.
fn psk_from_code(code: &str) -> [u8; 32] {
    let normalized = normalize_code(code);
    let mut hasher = Blake2s256::new();
    hasher.update(b"localnative-pairing-v1");
    hasher.update(normalized.as_bytes());
    hasher.finalize().into()
}

/// The pairing code a server is currently accepting, and until when.
/// Codes are held in memory only — never written to the database.
#[derive(Clone)]
pub struct ActiveCode {
    pub code: String,
    pub expires: Instant,
}

impl ActiveCode {
    /// Start accepting `code` for the next [`PAIRING_WINDOW`].
    pub fn fresh() -> Self {
        Self {
            code: generate_code(),
            expires: Instant::now() + PAIRING_WINDOW,
        }
    }

    pub fn expired(&self) -> bool {
        Instant::now() >= self.expires
    }

    /// Whether `code`, as typed, matches.
    pub fn matches(&self, code: &str) -> bool {
        normalize_code(code) == normalize_code(&self.code)
    }
}

// ── Handshakes ────────────────────────────────────────────────────────────

/// Errors from the secure layer, mapped by `rpc` onto [`crate::error::SyncError`].
pub type SecureResult<T> = Result<T, crate::error::SyncError>;

fn handshake_error(e: snow::Error) -> crate::error::SyncError {
    crate::error::SyncError::Handshake(e.to_string())
}

/// The client side of a sync handshake with a known server key.
fn client_sync_state(private_key: &[u8], server_public: &[u8]) -> SecureResult<HandshakeState> {
    let params: snow::params::NoiseParams = PATTERN_SYNC.parse().expect("valid noise pattern");
    Builder::new(params)
        .local_private_key(private_key)
        .and_then(|b| b.remote_public_key(server_public))
        .map_err(handshake_error)?
        .build_initiator()
        .map_err(handshake_error)
}

/// The client side of a pairing handshake with a code.
fn client_pairing_state(private_key: &[u8], code: &str) -> SecureResult<HandshakeState> {
    let params: snow::params::NoiseParams = PATTERN_PAIRING.parse().expect("valid noise pattern");
    let psk = psk_from_code(code);
    Builder::new(params)
        .local_private_key(private_key)
        .and_then(|b| b.psk(3, &psk))
        .map_err(handshake_error)?
        .build_initiator()
        .map_err(handshake_error)
}

/// The server side for either mode.
fn server_state(private_key: &[u8], pairing: Option<&ActiveCode>) -> SecureResult<HandshakeState> {
    if let Some(code) = pairing {
        let params: snow::params::NoiseParams =
            PATTERN_PAIRING.parse().expect("valid noise pattern");
        let psk = psk_from_code(&code.code);
        Builder::new(params)
            .psk(3, &psk)
            .map_err(handshake_error)?
            .local_private_key(private_key)
            .map_err(handshake_error)?
            .build_responder()
            .map_err(handshake_error)
    } else {
        let params: snow::params::NoiseParams = PATTERN_SYNC.parse().expect("valid noise pattern");
        Builder::new(params)
            .local_private_key(private_key)
            .map_err(handshake_error)?
            .build_responder()
            .map_err(handshake_error)
    }
}

/// A finished Noise handshake, before transport messages flow.
pub struct SecureHandshake {
    state: HandshakeState,
    /// Whether the pairing pattern (code as pre-shared key) was used. Only a
    /// pairing handshake can introduce a previously unknown device; a sync
    /// handshake with an untrusted static key must be rejected.
    pairing: bool,
}

impl SecureHandshake {
    /// The static key the peer presented, once the handshake completes.
    pub fn remote_static(&self) -> Option<&[u8]> {
        self.state.get_remote_static()
    }

    /// Whether the peer proved a pairing code (rather than a known key).
    pub fn pairing_used(&self) -> bool {
        self.pairing
    }

    pub fn into_transport(self) -> SecureResult<SecureChannel> {
        Ok(SecureChannel {
            state: self.state.into_transport_mode().map_err(handshake_error)?,
        })
    }
}

/// An encrypted, authenticated channel between paired devices.
pub struct SecureChannel {
    state: TransportState,
}

impl SecureChannel {
    /// Client side: run the three-message Noise_XX (or XXpsk3) handshake over
    /// `io` with this device's static `private_key`. `server_public` selects
    /// sync mode; a code selects pairing mode.
    pub async fn connect<IO>(
        private_key: &[u8],
        io: &mut IO,
        pairing_code: Option<&str>,
        server_public: Option<&[u8]>,
    ) -> SecureResult<SecureHandshake>
    where
        IO: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin,
    {
        let mut state = if let Some(code) = pairing_code {
            client_pairing_state(private_key, code)?
        } else {
            let server_public = server_public.ok_or(crate::error::SyncError::NotPaired)?;
            client_sync_state(private_key, server_public)?
        };

        let mut buf = [0u8; MAX_PAYLOAD + 128];
        // XX: -> e
        let n = state
            .write_message(&[], &mut buf)
            .map_err(handshake_error)?;
        write_frame(io, &buf[..n]).await?;
        // XX: <- e, ee, s, es
        let incoming = read_frame(io).await?;
        state
            .read_message(&incoming, &mut buf)
            .map_err(handshake_error)?;
        // XX: -> s, se
        let n = state
            .write_message(&[], &mut buf)
            .map_err(handshake_error)?;
        write_frame(io, &buf[..n]).await?;
        Ok(SecureHandshake {
            state,
            pairing: pairing_code.is_some(),
        })
    }

    /// Server side: run the handshake in `pairing` or sync mode with this
    /// device's static `private_key`. The mode is chosen by a one-byte prefix
    /// the client sends in the clear — it reveals nothing but which
    /// handshake to expect.
    pub async fn accept<IO>(
        private_key: &[u8],
        io: &mut IO,
        pairing: Option<&ActiveCode>,
    ) -> SecureResult<SecureHandshake>
    where
        IO: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin,
    {
        use tokio::io::AsyncReadExt;

        let mut mode = [0u8; 1];
        io.read_exact(&mut mode).await?;
        let mut state = if mode == *b"P" {
            let code = pairing.ok_or_else(|| {
                crate::error::SyncError::Rejected(
                    "the server is not accepting new pairings right now".to_string(),
                )
            })?;
            server_state(private_key, Some(code))?
        } else {
            server_state(private_key, None)?
        };

        let mut buf = [0u8; MAX_PAYLOAD + 128];
        // XX: <- e
        let incoming = read_frame(io).await?;
        state
            .read_message(&incoming, &mut buf)
            .map_err(handshake_error)?;
        // XX: -> e, ee, s, es
        let n = state
            .write_message(&[], &mut buf)
            .map_err(handshake_error)?;
        write_frame(io, &buf[..n]).await?;
        // XX: <- s, se
        let incoming = read_frame(io).await?;
        state
            .read_message(&incoming, &mut buf)
            .map_err(handshake_error)?;
        Ok(SecureHandshake {
            state,
            pairing: mode == *b"P",
        })
    }

    /// Encrypt and send one message.
    pub async fn send<IO>(&mut self, io: &mut IO, plaintext: &[u8]) -> SecureResult<()>
    where
        IO: tokio::io::AsyncWrite + Unpin,
    {
        assert!(plaintext.len() <= MAX_PAYLOAD, "caller splits into batches");
        let mut buf = vec![0u8; plaintext.len() + 128];
        let n = self
            .state
            .write_message(plaintext, &mut buf)
            .map_err(handshake_error)?;
        write_frame(io, &buf[..n]).await
    }

    /// Receive and decrypt one message.
    pub async fn recv<IO>(&mut self, io: &mut IO) -> SecureResult<Vec<u8>>
    where
        IO: tokio::io::AsyncRead + Unpin,
    {
        let incoming = read_frame(io).await?;
        let mut buf = vec![0u8; incoming.len()];
        let n = self
            .state
            .read_message(&incoming, &mut buf)
            .map_err(handshake_error)?;
        buf.truncate(n);
        Ok(buf)
    }
}

/// Length-prefixed frame I/O on the raw stream (before Noise takes over).
async fn write_frame<IO>(io: &mut IO, bytes: &[u8]) -> SecureResult<()>
where
    IO: tokio::io::AsyncWrite + Unpin,
{
    use tokio::io::AsyncWriteExt;
    let len = u32::try_from(bytes.len()).expect("frames are bounded by MAX_PAYLOAD + overhead");
    io.write_all(&len.to_le_bytes()).await?;
    io.write_all(bytes).await?;
    io.flush().await?;
    Ok(())
}

async fn read_frame<IO>(io: &mut IO) -> SecureResult<Vec<u8>>
where
    IO: tokio::io::AsyncRead + Unpin,
{
    use tokio::io::AsyncReadExt;
    let mut len = [0u8; 4];
    io.read_exact(&mut len).await?;
    // u32 -> usize is lossless on every supported target (32-bit and up).
    let len = usize::try_from(u32::from_le_bytes(len)).unwrap_or(usize::MAX);
    if len > MAX_PAYLOAD + 128 {
        return Err(crate::error::SyncError::Handshake(
            "frame larger than any Noise message".to_string(),
        ));
    }
    let mut buf = vec![0u8; len];
    io.read_exact(&mut buf).await?;
    Ok(buf)
}
