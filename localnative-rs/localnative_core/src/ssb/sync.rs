extern crate rusqlite;
use rusqlite::Connection;
use Ssb;
use SsbNote;

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
         author             ,
         is_active_author   ,
         is_last_note       ,
         seq                ,
         ts                 ,
         key                ,
         prev
         )
        values(
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
