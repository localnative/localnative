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
use rusqlite::Connection;
// version to upgrade to
const VERSION: &str = "0.5.0";
mod to_0_4_0;
mod to_0_5_0;
mod utils;
use crate::OneString;

fn set_meta_version(conn: &Connection, version: &str) -> anyhow::Result<()> {
    conn.execute(
        "
        UPDATE meta SET meta_value = ?1
        WHERE meta_key = 'version';",
        &[version],
    )?;
    Ok(())
}

pub fn upgrade(conn: &Connection) -> anyhow::Result<&str> {
    if get_meta_is_upgrading(conn)? {
        eprintln!("is_upgrading");
        Err(anyhow::anyhow!("is_upgrading"))
    } else {
        if Version::parse(&get_meta_version(conn)?)? < Version::parse("0.4.0")? {
            to_0_4_0::migrate_schema(conn).unwrap();
        }
        if Version::parse(&get_meta_version(conn)?)? == Version::parse("0.4.0")? {
            to_0_4_0::migrate_note(conn).unwrap();
            set_meta_version(conn, "0.4.1")?;
        }
        if Version::parse(&get_meta_version(conn)?)? == Version::parse("0.4.1")? {
            set_meta_version(conn, "0.4.2")?;
        }
        if Version::parse(&get_meta_version(conn)?)? == Version::parse("0.4.2")? {
            to_0_5_0::drop_ssb_table(conn).unwrap();
            set_meta_version(conn, VERSION)?;
        }
        eprintln!("upgraded to {}", VERSION);
        Ok(VERSION)
    }
}

fn get_meta_is_upgrading(conn: &Connection) -> anyhow::Result<bool> {
    let mut stmt = conn.prepare("SELECT meta_value FROM meta where meta_key = 'is_upgrading' ")?;
    match stmt.query_row([], |row| Ok(OneString { s: row.get(0)? })) {
        Ok(is_upgrading) => {
            if is_upgrading.s == "1" {
                eprintln!("get_meta_is_upgrading: true");
                Ok(true)
            } else {
                eprintln!("get_meta_is_upgrading: false");
                Ok(false)
            }
        }
        Err(_) => Ok(false),
    }
}

pub fn get_meta_version(conn: &Connection) -> anyhow::Result<String> {
    let mut stmt = conn.prepare("SELECT meta_value FROM meta where meta_key = 'version' ")?;
    match stmt.query_row([], |row| Ok(OneString { s: row.get(0)? })) {
        Ok(version) => {
            eprintln!("get_meta_version {}", version.s);
            Ok(version.s)
        }
        Err(_) => {
            eprintln!("get_meta_version: no meta table, default to earliest version 0.3.10");
            Ok("0.3.10".to_string())
        }
    }
}
