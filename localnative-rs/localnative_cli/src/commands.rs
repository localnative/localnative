/*
    Local Native
    Copyright (C) 2018-2026  Yi Wang

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

use clap::ArgMatches;
use localnative_core::{db, export, import, run_sync};
use serde_json::{Value, json};
use std::path::Path;

use crate::{host, mcp};

pub fn dispatch(matches: &ArgMatches) -> i32 {
    match matches.subcommand() {
        Some(("search", matches)) => run_core(json!({
            "action": "search",
            "query": matches.get_one::<String>("QUERY").map(String::as_str).unwrap_or(""),
            "limit": matches.get_one::<u32>("limit").copied().unwrap_or(20),
            "offset": matches.get_one::<u32>("offset").copied().unwrap_or(0)
        }))
        .map(|_| 0)
        .unwrap_or(1),
        Some(("get", matches)) => run_core(json!({
            "action": "get",
            "uuid4": matches.get_one::<String>("UUID4").unwrap()
        }))
        .map(|_| 0)
        .unwrap_or(1),
        Some(("mcp", _)) => match mcp::run() {
            Ok(()) => 0,
            Err(error) => {
                eprintln!("MCP server failed: {error}");
                1
            }
        },
        Some(("host", matches)) => match matches.subcommand() {
            Some(("install", matches)) => install_host(matches),
            _ => match host::run() {
                Ok(()) => 0,
                Err(error) => {
                    eprintln!("Browser host failed: {error}");
                    1
                }
            },
        },
        Some(("sync", matches)) => match matches.subcommand() {
            Some(("serve", matches)) => serve(matches),
            Some(("connect", matches)) => {
                let mut command = json!({
                    "action": "client-sync",
                    "addr": matches.get_one::<String>("addr").unwrap()
                });
                if let Some(code) = matches.get_one::<String>("code") {
                    command["code"] = json!(code);
                }
                run_core(command).map(|_| 0).unwrap_or(1)
            }
            Some(("stop", matches)) => run_core(json!({
                "action": "server-stop",
                "addr": matches.get_one::<String>("addr").unwrap()
            }))
            .map(|_| 0)
            .unwrap_or(1),
            _ => unreachable!(),
        },
        Some(("import", matches)) => import_notes(matches),
        Some(("export", matches)) => export_notes(matches),
        Some(("export-db", matches)) => export_db(matches),
        Some(("import-db", matches)) => import_db(matches),
        Some(("upgrade", _)) => run_core(json!({ "action": "upgrade" }))
            .map(|_| 0)
            .unwrap_or(1),
        _ => unreachable!(),
    }
}

fn run_core(command: Value) -> Result<Value, i32> {
    let response = run_sync(&command.to_string());
    println!("{response}");
    let value: Value = serde_json::from_str(&response).map_err(|error| {
        eprintln!("LocalNative returned invalid JSON: {error}");
        1
    })?;
    if value.get("error").is_some() {
        Err(1)
    } else {
        Ok(value)
    }
}

fn serve(matches: &ArgMatches) -> i32 {
    let action = if matches.get_flag("pair") {
        "server-pairing"
    } else {
        "server"
    };
    let response = match run_core(json!({
        "action": action,
        "addr": matches.get_one::<String>("addr").unwrap()
    })) {
        Ok(response) => response,
        Err(code) => return code,
    };
    if let Some(code) = response.get("pairing-code").and_then(Value::as_str) {
        println!("pairing code: {code}");
    }
    loop {
        std::thread::park();
    }
}

fn install_host(matches: &ArgMatches) -> i32 {
    let allowed_origins = matches
        .get_many::<String>("allowed-origin")
        .map(|origins| origins.cloned().collect());
    match localnative_hostinstall::WebKind::install_cli(allowed_origins) {
        Ok(path) => {
            println!(
                "Installed browser native-messaging host at {}",
                path.display()
            );
            0
        }
        Err(error) => {
            eprintln!("Host installation failed: {error}");
            1
        }
    }
}

fn import_notes(matches: &ArgMatches) -> i32 {
    let format = matches.get_one::<String>("format").unwrap();
    let file_path = matches.get_one::<String>("FILE").unwrap();
    let content = match std::fs::read_to_string(file_path) {
        Ok(content) => content,
        Err(error) => {
            eprintln!("Error reading file '{file_path}': {error}");
            return 1;
        }
    };
    let notes = match format.as_str() {
        "pocket" => import::parse_pocket_html(&content),
        "omnivore" => match import::parse_omnivore_json(&content) {
            Ok(notes) => notes,
            Err(error) => {
                eprintln!("Error parsing Omnivore JSON: {error}");
                return 1;
            }
        },
        "raindrop" => import::parse_raindrop_csv(&content),
        "plinky" => match import::parse_plinky_json(&content) {
            Ok(notes) => notes,
            Err(error) => {
                eprintln!("Error parsing Plinky JSON: {error}");
                return 1;
            }
        },
        _ => {
            eprintln!(
                "Unknown format '{format}'. Supported formats: pocket, omnivore, raindrop, plinky"
            );
            return 1;
        }
    };
    eprintln!("Parsed {} notes from {format} file", notes.len());
    let conn = match db::init_db() {
        Ok(conn) => conn,
        Err(error) => {
            eprintln!("Error opening database: {error}");
            return 1;
        }
    };
    match import::import_notes(&conn, notes) {
        Ok(result) => {
            eprintln!("{result}");
            0
        }
        Err(error) => {
            eprintln!("Error during import: {error}");
            1
        }
    }
}

fn export_notes(matches: &ArgMatches) -> i32 {
    let output_dir = matches.get_one::<String>("output").unwrap();
    let query = matches.get_one::<String>("query");
    let conn = match db::init_db() {
        Ok(conn) => conn,
        Err(error) => {
            eprintln!("Error opening database: {error}");
            return 1;
        }
    };
    match export::export_notes(&conn, Path::new(output_dir), query.map(String::as_str)) {
        Ok(summary) => {
            eprintln!(
                "Exported {} note(s) to {} ({} skipped)",
                summary.written, output_dir, summary.skipped
            );
            if summary.skipped > 0 { 2 } else { 0 }
        }
        Err(error) => {
            eprintln!("Export failed: {error}");
            1
        }
    }
}

fn export_db(matches: &ArgMatches) -> i32 {
    let destination = matches.get_one::<String>("output").unwrap();
    let conn = match db::init_db() {
        Ok(conn) => conn,
        Err(error) => {
            eprintln!("Error opening database: {error}");
            return 1;
        }
    };
    match db::queries::export_db(&conn, destination) {
        Ok(()) => {
            eprintln!("Exported database to {destination}");
            0
        }
        Err(error) => {
            eprintln!("Export failed: {error}");
            1
        }
    }
}

fn import_db(matches: &ArgMatches) -> i32 {
    let source = matches.get_one::<String>("input").unwrap();
    let conn = match db::init_db() {
        Ok(conn) => conn,
        Err(error) => {
            eprintln!("Error opening database: {error}");
            return 1;
        }
    };
    match db::queries::import_db(&conn, source) {
        Ok(merged) => {
            eprintln!("Imported from {source} ({merged} note(s) merged)");
            0
        }
        Err(error) => {
            eprintln!("Import failed: {error}");
            1
        }
    }
}
