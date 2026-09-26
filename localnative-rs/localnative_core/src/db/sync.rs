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

//! Database side of peer sync: version sets, bucket hashes for cheap
//! reconciliation, and loading/applying notes in wire form.
//!
//! Reconciliation: every `(uuid4, updated_at)` pair — tombstones included —
//! falls into one of [`BUCKETS`] buckets by a hash of its UUID. Peers first
//! compare one hash per bucket and then exchange versions only for buckets
//! that differ, so two devices that are already in sync exchange 8 KB instead
//! of their whole version lists.

use super::{DbResult, TOMBSTONE_CONTENT, meta_get, normalize_tags, now_millis, observe_millis};
use crate::wire::{Applied, Diff, NoteV1};
use blake2::{Blake2s256, Digest};
use rusqlite::types::ValueRef;
use rusqlite::{Connection, OptionalExtension};
use std::collections::{HashMap, HashSet};

/// Number of reconciliation buckets.
pub const BUCKETS: usize = 256;

pub type Version = (String, String);

pub fn get_meta_version(conn: &Connection) -> DbResult<String> {
    Ok(meta_get(conn, "version")?.unwrap_or_else(|| "0.3.10".to_string()))
}

/// `(uuid4, updated_at)` for every note, **including tombstones**, so that
/// both new notes and deletions are advertised to peers during sync.
pub fn note_versions(conn: &Connection) -> DbResult<Vec<Version>> {
    let mut stmt = conn.prepare("SELECT uuid4, updated_at FROM note ORDER BY uuid4")?;
    let rows = stmt
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?
        .collect::<Result<Vec<Version>, rusqlite::Error>>()?;
    Ok(rows)
}

/// The reconciliation bucket of a note.
pub fn bucket_of(uuid4: &str) -> u16 {
    u16::from(Blake2s256::digest(uuid4.as_bytes())[0])
}

/// One hash per bucket over the sorted versions it contains.
pub fn bucket_hashes(versions: &[Version]) -> Vec<[u8; 32]> {
    let mut buckets: Vec<Vec<&Version>> = vec![Vec::new(); BUCKETS];
    for version in versions {
        buckets[usize::from(bucket_of(&version.0))].push(version);
    }
    buckets
        .into_iter()
        .map(|mut bucket| {
            bucket.sort();
            let mut hasher = Blake2s256::new();
            for (uuid4, updated_at) in bucket {
                hasher.update(uuid4.as_bytes());
                hasher.update([0]);
                hasher.update(updated_at.as_bytes());
                hasher.update([0]);
            }
            hasher.finalize().into()
        })
        .collect()
}

/// `(versions, bucket_hashes(&versions))` in one step for callers that need
/// both out of a single database span.
pub fn bucket_hashes_pair(versions: Vec<Version>) -> (Vec<Version>, Vec<[u8; 32]>) {
    let hashes = bucket_hashes(&versions);
    (versions, hashes)
}

/// Indexes of the buckets whose hashes differ.
pub fn differing_buckets(mine: &[[u8; 32]], theirs: &[[u8; 32]]) -> Vec<u16> {
    (0..BUCKETS)
        .filter(|&i| mine.get(i) != theirs.get(i))
        .filter_map(|i| u16::try_from(i).ok())
        .collect()
}

/// The versions that fall into `buckets`.
pub fn versions_in(versions: &[Version], buckets: &[u16]) -> Vec<Version> {
    let wanted: HashSet<u16> = buckets.iter().copied().collect();
    versions
        .iter()
        .filter(|(uuid4, _)| wanted.contains(&bucket_of(uuid4)))
        .cloned()
        .collect()
}

/// Compare the client's versions inside `buckets` with this database's:
/// the client pushes what it holds newer or alone, and pulls the rest.
pub fn diff_versions(conn: &Connection, buckets: &[u16], client: Vec<Version>) -> DbResult<Diff> {
    let server: HashMap<String, String> = versions_in(&note_versions(conn)?, buckets)
        .into_iter()
        .collect();
    let client: HashMap<String, String> = client.into_iter().collect();

    let mut push: Vec<String> = client
        .iter()
        .filter(|(uuid4, theirs)| server.get(*uuid4).is_none_or(|ours| *theirs > ours))
        .map(|(uuid4, _)| uuid4.clone())
        .collect();
    let mut pull: Vec<String> = server
        .iter()
        .filter(|(uuid4, ours)| client.get(*uuid4).is_none_or(|theirs| *ours > theirs))
        .map(|(uuid4, _)| uuid4.clone())
        .collect();
    push.sort();
    pull.sort();
    Ok(Diff { push, pull })
}

