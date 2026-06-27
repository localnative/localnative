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

//! Import (one-way last-write-wins merge) notes from another Local Native
//! database file into the local database. The source file is not modified.

use clap::{Command, arg};
use std::process;

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .init();

    let matches = Command::new("localnative-import-db")
        .about(
            "Merge notes from another database file into the local DB (one-way, last-write-wins)",
        )
        .arg(arg!(-i --input <FILE> "Source .sqlite3 file to import from").required(true))
        .get_matches();

    let src = matches.get_one::<String>("input").unwrap();

    let conn = match localnative_core::db::init_db() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error opening database: {}", e);
            process::exit(1);
        }
    };

    match localnative_core::db::queries::import_db(&conn, src) {
        Ok(merged) => eprintln!("Imported from {} ({} note(s) merged)", src, merged),
        Err(e) => {
            eprintln!("Import failed: {}", e);
            process::exit(1);
        }
    }
}
