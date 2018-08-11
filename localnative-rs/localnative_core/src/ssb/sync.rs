extern crate rusqlite;
use super::publish;
use super::tail;
use rusqlite::Connection;
use Note;
use Ssb;
use SsbNote;

pub fn get_pubkeys(conn: &Connection) -> String {
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
        from ssb",
        ).unwrap();
    let ssb_iter =
        stmt.query_map(&[], |row| Ssb {
            note_rowid: row.get(0),
            author: row.get(1),
            is_active_author: false, //row.get(2),
            is_last_note: false,     // row.get(3),
            seq: row.get(4),
            ts: row.get(5),
            key: row.get(6),
            prev: row.get(7),
        }).unwrap();

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
        where rowid > (select max(note_rowid) from ssb)
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
                conn.execute_batch(&format!(
                    "BEGIN;
                UPDATE ssb SET seq = {},
                note_rowid = {}
                WHERE is_active_author = 1;
                 COMMIT;
                ",
                    ssb_note.seq, rowid
                )).unwrap();
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
        "BEGIN;
    UPDATE ssb
    SET is_last_note = CASE WHEN
    (select max(rowid) from note) = ssb.note_rowid
    THEN 1
    ELSE 0
    END;
     COMMIT;
    ",
    ).unwrap();
}

pub fn sync_all_to_db(conn: &Connection) {
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
        from ssb",
        ).unwrap();
    let ssb_iter =
        stmt.query_map(&[], |row| Ssb {
            note_rowid: row.get(0),
            author: row.get(1),
            is_active_author: false, //row.get(2),
            is_last_note: false,     // row.get(3),
            seq: row.get(4),
            ts: row.get(5),
            key: row.get(6),
            prev: row.get(7),
        }).unwrap();

    for ssb in ssb_iter {
        let ssb = ssb.unwrap();
        sync_one_to_db(conn, &ssb.author);
    }
}

pub fn sync_one_to_db(conn: &Connection, id: &str) {
    loop {
        let seq = get_ssb(&conn, id).seq;
        if let Some(rs) = tail(&id, seq) {
            eprintln!("{:?}", rs);
            assert_eq!(rs.author, id);
            insert_ssb_note_to_db(&conn, id, &rs);
        } else {
            eprintln!("tail end {}", id);
            break;
        }
    }
}

pub fn insert_ssb_note_to_db(conn: &Connection, id: &str, rs: &SsbNote) {
    conn.execute(
        "
       INSERT INTO note (title, url, tags, description, comments, annotations, created_at)
       VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
       ",
        &[
            &rs.note_title,
            &rs.note_url,
            &rs.note_tags,
            &rs.note_description,
            &rs.note_comments,
            &rs.note_annotations,
            &rs.note_created_at,
        ],
    ).unwrap();

    conn.execute(
        "
    UPDATE ssb SET is_last_note = 0;
    ",
        &[],
    ).unwrap();

    conn.execute(
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
        &[&id, &rs.seq, &rs.ts, &rs.key, &rs.prev],
    ).unwrap();
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
