extern crate localnative_core;
extern crate rusqlite;
extern crate time;
use localnative_core::Note;

use localnative_core::cmd::{clear, count, create, delete, insert, select};
use localnative_core::ssb::sync::{
    get_note_to_publish, get_ssb, get_ssb_active, init_active_author, insert_ssb_note_to_db,
    sync_to_ssb,
};
use localnative_core::ssb::{publish, tail, whoami};
use rusqlite::Connection;
use std::path::Path;

fn prepare_test_db() -> Connection {
    let path = Path::new("localnative-test.sqlite3");
    let conn = Connection::open(path).unwrap();
    create(&conn);
    conn
}

#[test]
fn test_reset_db() {
    let conn = prepare_test_db();
    clear(&conn);
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
fn test_insert() {
    let note = Note {
        rowid: -1,
        title: "title test insert".to_string(),
        url: "http://www.example.com".to_string(),
        tags: "tag1,tag2".to_string(),
        description: "desc".to_string(),
        comments: "comment".to_string(),
        annotations: "annotations".to_string(),
        created_at: "".to_string(),
    };
    let conn = prepare_test_db();
    insert(&conn, note);
}

#[test]
fn test_sync_to_ssb() {
    let conn = prepare_test_db();
    sync_to_ssb(&conn);
}

#[test]
fn test_get_note_to_publish() {
    let conn = prepare_test_db();
    let note = get_note_to_publish(&conn);
    eprintln!("{:?}", note);
}

#[test]
fn test_publish() {
    let note = Note {
        rowid: -1,
        title: "test_publish".to_string(),
        url: "http://www.example.com".to_string(),
        tags: "tag1,tag2".to_string(),
        description: "desc".to_string(),
        comments: "comment".to_string(),
        annotations: "annotations".to_string(),
        created_at: time::strftime("%Y-%m-%d %H:%M:%S:%f UTC", &time::now_utc()).unwrap(),
    };
    let ssb_note = publish(&note);
    eprintln!("{:?}", ssb_note);
}

#[test]
fn test_init_active_author() {
    let conn = prepare_test_db();
    let id = whoami();
    init_active_author(&conn, &id);
    let ssb = get_ssb(&conn, &id);
    let ssb_active = get_ssb_active(&conn);
    assert_eq!(ssb.author, ssb_active.author);
}

#[test]
fn test_tail() {
    let conn = prepare_test_db();
    let id = whoami();
    init_active_author(&conn, &id);
    loop {
        let seq = get_ssb_active(&conn).seq;
        if let Some(rs) = tail(&id, seq) {
            eprintln!("{:?}", rs);
            assert_eq!(rs.author, id);
            insert_ssb_note_to_db(&conn, &rs);
        } else {
            eprintln!("tail end");
            break;
        }
    }
}
