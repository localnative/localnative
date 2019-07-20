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
extern crate uuid;

use self::semver::Version;
use rusqlite::{Connection, NO_PARAMS};
// version to upgrade to
const VERSION: &'static str = "0.4.0";
mod to_0_4_0;
mod utils;
use self::utils::OneString;

pub fn upgrade(conn: &Connection) -> Result<&str, &str> {
    if get_meta_is_upgrading(conn) {
        eprintln!("is_upgrading");
        Err("is_upgrading")
    } else {
        if Version::parse(&get_meta_version(conn)) < Version::parse("0.4.0") {
            to_0_4_0::migrate_schema(conn).unwrap();
        }
        if Version::parse(&get_meta_version(conn)) == Version::parse("0.4.0") {
            to_0_4_0::migrate_note(conn).unwrap();
        }
        eprintln!("upgraded to {}", VERSION);
        Ok(VERSION)
    }
}

fn get_meta_is_upgrading(conn: &Connection) -> bool {
    let mut stmt = conn
        .prepare("SELECT meta_value FROM meta where meta_key = 'is_upgrading' ")
        .unwrap();
    match stmt.query_row(NO_PARAMS, |row| Ok(OneString { s: row.get(0)? })) {
        Ok(is_upgrading) => {
            if is_upgrading.s == "1" {
                eprintln!("get_meta_is_upgrading: true");
                true
            } else {
                eprintln!("get_meta_is_upgrading: false");
                false
            }
        }
        Err(_) => false,
    }
}

pub fn get_meta_version(conn: &Connection) -> String {
    let mut stmt = conn
        .prepare("SELECT meta_value FROM meta where meta_key = 'version' ")
        .unwrap();
    match stmt.query_row(NO_PARAMS, |row| Ok(OneString { s: row.get(0)? })) {
        Ok(version) => {
            eprintln!("get_meta_version {}", version.s);
            version.s
        }
        Err(_) => {
            eprintln!("get_meta_version: no meta table, default to earliest version 0.3.10");
            "0.3.10".to_string()
        }
    }
}
