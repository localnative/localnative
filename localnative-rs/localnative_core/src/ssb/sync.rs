extern crate rusqlite;
use super::publish;
use rusqlite::Connection;
use Note;
use Ssb;
use SsbNote;

pub fn get_note_to_publish(conn: &Connection) -> Result<Note, rusqlite::Error> {
    let mut stmt =
        conn.prepare(
            "select rowid,
        title,
        url,
        tags,
        description,
        comments,
        annotations,
        created_at
        from note
        where rowid > (select note_rowid from ssb where is_active_author = 1)
        order by rowid
        limit 1",
        ).unwrap();
    stmt.query_row(&[], |row| Note {
        rowid: row.get(0),
        title: row.get(1),
        url: row.get(2),
        tags: row.get(3),
        description: row.get(4),
        comments: row.get(5),
        annotations: row.get(6),
        created_at: row.get(7),
    })
}

pub fn sync_to_ssb(conn: &Connection) {
    // loop till ssb note_rowid catch up to note
    loop {
        match get_note_to_publish(conn) {
            Ok(note) => {
                eprintln!("{:?}", note);
                // sync from db to ssb
                let ssb_note = publish(&note);

                // update ssb
                conn.execute_batch(&format!(
                    "BEGIN;
                UPDATE ssb SET seq = {},
                note_rowid = {}
                where is_active_author = 1;
                 COMMIT;
                ",
                    ssb_note.seq, note.rowid
                )).unwrap();
            }
            Err(e) => {
                match e {
                    rusqlite::Error::QueryReturnedNoRows => eprintln!("nothing to publish"),
                    _ => eprintln!("{:?}", e),
                }
                break;
            }
        }
    }
}

pub fn insert_ssb_note_to_db(conn: &Connection, rs: &SsbNote) {
    conn.execute_batch(&format!(
        "BEGIN;
       INSERT INTO note (title, url, tags, description, comments, annotations, created_at)
       VALUES ('{}', '{}', '{}', '', '', '', '');

       UPDATE ssb SET is_last_note = 0;

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
            (select author from ssb where is_active_author = 1),
            1,
            1,
            {seq},
            {ts},
            '{key}',
            '{prev}'
            );
       COMMIT;
       ",
        rs.note_title,
        rs.note_url,
        rs.note_tags,
        seq = rs.seq,
        ts = rs.ts,
        key = rs.key,
        prev = rs.prev
    )).unwrap();
}

pub fn get_ssb_active(conn: &Connection) -> Ssb {
    let mut stmt =
        conn.prepare(
            "select note_rowid,
        author,
        is_active_author,
        is_last_note,
        seq,
        ts,
        key,
        prev
        from ssb where is_active_author = 1",
        ).unwrap();
    let rs =
        stmt.query_row(&[], |row| Ssb {
            note_rowid: row.get(0),
            author: row.get(1),
            is_active_author: row.get(2),
            is_last_note: row.get(3),
            seq: row.get(4),
            ts: row.get(5),
            key: row.get(6),
            prev: row.get(7),
        }).unwrap();
    rs
}

pub fn get_ssb(conn: &Connection, author: &str) -> Ssb {
    let mut stmt =
        conn.prepare(&format!(
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
        )).unwrap();
    let rs =
        stmt.query_row(&[], |row| Ssb {
            note_rowid: row.get(0),
            author: row.get(1),
            is_active_author: row.get(2),
            is_last_note: row.get(3),
            seq: row.get(4),
            ts: row.get(5),
            key: row.get(6),
            prev: row.get(7),
        }).unwrap();
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
    ).unwrap();
}
