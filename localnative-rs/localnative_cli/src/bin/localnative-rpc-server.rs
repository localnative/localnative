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

//! Run a sync server other Local Native devices can pair and sync with.
//!
//! With `--pair` it also accepts one new device: the printed code is entered
//! on the other side. The process keeps running until interrupted.

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

    let matches = Command::new("localnative-rpc-server")
        .about("Run a Local Native LAN sync server (Ctrl-C to stop)")
        .arg(arg!(-a --addr[ADDR] "Bind address").default_value("0.0.0.0:2345"))
        .arg(arg!(-d --db[DB] "Database file (default: platform location)"))
        .arg(arg!(--pair "Accept one new device and print its pairing code"))
        .get_matches();

    if let Some(db) = matches.get_one::<String>("db") {
        localnative_core::db::set_db_path(db);
    }
    let addr = matches.get_one::<String>("addr").unwrap();

    let action = if matches.get_flag("pair") {
        "server-pairing"
    } else {
        "server"
    };
    let json = serde_json::json!({ "action": action, "addr": addr }).to_string();
    let response = run(&json);

    let ok = serde_json::from_str::<serde_json::Value>(&response)
        .map(|v| v.get("error").is_none())
        .unwrap_or(false);
    println!("{response}");
    if !ok {
        process::exit(1);
    }
    if action == "server-pairing"
        && let Some(code) = serde_json::from_str::<serde_json::Value>(&response)
            .ok()
            .and_then(|v| {
                v.get("pairing-code")
                    .and_then(|c| c.as_str())
                    .map(str::to_owned)
            })
    {
        println!("pairing code: {code}");
    }

    // The server runs on the shared runtime; park this thread until
    // interrupted.
    loop {
        std::thread::park();
    }
}
