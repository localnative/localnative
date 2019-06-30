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

extern crate rusqlite;
extern crate semver;
use rusqlite::types::ToSql;
use rusqlite::{Connection, NO_PARAMS};
use semver::Version;
use std::io;
// version to upgrade to
const VERSION: &'static str = "0.4.0";
mod to_0_4_0;

pub fn upgrade(conn: &Connection) -> Result<&str, io::Error> {
    if Version::parse(&check_version(conn)) < Version::parse("0.4.0") {
        to_0_4_0::run(conn);
    }
    eprintln!("upgraded to {}", VERSION);
    Ok(VERSION)
}

fn check_version(conn: &Connection) -> String {
    if check_table_exist(conn, "meta") {
        eprintln!("0.4.0");
        "0.4.0".to_string()
    } else {
        eprintln!("0.3.0");
        "0.3.10".to_string()
    }
    //    let mut stmt = conn.prepare("SELECT version FROM meta").unwrap();
    //    let version = match stmt.query_row(NO_PARAMS, |row| Ok( row.get(0)?)).unwrap();
    //    version
}

fn check_table_exist(conn: &Connection, table_name: &str) -> bool {
    let mut stmt = conn
        .prepare("SELECT name FROM sqlite_master WHERE type='table' AND name= :table_name")
        .unwrap();
    match stmt.query_row_named(&[(":table_name", &table_name)], |row| {
        Ok(OneString { s: row.get(0)? })
    }) {
        Ok(rs) => true,
        Err(_) => false,
    }
}

pub struct OneString {
    pub s: String,
}
