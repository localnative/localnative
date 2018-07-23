extern crate rusqlite;
use self::rusqlite::Connection;
use Ssb;

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
        "insert into ssb (
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
            0,
            0,
            '',
            '',
            ''
            )",
        &[&author],
    ).unwrap();
}
