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
use crate::OneString;
use rusqlite::Connection;

pub fn check_table_exist(conn: &Connection, table_name: &str) -> anyhow::Result<bool> {
    let mut stmt =
        conn.prepare("SELECT name FROM sqlite_master WHERE type='table' AND name= :table_name")?;
    match stmt.query_row(&[(":table_name", &table_name)], |row| {
        Ok(OneString { s: row.get(0)? })
    }) {
        Ok(rs) => {
            if rs.s == table_name {
                Ok(true)
            } else {
                Err(anyhow::anyhow!(
                    "check_table_exist returned table name not match: {}",
                    table_name
                ))
            }
        }
        Err(_) => Ok(false),
    }
}
