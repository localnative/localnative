/*
    Local Native
    Copyright (C) 2018-2019  Yi Wang

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
extern crate localnative_core;
extern crate localnative_ssb;
extern crate time;
extern crate uuid;
use localnative_core::cmd::{create, insert};
use localnative_core::exe::get_sqlite_connection;
use localnative_core::rusqlite;
use localnative_core::Note;
use localnative_ssb::sync::{
    get_note_to_publish, get_pubkeys, get_ssb, get_ssb_active, init_active_author,
    insert_ssb_note_to_db, sync_all_to_db, sync_one_to_db, sync_to_ssb,
};
use localnative_ssb::{publish, tail, whoami};
use rusqlite::Connection;
use uuid::Uuid;

fn prepare_test_db() -> Connection {
    let conn = get_sqlite_connection();
    create(&conn);
    conn
}

// #[test]
// fn test_reset_db() {
//     let conn = prepare_test_db();
//     clear(&conn);
//     create(&conn);
//     assert_eq!(0, count(&conn, "ssb"));
//     assert_eq!(0, count(&conn, "note"));
// }

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
        uuid4: Uuid::new_v4().to_string(),
        title: "title test insert".to_string(),
        url: "http://www.example.com".to_string(),
        tags: "tag1,tag2".to_string(),
        description: "desc".to_string(),
        comments: "comment".to_string(),
        annotations: "annotations".to_string(),
        created_at: "".to_string(),
        is_public: false,
    };
    insert(note);
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
        uuid4: Uuid::new_v4().to_string(),
        title: "test_publish".to_string(),
        url: "http://www.example.com".to_string(),
        tags: "tag1,tag2".to_string(),
        description: "desc".to_string(),
        comments: "comment".to_string(),
        annotations: "annotations".to_string(),
        created_at: time::strftime("%Y-%m-%d %H:%M:%S:%f UTC", &time::now_utc()).unwrap(),
        is_public: false,
    };
    let conn = prepare_test_db();
    let pubkeys = get_pubkeys(&conn);
    let ssb_note = publish(note, &pubkeys);
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
        let seq = get_ssb(&conn, &id).seq;
        if let Some(rs) = tail(&id, seq) {
            eprintln!("{:?}", rs);
            assert_eq!(rs.author, id);
            insert_ssb_note_to_db(&id, &rs);
        } else {
            eprintln!("tail end");
            break;
        }
    }
}

#[test]
fn test_sync_one_to_db() {
    let conn = prepare_test_db();
    let id = whoami();
    init_active_author(&conn, &id);
    sync_one_to_db(&id);
}

#[test]
fn test_sync_all_to_db() {
    sync_all_to_db();
}
