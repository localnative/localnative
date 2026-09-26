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

use clap::{Command, arg};
use std::path::Path;
use std::process;

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .init();

    let matches = Command::new("localnative-export")
        .about("Export Local Native notes as Markdown files with YAML frontmatter")
        .arg(arg!(-o --output <DIR> "Output directory for exported .md files").required(true))
        .arg(arg!(-q --query <QUERY> "Optional search query to filter notes").required(false))
        .arg(arg!(-d --db <FILE> "Database file (default: platform location)").required(false))
        .get_matches();

    if let Some(db) = matches.get_one::<String>("db") {
        localnative_core::db::set_db_path(db);
    }
    let output_dir = matches.get_one::<String>("output").unwrap();
    let query = matches.get_one::<String>("query");

    let output_path = Path::new(output_dir);

    let conn = match localnative_core::db::init_db() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error opening database: {}", e);
            process::exit(1);
        }
    };

    match localnative_core::export::export_notes(&conn, output_path, query.map(|s| s.as_str())) {
        Ok(summary) => {
            eprintln!(
                "Exported {} note(s) to {} ({} skipped)",
                summary.written, output_dir, summary.skipped
            );
            if summary.skipped > 0 {
                process::exit(2);
            }
        }
        Err(e) => {
            eprintln!("Export failed: {}", e);
            process::exit(1);
        }
    }
}
