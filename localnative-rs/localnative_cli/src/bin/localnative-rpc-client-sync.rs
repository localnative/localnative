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

//! Sync with a peer sync server. On first contact, pass the pairing code
//! shown on the other device with `--code`.

use clap::{Command, arg};
use localnative_core::run_sync as run;
use std::process;

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .init();

    let matches = Command::new("localnative-rpc-client-sync")
        .about("Sync notes with a Local Native peer over the LAN")
        .arg(
            arg!(-a --addr[ADDR] "Peer address, e.g. 192.168.1.5:2345")
                .default_value("127.0.0.1:2345"),
        )
        .arg(arg!(-d --db[DB] "Database file (default: platform location)"))
        .arg(arg!(-c --code[CODE] "Pairing code from the peer (first sync only)"))
        .get_matches();

    if let Some(db) = matches.get_one::<String>("db") {
        localnative_core::db::set_db_path(db);
    }
    let addr = matches.get_one::<String>("addr").unwrap();
    let mut command = serde_json::json!({ "action": "client-sync", "addr": addr });
    if let Some(code) = matches.get_one::<String>("code") {
        command["code"] = serde_json::json!(code);
    }

    let response = run(&command.to_string());
    let ok = serde_json::from_str::<serde_json::Value>(&response)
        .map(|v| v.get("error").is_none())
        .unwrap_or(false);
    println!("{response}");
    if !ok {
        process::exit(1);
    }
}
