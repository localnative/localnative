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

//! Types that cross the network during peer sync.
//!
//! They are deliberately separate from the API types in [`crate::db::models`]:
//! front-ends can change what they see without changing what peers exchange,
//! and a new wire shape means a new [`PROTOCOL_VERSION`], negotiated in the
//! handshake, rather than a schema-version coincidence.

use crate::db::is_valid_token;
use crate::error::ValidationError;
use serde::{Deserialize, Serialize};

/// Sync protocol spoken by this build.
pub const PROTOCOL_VERSION: u32 = 2;
/// Oldest protocol this build can still sync with.
pub const MIN_PROTOCOL_VERSION: u32 = 2;

/// Maximum size of a text field in a received note (1 MB).
pub const MAX_TEXT_FIELD: usize = 1_048_576;
/// Maximum size of a received note's annotations (10 MB).
pub const MAX_ANNOTATIONS: usize = 10_485_760;
/// Maximum size of a received note's `created_at`.
pub const MAX_CREATED_AT: usize = 64;

/// A note as exchanged between peers. `annotations` travels as raw bytes.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Default)]
pub struct NoteV1 {
    pub uuid4: String,
    pub title: String,
    pub url: String,
    pub tags: String,
    pub description: String,
    pub comments: String,
    pub annotations: Vec<u8>,
    pub created_at: String,
    pub is_public: bool,
    pub metadata: String,
    pub updated_at: String,
    pub deleted: bool,
}

impl NoteV1 {
    /// Reject notes a well-behaved peer could not have produced: malformed
    /// UUIDs, oversized fields, and last-write-wins tokens that are malformed
    /// or claim to be implausibly far in the future (which would otherwise
    /// outrank every later edit forever).
    pub fn validate(&self, now_ms: u64) -> Result<(), ValidationError> {
        uuid::Uuid::parse_str(&self.uuid4).map_err(|_| ValidationError::InvalidUuid)?;
        if !is_valid_token(&self.updated_at, now_ms) {
            return Err(ValidationError::InvalidToken);
        }
        for (field, value) in [
            ("title", &self.title),
            ("url", &self.url),
            ("tags", &self.tags),
            ("description", &self.description),
            ("comments", &self.comments),
            ("metadata", &self.metadata),
        ] {
            if value.len() > MAX_TEXT_FIELD {
                return Err(ValidationError::FieldTooLarge { field });
            }
        }
        if self.annotations.len() > MAX_ANNOTATIONS {
            return Err(ValidationError::FieldTooLarge {
                field: "annotations",
            });
        }
        if self.created_at.len() > MAX_CREATED_AT {
            return Err(ValidationError::FieldTooLarge {
                field: "created_at",
            });
        }
        Ok(())
    }

    /// Approximate encoded size, used to budget batches.
    pub fn encoded_len(&self) -> usize {
        self.uuid4.len()
            + self.title.len()
            + self.url.len()
            + self.tags.len()
            + self.description.len()
            + self.comments.len()
            + self.annotations.len()
            + self.created_at.len()
            + self.metadata.len()
            + self.updated_at.len()
            + 64
    }
}

/// Identity and capabilities, exchanged inside the encrypted handshake.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Hello {
    pub protocol: u32,
    pub min_protocol: u32,
    pub node_id: String,
    pub name: String,
    /// Informational only: each device migrates its own schema.
    pub schema: String,
}

impl Hello {
    /// Whether the two ends share a protocol version.
    pub fn compatible_with(&self, other: &Hello) -> bool {
        other.protocol >= self.min_protocol && self.protocol >= other.min_protocol
    }
}

/// The server's decision after the handshake, sent as the first encrypted
/// message. Receiving it also proves the server held the pairing code.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Verdict {
    pub accepted: bool,
    pub reason: Option<String>,
}

/// Which notes each side should send after comparing versions.
#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq, Eq)]
pub struct Diff {
    /// Notes the client holds a newer version of (or the server lacks).
    pub push: Vec<String>,
    /// Notes the server holds a newer version of (or the client lacks).
    pub pull: Vec<String>,
}

/// A batch of notes returned by `pull`, cut at a byte budget.
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Pulled {
    pub notes: Vec<NoteV1>,
    /// How many of the requested UUIDs this response covers (found or not).
    pub consumed: u32,
}

/// Outcome of applying a batch of notes.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Applied {
    /// Notes inserted or updated.
    pub applied: u32,
    /// Notes refused by validation.
    pub rejected: u32,
}

/// An error returned by a peer, with a stable code.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("{code}: {message}")]
pub struct WireError {
    pub code: String,
    pub message: String,
}

impl WireError {
    pub fn new(code: &str, message: impl Into<String>) -> Self {
        Self {
            code: code.to_string(),
            message: message.into(),
        }
    }
}
