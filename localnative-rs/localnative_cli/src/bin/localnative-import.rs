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

use clap::{arg, Command};
use localnative_core::db;
use localnative_core::import;

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .init();

    let matches = Command::new("localnative-import")
        .about("Import notes from external services")
        .arg(
            arg!(-f --format <FORMAT> "Import format: pocket or omnivore")
                .required(true),
        )
        .arg(
            arg!(<FILE> "Path to the export file")
                .required(true),
        )
        .get_matches();

    let format = matches.get_one::<String>("format").unwrap();
    let file_path = matches.get_one::<String>("FILE").unwrap();

    let content = match std::fs::read_to_string(file_path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error reading file '{}': {}", file_path, e);
            std::process::exit(1);
        }
    };

    let notes = match format.as_str() {
        "pocket" => import::parse_pocket_html(&content),
        "omnivore" => match import::parse_omnivore_json(&content) {
            Ok(notes) => notes,
            Err(e) => {
                eprintln!("Error parsing Omnivore JSON: {}", e);
                std::process::exit(1);
            }
        },
        _ => {
            eprintln!(
                "Unknown format '{}'. Supported formats: pocket, omnivore",
                format
            );
            std::process::exit(1);
        }
    };

    eprintln!("Parsed {} notes from {} file", notes.len(), format);

    let conn = match db::init_db() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error opening database: {}", e);
            std::process::exit(1);
        }
    };

    match import::import_notes(&conn, notes) {
        Ok(result) => {
            eprintln!("{}", result);
        }
        Err(e) => {
            eprintln!("Error during import: {}", e);
            std::process::exit(1);
        }
    }
}
