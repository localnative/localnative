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
use localnative_core::run_sync as run;
fn main() {
    let matches = Command::new("localnative-rpc-client")
        .arg(arg!(-a - -addr[ADDR]))
        .get_matches();

    let addr = matches.get_one::<&str>("addr").unwrap_or(&"127.0.0.1:2345");
    eprintln!("addr: {}", addr);
    run(&(r#"{"action":"client-stop-server", "addr": ""#.to_string() + addr + r#""}"#));
}
