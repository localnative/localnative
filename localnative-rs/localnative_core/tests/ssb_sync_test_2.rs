extern crate localnative_core;
extern crate rusqlite;
extern crate time;
use localnative_core::Note;

use localnative_core::cmd::{clear, count, create, delete, insert, select};
use localnative_core::ssb::sync::{
    get_note_to_publish, get_pubkeys, get_ssb, get_ssb_active, init_active_author,
    insert_ssb_note_to_db, sync_to_ssb,
};
use localnative_core::ssb::{get_sqlite_connection, publish, tail, whoami};
use rusqlite::Connection;

fn prepare_test_db() -> Connection {
    let conn = get_sqlite_connection();
    create(&conn);
    conn
}

#[test]
fn test_get_pubkeys() {
    let conn = prepare_test_db();
    let pubkeys = get_pubkeys(&conn);
    eprintln!("{:?}", pubkeys);
}
