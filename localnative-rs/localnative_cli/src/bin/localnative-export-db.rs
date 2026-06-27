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

//! Export a clean, single-file copy of the Local Native database (`VACUUM INTO`).

use clap::{Command, arg};
use std::process;

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .init();

    let matches = Command::new("localnative-export-db")
        .about("Export a standalone copy of the database (VACUUM INTO, no -wal/-shm sidecars)")
        .arg(arg!(-o --output <FILE> "Destination .sqlite3 file path").required(true))
        .get_matches();

    let dest = matches.get_one::<String>("output").unwrap();

    let conn = match localnative_core::db::init_db() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error opening database: {}", e);
            process::exit(1);
        }
    };

    match localnative_core::db::queries::export_db(&conn, dest) {
        Ok(()) => eprintln!("Exported database to {}", dest),
        Err(e) => {
            eprintln!("Export failed: {}", e);
            process::exit(1);
        }
    }
}
