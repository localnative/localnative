extern crate localnative_core;
extern crate rusqlite;

use localnative_core::cmd::{clear, count, create, delete, insert, select};
use localnative_core::ssb::sync::{get_ssb, init_active_author};
use localnative_core::ssb::{publish, tail, whoami};
use rusqlite::Connection;
use std::path::Path;

fn prepare_test_db() -> Connection {
    let path = Path::new("localnative-test.sqlite3");
    let conn = Connection::open(path).unwrap();
    clear(&conn);
    conn
}

#[test]
fn test_reset_db() {
    let conn = prepare_test_db();
    create(&conn);
    assert_eq!(0, count(&conn, "ssb"));
    assert_eq!(0, count(&conn, "note"));
}

#[test]
fn test_whoami() {
    let id = whoami();
    eprintln!("{}", id);
    assert_eq!(whoami(), id);
}

#[test]
fn test_init_active_author() {
    let conn = prepare_test_db();
    create(&conn);
    let id = whoami();
    init_active_author(&conn, &id);
    let ssb = get_ssb(&conn, &id);
    tail(&id, 0);
}

#[test]
fn test_tail() {
    let id = whoami();
    let rs = tail(&id, 0).unwrap();
    eprintln!("{:?}", rs);
    assert_eq!(rs.author, id);
}
