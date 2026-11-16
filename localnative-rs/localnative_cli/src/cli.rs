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

use clap::{ArgAction, Command, arg, value_parser};
use std::ffi::{OsStr, OsString};

use crate::{commands, host};

pub fn run_from_env() -> i32 {
    init_logging();
    let mut process_args = std::env::args_os();
    let argv0 = process_args
        .next()
        .unwrap_or_else(|| OsString::from("localnative"));
    let name = executable_name(&argv0);

    if name.eq_ignore_ascii_case("localnative-web-ext-host") {
        return match host::run() {
            Ok(()) => 0,
            Err(error) => {
                eprintln!("Browser host failed: {error}");
                1
            }
        };
    }

    let mut args = vec![argv0];
    if let Some(subcommands) = legacy_subcommands(&name) {
        args.extend(subcommands.iter().map(OsString::from));
    }
    args.extend(process_args);

    let matches = match command().try_get_matches_from(args) {
        Ok(matches) => matches,
        Err(error) => {
            let code = error.exit_code();
            let _ = error.print();
            return code;
        }
    };
    if let Some(db) = matches.get_one::<String>("db") {
        localnative_core::db::set_db_path(db);
    }
    commands::dispatch(&matches)
}

fn init_logging() {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .with_writer(std::io::stderr)
        .try_init();
}

fn executable_name(argv0: &OsStr) -> String {
    let path = argv0.to_string_lossy();
    let base = path.rsplit(['/', '\\']).next().unwrap_or(&path);
    base.rsplit_once('.')
        .map_or(base, |(stem, extension)| {
            if extension.eq_ignore_ascii_case("exe") {
                stem
            } else {
                base
            }
        })
        .to_string()
}

fn legacy_subcommands(name: &str) -> Option<&'static [&'static str]> {
    match name {
        "localnative-mcp" => Some(&["mcp"]),
        "localnative-rpc-server" => Some(&["sync", "serve"]),
        "localnative-rpc-client-sync" => Some(&["sync", "connect"]),
        "localnative-rpc-client-stop-server" => Some(&["sync", "stop"]),
        "localnative-import" => Some(&["import"]),
        "localnative-export" => Some(&["export"]),
        "localnative-export-db" => Some(&["export-db"]),
        "localnative-import-db" => Some(&["import-db"]),
        "localnative-upgrade" => Some(&["upgrade"]),
        _ => None,
    }
}

fn command() -> Command {
    Command::new("localnative")
        .version(env!("CARGO_PKG_VERSION"))
        .about("Search, archive, sync, and export your local LocalNative library")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .arg(
            arg!(-d --db <FILE> "Database file (default: platform location)")
                .global(true),
        )
        .subcommand(
            Command::new("search")
                .about("Search notes and archived article text; prints JSON")
                .arg(arg!([QUERY] "Search terms; omit to list recent notes"))
                .arg(
                    arg!(-l --limit <COUNT> "Maximum notes to return")
                        .value_parser(value_parser!(u32))
                        .default_value("20"),
                )
                .arg(
                    arg!(-o --offset <COUNT> "Results to skip")
                        .value_parser(value_parser!(u32))
                        .default_value("0"),
                ),
        )
        .subcommand(
            Command::new("get")
                .about("Get one complete note by UUID, including archived text; prints JSON")
                .arg(arg!(<UUID4> "Note UUID returned by search")),
        )
        .subcommand(Command::new("mcp").about("Run the read-only MCP stdio server"))
        .subcommand(
            Command::new("host")
                .about("Run the browser native-messaging host, or install its manifests")
                .subcommand(
                    Command::new("install")
                        .about("Install the host alias and Chrome/Firefox manifests")
                        .arg(
                            arg!(--"allowed-origin" <ORIGIN> "Allowed Chrome extension origin; repeatable")
                                .action(ArgAction::Append),
                        ),
                ),
        )
        .subcommand(
            Command::new("sync")
                .about("Synchronize notes with another LocalNative device")
                .subcommand_required(true)
                .subcommand(
                    Command::new("serve")
                        .about("Run a LAN sync server (Ctrl-C to stop)")
                        .arg(
                            arg!(-a --addr <ADDR> "Bind address")
                                .default_value("0.0.0.0:2345"),
                        )
                        .arg(arg!(--pair "Accept one new device and print its pairing code")),
                )
                .subcommand(
                    Command::new("connect")
                        .about("Sync with a peer over the LAN")
                        .arg(
                            arg!(-a --addr <ADDR> "Peer address, e.g. 192.168.1.5:2345")
                                .default_value("127.0.0.1:2345"),
                        )
                        .arg(arg!(-c --code <CODE> "Pairing code from the peer (first sync only)")),
                )
                .subcommand(
                    Command::new("stop")
                        .about("Stop a sync server started by this process")
                        .arg(
                            arg!(-a --addr <ADDR> "Address the server was started on")
                                .default_value("0.0.0.0:2345"),
                        ),
                ),
        )
        .subcommand(
            Command::new("import")
                .about("Import notes from external services")
                .arg(
                    arg!(-f --format <FORMAT> "Import format: pocket, omnivore, raindrop, or plinky")
                        .required(true),
                )
                .arg(arg!(<FILE> "Path to the export file").required(true)),
        )
        .subcommand(
            Command::new("export")
                .about("Export notes as Markdown files with YAML frontmatter")
                .arg(arg!(-o --output <DIR> "Output directory for exported .md files").required(true))
                .arg(arg!(-q --query <QUERY> "Optional search query to filter notes")),
        )
        .subcommand(
            Command::new("export-db")
                .about("Export a standalone database copy without WAL sidecars or device identity")
                .arg(arg!(-o --output <FILE> "Destination .sqlite3 file path").required(true)),
        )
        .subcommand(
            Command::new("import-db")
                .about("Merge another database into the local DB (one-way, last-write-wins)")
                .arg(arg!(-i --input <FILE> "Source .sqlite3 file to import from").required(true)),
        )
        .subcommand(Command::new("upgrade").about("Run pending database schema migrations"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn command_definition_is_valid() {
        command().debug_assert();
    }

    #[test]
    fn every_legacy_binary_maps_to_a_unified_subcommand() {
        let aliases = [
            ("localnative-mcp", &["mcp"][..]),
            ("localnative-rpc-server", &["sync", "serve"][..]),
            ("localnative-rpc-client-sync", &["sync", "connect"][..]),
            ("localnative-rpc-client-stop-server", &["sync", "stop"][..]),
            ("localnative-import", &["import"][..]),
            ("localnative-export", &["export"][..]),
            ("localnative-export-db", &["export-db"][..]),
            ("localnative-import-db", &["import-db"][..]),
            ("localnative-upgrade", &["upgrade"][..]),
        ];
        for (alias, expected) in aliases {
            assert_eq!(legacy_subcommands(alias), Some(expected));
        }
        assert_eq!(legacy_subcommands("localnative"), None);
    }

    #[test]
    fn windows_executable_suffix_does_not_hide_an_alias() {
        assert_eq!(
            executable_name(OsStr::new(r"C:\tools\localnative-rpc-client-sync.exe")),
            "localnative-rpc-client-sync"
        );
    }
}
