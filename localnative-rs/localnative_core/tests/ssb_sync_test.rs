extern crate localnative_core;
extern crate rusqlite;

use localnative_core::cmd::{create, delete, insert, select};
use rusqlite::Connection;
use std::path::Path;

fn prepare_test_db()->Connection{
    let path = Path::new("localnative-test.sqlite3");
    let conn = Connection::open(path).unwrap();
    create(&conn);
    conn
}

#[test]
fn it_works() {
    let conn = prepare_test_db();
    assert_eq!(4, 4);
}
