/*
    Local Native
    Copyright (C) 2019  Yi Wang

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

// rename note to _note_0_3
// create new note table with new uuid4 column
// insert each record to note from _note_0_3 with newly generated uuid4 value
// create meta table
// set version 0.4.0

extern crate rusqlite;
use rusqlite::{params, Connection, Result};

pub fn run(conn: &Connection) -> Result<()> {
    create_meta_table(conn)?;
    set_version(conn)?;
    Ok(())
}

fn create_meta_table(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE meta (
                  meta_key        TEXT PRIMARY KEY,
                  meta_value      TEXT NOT NULL
                  )",
        params![],
    )?;
    Ok(())
}

fn set_version(conn: &Connection) -> Result<()> {
    conn.execute(
        "INSERT INTO meta (
                  meta_key,
                  meta_value
                  )
                  VALUES('version','0.4.0')
                  ",
        params![],
    )?;
    Ok(())
}
