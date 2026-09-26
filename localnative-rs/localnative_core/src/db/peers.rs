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

//! Devices this database has been paired with. Sync only ever talks to a
//! device whose static public key is in this table (see [`crate::secure`]).

use super::DbResult;
use rusqlite::{Connection, OptionalExtension};
use serde::Serialize;

/// A paired device.
#[derive(Serialize, Debug, Clone, PartialEq, Eq)]
pub struct Peer {
    /// Hex-encoded Noise static public key.
    pub public_key: String,
    pub node_id: String,
    pub name: String,
    /// Last known `host:port`, a hint that is refreshed on every sync.
    pub addr: String,
    pub paired_at: String,
    pub last_sync_at: Option<String>,
}

fn now() -> String {
    chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string()
}

/// Whether the device with this public key is paired.
pub fn is_trusted(conn: &Connection, public_key: &[u8]) -> DbResult<bool> {
    Ok(conn
        .query_row(
            "SELECT 1 FROM peer WHERE public_key = ?1",
            rusqlite::params![hex::encode(public_key)],
            |_| Ok(()),
        )
        .optional()?
        .is_some())
}

/// The public key of the device last known at `addr`, if paired.
pub fn find_by_addr(conn: &Connection, addr: &str) -> DbResult<Option<Vec<u8>>> {
    let key: Option<String> = conn
        .query_row(
            "SELECT public_key FROM peer WHERE addr = ?1 ORDER BY paired_at DESC LIMIT 1",
            rusqlite::params![addr],
            |row| row.get(0),
        )
        .optional()?;
    Ok(key.and_then(|k| hex::decode(k).ok()))
}

/// Record a successful pairing (or refresh what is known about a device).
pub fn trust(
    conn: &Connection,
    public_key: &[u8],
    node_id: &str,
    name: &str,
    addr: &str,
) -> DbResult<()> {
    conn.execute(
        "INSERT INTO peer (public_key, node_id, name, addr, paired_at) VALUES (?1, ?2, ?3, ?4, ?5)
         ON CONFLICT(public_key) DO UPDATE SET
             node_id = excluded.node_id, name = excluded.name, addr = excluded.addr",
        rusqlite::params![hex::encode(public_key), node_id, name, addr, now()],
    )?;
    Ok(())
}

/// Note a completed sync with a paired device.
pub fn touch(conn: &Connection, public_key: &[u8]) -> DbResult<()> {
    conn.execute(
        "UPDATE peer SET last_sync_at = ?2 WHERE public_key = ?1",
        rusqlite::params![hex::encode(public_key), now()],
    )?;
    Ok(())
}

/// Every paired device, most recently paired first.
pub fn list(conn: &Connection) -> DbResult<Vec<Peer>> {
    let mut stmt = conn.prepare(
        "SELECT public_key, node_id, name, addr, paired_at, last_sync_at FROM peer
         ORDER BY paired_at DESC",
    )?;
    let peers = stmt
        .query_map([], |row| {
            Ok(Peer {
                public_key: row.get(0)?,
                node_id: row.get(1)?,
                name: row.get(2)?,
                addr: row.get(3)?,
                paired_at: row.get(4)?,
                last_sync_at: row.get(5)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(peers)
}

/// Unpair a device (hex public key). Returns whether it was paired.
pub fn forget(conn: &Connection, public_key_hex: &str) -> DbResult<bool> {
    Ok(conn.execute(
        "DELETE FROM peer WHERE public_key = ?1",
        rusqlite::params![public_key_hex.to_ascii_lowercase()],
    )? > 0)
}
