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
use localnative_core::cmd::{clear, count, create, delete, insert, select};
use localnative_core::exe::get_sqlite_connection;
use localnative_core::rusqlite;
use localnative_core::Note;
use localnative_ssb::sync::{
    get_note_to_publish, get_pubkeys, get_ssb, get_ssb_active, init_active_author,
    insert_ssb_note_to_db, sync_to_ssb,
};
use localnative_ssb::{publish, tail, whoami};
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
