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

//! Stop the sync server run by *this* process's `localnative-rpc-server`.
//! Stopping a server is a local action; there is deliberately no remote stop.

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

    let matches = Command::new("localnative-rpc-server-stop")
        .about("Stop a sync server started by this process")
        .arg(
            arg!(-a --addr[ADDR] "Address the server was started on").default_value("0.0.0.0:2345"),
        )
        .get_matches();

    let addr = matches.get_one::<String>("addr").unwrap();
    let json = serde_json::json!({ "action": "server-stop", "addr": addr }).to_string();
    let response = run(&json);
    let ok = serde_json::from_str::<serde_json::Value>(&response)
        .map(|v| v.get("error").is_none())
        .unwrap_or(false);
    println!("{response}");
    if !ok {
        process::exit(1);
    }
}
