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
use thiserror::Error;

// ---------------------------------------------------------------------------
// ValidationError — field size limits, invalid UUIDs, invalid paths
// ---------------------------------------------------------------------------

#[derive(Debug, Error)]
pub enum ValidationError {
    #[error("Invalid UUID4 format")]
    InvalidUuid,
    #[error("Field exceeds maximum size: {field}")]
    FieldTooLarge { field: &'static str },
    #[error("Invalid last-write-wins token")]
    InvalidToken,
    #[error("Invalid path: {0}")]
    InvalidPath(String),
    #[error("Validation error: {0}")]
    Other(String),
}

// ---------------------------------------------------------------------------
// DatabaseError — SQLite errors, migration failures, constraint violations
// ---------------------------------------------------------------------------

#[derive(Debug, Error)]
pub enum DatabaseError {
    #[error("Rusqlite error: {0}")]
    RusqliteError(#[from] rusqlite::Error),
    #[error("Serialization error: {0}")]
    SerdeError(#[from] serde_json::Error),
    #[error("Base64 decoding error: {0}")]
    Base64Error(#[from] base64::DecodeError),
    #[error("Semver parsing error: {0}")]
    SemverError(#[from] semver::Error),
    #[error("Invalid created_at format")]
    InvalidFormat,
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("{0}")]
    Validation(#[from] ValidationError),
    #[error("SQLCipher is not compiled in; refusing to open an unencrypted database as encrypted")]
    EncryptionUnavailable,
}

/// Backward-compatible alias used throughout the crate.
pub type DbError = DatabaseError;
pub type DbResult<T> = Result<T, DatabaseError>;

// ---------------------------------------------------------------------------
// SyncError — connection, handshake, pairing and protocol failures
// ---------------------------------------------------------------------------

#[derive(Debug, Error)]
pub enum SyncError {
    #[error("Database error: {0}")]
    DbError(#[from] DatabaseError),
    #[error("Sync protocol error: {0}")]
    Protocol(String),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Address parse error: {0}")]
    AddrParseError(#[from] std::net::AddrParseError),
    #[error("{0}")]
    Validation(#[from] ValidationError),
    #[error("Secure handshake failed: {0}")]
    Handshake(String),
    #[error(
        "This device is not paired with the peer; enter the pairing code shown on the device running the sync server"
    )]
    NotPaired,
    #[error("Pairing failed: the pairing code was not accepted")]
    PairingFailed,
    #[error("Invalid pairing code: expected 16 letters or digits, e.g. ABCD-EFGH-JKMN-PQRS")]
    InvalidPairingCode,
    #[error("Peer speaks sync protocol {remote}, this device speaks {local}; update both devices")]
    Incompatible { local: u32, remote: u32 },
    #[error("Peer refused the connection: {0}")]
    Rejected(String),
    #[error("Peer reported an error ({code}): {message}")]
    Remote { code: String, message: String },
    #[error("Timed out: {0}")]
    Timeout(&'static str),
    #[error("Server configuration error: {0}")]
    ServerConfigError(String),
    #[error("Connection pool error: {0}")]
    PoolError(String),
}

/// Backward-compatible alias.
pub type RpcError = SyncError;

impl SyncError {
    /// Stable machine-readable error code for front-ends.
    pub fn code(&self) -> &'static str {
        match self {
            SyncError::DbError(e) => database_code(e),
            SyncError::Protocol(_) => "protocol",
            SyncError::IoError(_) => "io",
            SyncError::AddrParseError(_) => "invalid-address",
            SyncError::Validation(_) => "invalid-input",
            SyncError::Handshake(_) => "handshake",
            SyncError::NotPaired => "not-paired",
            SyncError::PairingFailed => "pairing-failed",
            SyncError::InvalidPairingCode => "invalid-pairing-code",
            SyncError::Incompatible { .. } => "incompatible-peer",
            SyncError::Rejected(_) => "rejected",
            SyncError::Remote { .. } => "remote",
            SyncError::Timeout(_) => "timeout",
            SyncError::ServerConfigError(_) => "server-config",
            SyncError::PoolError(_) => "database",
        }
    }
}

fn database_code(e: &DatabaseError) -> &'static str {
    match e {
        DatabaseError::Validation(_) => "invalid-input",
        DatabaseError::EncryptionUnavailable => "encryption-unavailable",
        _ => "database",
    }
}

// ---------------------------------------------------------------------------
// Error — top-level error returned by the JSON dispatcher
// ---------------------------------------------------------------------------

#[derive(Debug, Error)]
pub enum Error {
    #[error("database error: {0}")]
    Database(#[from] DatabaseError),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("address parse error: {0}")]
    AddrParse(#[from] std::net::AddrParseError),
    #[error("sync error: {0}")]
    Sync(#[from] SyncError),
    #[error("invalid command: {0}")]
    Serde(#[from] serde_json::Error),
}

/// Backward-compatible alias so `lib.rs` keeps compiling with `ProcessError`.
pub type ProcessError = Error;

impl Error {
    /// Stable machine-readable error code for front-ends.
    pub fn code(&self) -> &'static str {
        match self {
            Error::Database(e) => database_code(e),
            Error::Io(_) => "io",
            Error::AddrParse(_) => "invalid-address",
            Error::Sync(e) => e.code(),
            Error::Serde(_) => "invalid-command",
        }
    }
}
