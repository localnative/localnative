use std::io;
use std::io::{Read, Write};
use std::mem::transmute;
use std::path::Path;
use std::str;
extern crate localnative_core;
extern crate rusqlite;
extern crate serde_json;
extern crate time;
use localnative_core::cmd::{create, insert, select};
use localnative_core::sql;
use localnative_core::Cmd;
use localnative_core::CmdInsert;
use localnative_core::CmdSearch;
use localnative_core::CmdSelect;
use localnative_core::Note;
use serde_json::Error;

use rusqlite::Connection;

fn main() {
    // Read the message length (first 4 bytes).
    let mut text_length_bytes = [0u8; 4];
    let stdin = io::stdin();
    let mut handle = stdin.lock();
    handle.read_exact(&mut text_length_bytes);

    //    if len(text_length_bytes) == 0:
    //        sys.exit(0)
    // Unpack message length as 4 byte integer.
    //    text_length = struct.unpack('i', text_length_bytes)[0]
    let text_length: u32 = unsafe { transmute(text_length_bytes) };
    let text_length: usize = text_length as usize;
    eprintln!("text_length {:?}", text_length);

    // Read the text (JSON object) of the message.
    //let mut text_buf = vec![0; text_length as usize];
    let mut text_buf = vec![0; text_length];
    handle.read_exact(&mut text_buf);
    let text = str::from_utf8(&text_buf).expect("not utf8 string");
    eprintln!("text_buf {:?}", text);

    if let Ok(cmd) = serde_json::from_str::<Cmd>(text) {
        process(cmd, text)
    } else {
        eprintln!("cmd json error");
    };
}

fn process(cmd: Cmd, text: &str) {
    eprintln!("process cmd {:?}", cmd);
    //let path = Path::new("/home/e/.ln/ln.sqlite3");
    let path = Path::new("localnative.sqlite3");
    let conn = Connection::open(path).unwrap();

    create(&conn);
    match cmd.action.as_ref() {
        "insert" => {
            if let Ok(i) = serde_json::from_str::<CmdInsert>(text) {
                let created_at =
                    time::strftime("%Y-%m-%d %H:%M:%S:%f UTC", &time::now_utc()).unwrap();
                //eprintln!("created_at {}", created_at);
                let note = Note {
                    title: i.title,
                    url: i.url,
                    tags: i.tags,
                    description: i.description,
                    comments: i.comments,
                    annotations: i.annotations,
                    created_at,
                };
                insert(&conn, note);
                do_select(&conn, &sql::select(i.limit, i.offset));
            } else {
                eprintln!("cmd insert json error");
            };
        }
        "select" => {
            if let Ok(s) = serde_json::from_str::<CmdSelect>(text) {
                do_select(&conn, &sql::select(s.limit, s.offset))
            } else {
                eprintln!("cmd select json error");
            };
        }
        "search" => {
            if let Ok(s) = serde_json::from_str::<CmdSearch>(text) {
                do_select(&conn, &sql::search(s.limit, s.offset, &s.query))
            } else {
                eprintln!("cmd search json error");
            }
        }
        _ => eprintln!("cmd no match"),
    }
}

fn do_select(conn: &Connection, sql: &str) {
    let j = select(&conn, sql);
    let msg = format!("{{\"notes\":{}}}", j);
    eprintln!("msg {}", msg);
    send_message(&msg);
}
// Helper function that sends a message to the webapp.

fn send_message(message: &str) {
    let buf = message.as_bytes();
    let size = buf.len() as u32;

    let bytes: [u8; 4] = if cfg!(target_endian = "little") {
        eprintln!("LE");
        unsafe { transmute(size.to_le()) }
    } else {
        eprintln!("BE");
        unsafe { transmute(size.to_be()) }
    };

    let mut handle = io::stdout();
    // Write message size.
    handle.write(&bytes);
    // Write the message itself.
    handle.write(buf);
    handle.flush();
}
