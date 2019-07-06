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
use localnative_core::rusqlite;
use localnative_core::{Note, Ssb, SsbNote};
use publish;
use rusqlite::types::ToSql;
use rusqlite::{Connection, NO_PARAMS};
use tail;

pub fn get_pubkeys(conn: &Connection) -> String {
    let mut stmt = conn
        .prepare(
            "select note_rowid,
        author,
        is_active_author,
        is_last_note,
        seq,
        ts,
        key,
        prev
        from ssb",
        )
        .unwrap();
    let ssb_iter = stmt
        .query_map(NO_PARAMS, |row| {
            Ok(Ssb {
                note_rowid: row.get(0)?,
                author: row.get(1)?,
                is_active_author: false, //row.get(2),
                is_last_note: false,     // row.get(3),
                seq: row.get(4)?,
                ts: row.get(5)?,
                key: row.get(6)?,
                prev: row.get(7)?,
            })
        })
        .unwrap();

    let mut j = "[ ".to_owned();
    for ssb in ssb_iter {
        let ssb = ssb.unwrap();
        j.push_str(r#"""#);
        j.push_str(&ssb.author);
        j.push_str(r#"","#);
    }
    j.pop();
    j.push_str("]");
    j
}

pub fn get_note_to_publish(conn: &Connection) -> Result<Note, rusqlite::Error> {
    let mut stmt = conn
        .prepare(
            "select rowid,
        title,
        url,
        tags,
        description,
        comments,
        annotations,
        created_at,
        is_public
        from note
        where rowid > (select max(note_rowid) from ssb)
        order by rowid
        limit 1",
        )
        .unwrap();
    stmt.query_row(NO_PARAMS, |row| {
        Ok(Note {
            rowid: row.get(0)?,
            uuid4: row.get(1)?,
            title: row.get(2)?,
            url: row.get(3)?,
            tags: row.get(4)?,
            description: row.get(5)?,
            comments: row.get(6)?,
            annotations: row.get(7)?,
            created_at: row.get(8)?,
            is_public: row.get(9)?,
        })
    })
}

pub fn sync_to_ssb(conn: &Connection) {
    let pubkeys = get_pubkeys(conn);
    // loop till ssb note_rowid catch up to note
    loop {
        match get_note_to_publish(conn) {
            Ok(note) => {
                eprintln!("{:?}", note);
                // sync from db to ssb
                let rowid = note.rowid;
                let ssb_note = publish(note, &pubkeys);
                // update ssb
                conn.execute(
                    "
                UPDATE ssb SET
                note_rowid = ?1,
                ts = ?2,
                key = ?3,
                prev = ?4,
                seq = ?5
                WHERE is_active_author = 1
                ",
                    &[
                        &rowid as &dyn ToSql,
                        &ssb_note.ts,
                        &ssb_note.key,
                        &ssb_note.prev,
                        &ssb_note.seq,
                    ],
                )
                .unwrap();
            }
            Err(e) => {
                match e {
                    rusqlite::Error::QueryReturnedNoRows => {
                        refresh_is_last_note(&conn);
                        eprintln!("nothing to publish")
                    }
                    _ => eprintln!("{:?}", e),
                }
                break;
            }
        }
    }
}

pub fn refresh_is_last_note(conn: &Connection) {
    conn.execute_batch(
        "
    BEGIN;
    UPDATE ssb
    SET is_last_note = CASE WHEN
    (select max(rowid) from note) = ssb.note_rowid
    THEN 1
    ELSE 0
    END;
    COMMIT;
    ",
    )
    .unwrap();
}

pub fn get_authors() -> Vec<String> {
    let conn = super::get_sqlite_connection();
    let mut stmt = conn
        .prepare(
            "
        select note_rowid,
        author,
        is_active_author,
        is_last_note,
        seq,
        ts,
        key,
        prev
        from ssb",
        )
        .unwrap();
    let ssb_iter = stmt
        .query_map(NO_PARAMS, |row| {
            Ok(Ssb {
                note_rowid: row.get(0)?,
                author: row.get(1)?,
                is_active_author: false, //row.get(2),
                is_last_note: false,     // row.get(3),
                seq: row.get(4)?,
                ts: row.get(5)?,
                key: row.get(6)?,
                prev: row.get(7)?,
            })
        })
        .unwrap();
    let mut v = Vec::new();
    for ssb in ssb_iter {
        v.push(ssb.unwrap().author)
    }
    v
}
pub fn sync_all_to_db() {
    let authors = get_authors();
    eprintln!("{:?}", authors);
    for id in authors {
        sync_one_to_db(&id);
    }
}

pub fn sync_one_to_db(id: &str) {
    loop {
        let conn = super::get_sqlite_connection();
        let seq = get_ssb(&conn, id).seq;
        conn.close().unwrap();
        if let Some(rs) = tail(&id, seq) {
            eprintln!("{:?}", rs);
            assert_eq!(rs.author, id);
            insert_ssb_note_to_db(id, &rs);
        } else {
            eprintln!("tail end {}", id);
            break;
        }
    }
}

pub fn insert_ssb_note_to_db(id: &str, rs: &SsbNote) {
    let conn = &mut super::get_sqlite_connection();
    let tx = conn.transaction().unwrap();

    {
        tx.execute(
            "
       INSERT INTO note (title, url, tags, description, comments, annotations, created_at, is_public)
       VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
       ",
            &[
                &rs.note_title,
                &rs.note_url,
                &rs.note_tags,
                &rs.note_description,
                &rs.note_comments,
                &rs.note_annotations,
                &rs.note_created_at,
                &rs.is_public as &dyn ToSql,
            ],
        ).unwrap();
    }

    {
        tx.execute(
            "
    UPDATE ssb SET is_last_note = 0;
    ",
            NO_PARAMS,
        )
        .unwrap();
    }

    {
        tx.execute(
            "
       REPLACE INTO ssb (
         note_rowid         ,
         author             ,
         is_active_author   ,
         is_last_note       ,
         seq                ,
         ts                 ,
         key                ,
         prev
         )
        values(
            last_insert_rowid(),
            ?1,
            (select ?1 = (select author from ssb where is_active_author = 1)),  --is_active_author
            1, --is_last_note
            ?2,
            ?3,
            ?4,
            ?5
            );
       COMMIT;
       ",
            &[&id, &rs.seq as &dyn ToSql, &rs.ts, &rs.key, &rs.prev],
        )
        .unwrap();
    }

    tx.commit().unwrap();
}

pub fn get_ssb_active(conn: &Connection) -> Ssb {
    let mut stmt = conn
        .prepare(
            "select note_rowid,
        author,
        is_active_author,
        is_last_note,
        seq,
        ts,
        key,
        prev
        from ssb where is_active_author = 1",
        )
        .unwrap();
    let rs = stmt
        .query_row(NO_PARAMS, |row| {
            Ok(Ssb {
                note_rowid: row.get(0)?,
                author: row.get(1)?,
                is_active_author: row.get(2)?,
                is_last_note: row.get(3)?,
                seq: row.get(4)?,
                ts: row.get(5)?,
                key: row.get(6)?,
                prev: row.get(7)?,
            })
        })
        .unwrap();
    rs
}

pub fn get_ssb(conn: &Connection, author: &str) -> Ssb {
    let mut stmt = conn
        .prepare(&format!(
            "select note_rowid,
        author,
        is_active_author,
        is_last_note,
        seq,
        ts,
        key,
        prev
        from ssb where author = '{}'",
            author
        ))
        .unwrap();
    let rs = stmt
        .query_row(NO_PARAMS, |row| {
            Ok(Ssb {
                note_rowid: row.get(0)?,
                author: row.get(1)?,
                is_active_author: row.get(2)?,
                is_last_note: row.get(3)?,
                seq: row.get(4)?,
                ts: row.get(5)?,
                key: row.get(6)?,
                prev: row.get(7)?,
            })
        })
        .unwrap();
    rs
}

pub fn init_active_author(conn: &Connection, author: &str) {
    conn.execute(
        "replace into ssb (
         note_rowid         ,
         author             ,
         is_active_author   ,
         is_last_note       ,
         seq                ,
         ts                 ,
         key                ,
         prev
         )
        values(
            coalesce((select note_rowid from ssb where author = ?1), 0),
            ?1,
            1,
            coalesce((select is_last_note from ssb where author = ?1), 0),
            coalesce((select seq from ssb where author = ?1), 0),
            coalesce((select ts from ssb where author = ?1), 0),
            coalesce((select key from ssb where author = ?1), ''),
            coalesce((select prev from ssb where author = ?1), '')
            )",
        &[&author],
    )
    .unwrap();
}
