use serde::{de, ser, Deserialize, Serialize};
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
}

/// Backward-compatible alias used throughout the crate.
pub type DbError = DatabaseError;
pub type DbResult<T> = Result<T, DatabaseError>;

// ---------------------------------------------------------------------------
// SyncError — RPC errors, connection failures, version mismatches
// ---------------------------------------------------------------------------

#[derive(Debug, Error)]
pub enum SyncError {
    #[error("Database error: {0}")]
    DbError(#[from] DatabaseError),
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
    #[error("{0}")]
    Validation(#[from] ValidationError),
    #[error("Server configuration error: {0}")]
    ServerConfigError(String),
    #[error("Rate limited: too many requests")]
    RateLimited,
}

/// Backward-compatible alias so `rpc.rs` keeps compiling with `RpcError`.
pub type RpcError = SyncError;

// ---------------------------------------------------------------------------
// Error — top-level error that wraps the domain errors
// ---------------------------------------------------------------------------

#[derive(Debug, Error)]
pub enum Error {
    #[error("database error: {0}")]
    Database(#[from] DatabaseError),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("address parse error: {0}")]
    AddrParse(#[from] std::net::AddrParseError),
    #[error("rpc error: {0}")]
    Rpc(#[from] tarpc::client::RpcError),
    #[error("sync error: {0}")]
    Sync(#[from] SyncError),
    #[error("serialization error: {0}")]
    Serde(#[from] serde_json::Error),
    #[error("Process error (serialized): {0}")]
    SerializedErr(String),
}

/// Backward-compatible alias so `lib.rs` keeps compiling with `ProcessError`.
pub type ProcessError = Error;

// ---------------------------------------------------------------------------
// Serde impls — serialize any error as `{"message": "..."}`, deserialize back
// into the `SerializedErr` variant.
// ---------------------------------------------------------------------------

#[derive(Serialize, Deserialize)]
struct SerdeHelper {
    message: String,
}

macro_rules! impl_error_serialize_deserialize {
    ($($error_type:ty => $serialized_err:ident),*) => {
        $(
            impl Serialize for $error_type {
                fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                where
                    S: ser::Serializer,
                {
                    let message = self.to_string();
                    SerdeHelper { message }.serialize(serializer)
                }
            }

            impl<'de> Deserialize<'de> for $error_type {
                fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
                where
                    D: de::Deserializer<'de>,
                {
                    let error = SerdeHelper::deserialize(deserializer)?;
                    Ok(Self::$serialized_err(error.message))
                }
            }
        )*
    };
}

impl_error_serialize_deserialize!(SyncError => SerializedErr, Error => SerializedErr);
