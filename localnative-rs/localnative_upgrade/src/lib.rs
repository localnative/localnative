extern crate rusqlite;
extern crate semver;
use rusqlite::Connection;
use semver::Version;
use std::io;
const VERSION: &'static str = "0.4.0";

pub fn upgrade(conn: &Connection) -> Result<&str, io::Error> {
    if Version::parse(check_version(conn)) < Version::parse("0.4.0") {
        upgrade_to_0_4_0(conn);
    }
    eprintln!("upgraded to {}", VERSION);
    Ok(VERSION)
}

fn check_version(conn: &Connection) -> &str {
    "0.3.10"
}

fn upgrade_to_0_4_0(conn: &Connection) -> io::Result<()> {
    Ok(())
}