fn annotations_bytes(value: ValueRef<'_>) -> Vec<u8> {
    match value {
        ValueRef::Text(bytes) | ValueRef::Blob(bytes) => bytes.to_vec(),
        _ => Vec::new(),
    }
}

/// One note in wire form, tombstones included.
pub fn get_wire_note(conn: &Connection, uuid4: &str) -> DbResult<Option<NoteV1>> {
    Ok(conn
        .query_row(
            "SELECT uuid4, title, url, tags, description, comments, annotations, created_at,
                    is_public, metadata, updated_at, deleted
             FROM note WHERE uuid4 = ?1",
            rusqlite::params![uuid4],
            |row| {
                Ok(NoteV1 {
                    uuid4: row.get(0)?,
                    title: row.get(1)?,
                    url: row.get(2)?,
                    tags: row.get(3)?,
                    description: row.get(4)?,
                    comments: row.get(5)?,
                    annotations: annotations_bytes(row.get_ref(6)?),
                    created_at: row.get(7)?,
                    is_public: row.get(8)?,
                    metadata: row.get(9)?,
                    updated_at: row.get(10)?,
                    deleted: row.get(11)?,
                })
            },
        )
        .optional()?)
}

/// Load notes for `uuids` in order until `byte_budget` is reached (always at
/// least one). Returns the notes and how many UUIDs were consumed; UUIDs that
/// no longer exist are consumed without a note.
pub fn load_notes(
    conn: &Connection,
    uuids: &[String],
    byte_budget: usize,
) -> DbResult<(Vec<NoteV1>, usize)> {
    let mut notes = Vec::new();
    let mut used = 0;
    let mut consumed = 0;
    for uuid4 in uuids {
        if let Some(note) = get_wire_note(conn, uuid4)? {
            let size = note.encoded_len();
            if !notes.is_empty() && used + size > byte_budget {
                break;
            }
            used += size;
            notes.push(note);
        }
        consumed += 1;
    }
    Ok((notes, consumed))
}

/// Apply notes received from a peer in one transaction, last write wins.
///
/// Each note is validated first; a note that fails validation is counted as
/// rejected and skipped. Tombstones are stored without content whatever the
/// peer sent, tags are normalized, and the local clock advances past every
/// accepted token so later local edits outrank them.
pub fn apply_notes(conn: &Connection, notes: &[NoteV1]) -> DbResult<Applied> {
    let now = now_millis();
    let tx = conn.unchecked_transaction()?;
    let mut outcome = Applied::default();
    let mut newest = 0;
    for note in notes {
        if let Err(e) = note.validate(now) {
            tracing::warn!(uuid4 = %note.uuid4, %e, "rejected note from peer");
            outcome.rejected += 1;
            continue;
        }
        if let Some(ms) = super::token_millis(&note.updated_at) {
            newest = newest.max(ms);
        }
        let changed = if note.deleted {
            tx.execute(
                &format!(
                    "INSERT INTO note (uuid4, title, url, tags, description, comments, annotations,
                                       created_at, is_public, metadata, updated_at, deleted)
                     VALUES (?1, '', '', '', '', '', '', ?2, ?3, '{{}}', ?4, 1)
                     ON CONFLICT(uuid4) DO UPDATE SET {TOMBSTONE_CONTENT},
                         created_at = excluded.created_at, is_public = excluded.is_public,
                         updated_at = excluded.updated_at, deleted = 1
                     WHERE excluded.updated_at > note.updated_at"
                ),
                rusqlite::params![note.uuid4, note.created_at, note.is_public, note.updated_at],
            )?
        } else {
            let metadata = if note.metadata.is_empty() {
                "{}"
            } else {
                note.metadata.as_str()
            };
            tx.execute(
                "INSERT INTO note (uuid4, title, url, tags, description, comments, annotations,
                                   created_at, is_public, metadata, updated_at, deleted)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, 0)
                 ON CONFLICT(uuid4) DO UPDATE SET
                     title = excluded.title, url = excluded.url, tags = excluded.tags,
                     description = excluded.description, comments = excluded.comments,
                     annotations = excluded.annotations, created_at = excluded.created_at,
                     is_public = excluded.is_public, metadata = excluded.metadata,
                     updated_at = excluded.updated_at, deleted = 0
                 WHERE excluded.updated_at > note.updated_at",
                rusqlite::params![
                    note.uuid4,
                    note.title,
                    note.url,
                    normalize_tags(&note.tags),
                    note.description,
                    note.comments,
                    note.annotations,
                    note.created_at,
                    note.is_public,
                    metadata,
                    note.updated_at
                ],
            )?
        };
        if changed > 0 {
            outcome.applied += 1;
        }
    }
    if newest > 0 {
        observe_millis(&tx, newest)?;
    }
    tx.commit()?;
    Ok(outcome)
}
