/*
    Local Native
    Copyright (C) 2021  Yi Wang

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

use super::utils;
use super::uuid::Uuid;
use rusqlite::{Connection, Result, ToSql};

pub fn drop_ssb_table(conn: &Connection) -> Result<()> {
    eprintln!("to_0_5_0 drop_ssb_table");
    conn.execute_batch(
        "BEGIN;
        drop table if exists ssb;
        COMMIT;",
    )
    .unwrap();
    Ok(())
}
