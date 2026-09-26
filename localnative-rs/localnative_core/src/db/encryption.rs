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

//! Optional at-rest encryption with SQLCipher.
//!
//! Everything here requires the `encryption` cargo feature, which swaps the
//! bundled SQLite for the bundled SQLCipher build. `PRAGMA key` on a plain
//! SQLite build is accepted and silently ignored — the database stays in
//! plaintext while appearing encrypted — so [`set_encryption_key`] verifies
//! `PRAGMA cipher_version` actually answers and refuses otherwise.

use super::{CONNECTION_PRAGMAS, DatabaseError, DbResult};
use rusqlite::Connection;
use std::path::Path;

/// Apply the encryption key to an already-opened SQLCipher connection.
///
/// This **must** be the first statement executed on the connection
/// (before any other SQL), otherwise SQLCipher will treat the file as
/// plain-text and subsequent operations will fail.
pub fn set_encryption_key(conn: &Connection, key: &str) -> DbResult<()> {
    // Use PRAGMA key with single-quote escaping to avoid injection.
    conn.execute_batch(&format!("PRAGMA key = '{}';", key.replace('\'', "''")))
        .map_err(DatabaseError::from)?;
    // A plain SQLite build accepts PRAGMA key and ignores it; only SQLCipher
    // reports a cipher version. Refuse to continue on the wrong build.
    let cipher: Option<String> = conn
        .query_row("PRAGMA cipher_version", [], |row| row.get(0))
        .ok();
    if cipher.is_none() {
        return Err(DatabaseError::EncryptionUnavailable);
    }
    Ok(())
}

/// Re-key (change the passphrase of) an already-unlocked database.
///
/// The connection must have been opened and unlocked with
/// [`set_encryption_key`] first.  After this call succeeds the database
/// file on disk is re-encrypted with `new_key`.
pub fn change_encryption_key(conn: &Connection, new_key: &str) -> DbResult<()> {
    conn.execute_batch(&format!(
        "PRAGMA rekey = '{}';",
        new_key.replace('\'', "''")
    ))
    .map_err(DatabaseError::from)?;
    Ok(())
}

/// Open (or create) the database at the configured location, unlock it with
/// `key`, and run migrations. The encrypted counterpart of [`super::init_db`].
pub fn init_db_encrypted(key: &str) -> DbResult<Connection> {
    let db_path = super::db_path()?;
    tracing::info!(db_path = %db_path.display(), "opening encrypted database");
    let conn = Connection::open(&db_path)?;
    set_encryption_key(&conn, key)?;
    conn.execute_batch(CONNECTION_PRAGMAS)?;
    super::migrations::upgrade(&conn)?;
    Ok(conn)
}

/// Migrate an existing *unencrypted* database to a new *encrypted* copy.
///
/// 1. Opens `source_path` as a plain-text SQLite database.
/// 2. Creates a new encrypted database at `dest_path`.
/// 3. Copies all data using `ATTACH` + `sqlcipher_export()`.
///
/// `dest_path` must not already exist.  On success the caller can swap
/// the files and start using the encrypted database.
pub fn encrypt_existing_db(source_path: &str, dest_path: &str, key: &str) -> DbResult<()> {
    // Safety: dest must not exist yet.
    if Path::new(dest_path).exists() {
        return Err(DatabaseError::IoError(std::io::Error::new(
            std::io::ErrorKind::AlreadyExists,
            format!("destination already exists: {dest_path}"),
        )));
    }

    let source_conn = Connection::open(source_path)?;

    // Attach the (not-yet-existing) destination as an encrypted database.
    source_conn.execute_batch(&format!(
        "ATTACH DATABASE '{}' AS encrypted KEY '{}';",
        dest_path.replace('\'', "''"),
        key.replace('\'', "''"),
    ))?;

    // Copy all schema and data from main to the encrypted database.
    source_conn.execute_batch("SELECT sqlcipher_export('encrypted');")?;

    source_conn.execute_batch("DETACH DATABASE encrypted;")?;

    Ok(())
}
