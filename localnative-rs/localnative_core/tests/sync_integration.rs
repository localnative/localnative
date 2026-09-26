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

//! End-to-end sync tests: two real databases, a real server on a real port,
//! the real client. These are the tests that would have caught every sync
//! defect in the 2026-09 review: the 20-note rate-limit wall, unauthenticated
//! reads and wipes, tombstone resurrection under clock skew, and forged
//! last-write-wins tokens.

use localnative_core::db::{self, peers, queries, sync as dbsync};
use localnative_core::rpc;
use std::net::{SocketAddr, TcpListener};
use std::path::PathBuf;
use std::time::Duration;

fn scratch(tag: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "ln_it_{}_{tag}_{}.sqlite3",
        std::process::id(),
        line!()
    ))
}

fn new_pool(tag: &str) -> db::Pool {
    let path = scratch(tag);
    for suffix in ["", "-wal", "-shm"] {
        let _ = std::fs::remove_file(format!("{}{suffix}", path.display()));
    }
    db::init_pool_at(&path).expect("scratch pool")
}

fn free_port() -> u16 {
    TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port()
}

fn server_addr() -> SocketAddr {
    format!("127.0.0.1:{}", free_port()).parse().unwrap()
}

fn count(pool: &db::Pool) -> u32 {
    let conn = pool.get().unwrap();
    queries::do_select(&conn, 1000, 0).unwrap().count
}

async fn pair(server: SocketAddr, server_pool: &db::Pool, client_pool: &db::Pool) {
    let code = rpc::start_pairing(&server.to_string(), server_pool)
        .await
        .expect("start pairing server");
    tokio::time::sleep(Duration::from_millis(150)).await;
    rpc::sync(&server.to_string(), Some(&code), client_pool)
        .await
        .expect("first sync pairs");
}

/// Two devices, one bulk transfer: hundreds of notes move in one session,
/// batched by byte budget — the case the old per-note rate limiter aborted
/// after 20 notes.
#[tokio::test(flavor = "multi_thread")]
async fn bulk_sync_moves_every_note() {
    let server_pool = new_pool("bulk_server");
    let client_pool = new_pool("bulk_client");

    let attachment = vec![0x5au8; 4096];
    {
        let conn = client_pool.get().unwrap();
        for i in 0..200 {
            queries::insert_note(
                &conn,
                &format!("note {i} 如何学习"),
                &format!("https://example.com/{i}"),
                "bulk,测试",
                "",
                "",
                &attachment,
                false,
            )
            .unwrap();
        }
    }

    let addr = server_addr();
    pair(addr, &server_pool, &client_pool).await;

    assert_eq!(count(&server_pool), 200, "all notes arrived");
    let conn = server_pool.get().unwrap();
    let found = queries::do_search(&conn, "如何学习", 1000, 0).unwrap();
    assert_eq!(found.count, 200, "transferred CJK text is searchable");
    drop(conn);

    // A second sync with no code — the stored key is enough.
    rpc::sync(&addr.to_string(), None, &client_pool)
        .await
        .expect("paired sync without code");
}

/// A client that is not paired gets nothing: handshake completed, key not in
/// the server's peer table, verdict refused — no note data crosses.
#[tokio::test(flavor = "multi_thread")]
async fn unpaired_client_is_refused() {
    let server_pool = new_pool("refuse_server");
    let client_pool = new_pool("refuse_client");
    {
        let conn = server_pool.get().unwrap();
        for i in 0..10 {
            queries::insert_note(&conn, &format!("secret {i}"), "", "", "", "", b"", false)
                .unwrap();
        }
    }

    let addr = server_addr();
    // Pair, then the server forgets the client (simulating an unpair).
    pair(addr, &server_pool, &client_pool).await;
    {
        let conn = server_pool.get().unwrap();
        let all = peers::list(&conn).unwrap();
        for peer in all {
            peers::forget(&conn, &peer.public_key).unwrap();
        }
    }

    let err = rpc::sync(&addr.to_string(), None, &client_pool)
        .await
        .expect_err("unpaired client must be refused");
    assert!(
        matches!(
            err,
            localnative_core::SyncError::NotPaired
                | localnative_core::SyncError::Rejected(_)
                | localnative_core::SyncError::Handshake(_)
        ),
        "got: {err:?}"
    );
    assert_eq!(
        count(&client_pool),
        10,
        "client keeps what it pulled while paired"
    );
    // And nothing new could have leaked: the server still holds everything.
    assert_eq!(count(&server_pool), 10);
}

/// A note with a forged last-write-wins token ("~", or the far future) is
/// refused by the receiving side. Before validation, such a token outranked
/// every future edit and could wipe a library.
#[tokio::test(flavor = "multi_thread")]
async fn forged_tokens_are_refused_end_to_end() {
    let server_pool = new_pool("forge_server");
    let client_pool = new_pool("forge_client");
    let addr = server_addr();
    pair(addr, &server_pool, &client_pool).await;

    let uuid4 = {
        let conn = client_pool.get().unwrap();
        let note = queries::insert_note(&conn, "honest note", "", "", "", "", b"", false).unwrap();
        conn.execute(
            "UPDATE note SET updated_at = '~', title = 'forged' WHERE uuid4 = ?1",
            rusqlite::params![note.uuid4],
        )
        .unwrap();
        note.uuid4
    };

    rpc::sync(&addr.to_string(), None, &client_pool)
        .await
        .expect("sync carries the tampered row");

    let conn = server_pool.get().unwrap();
    let stored = dbsync::get_wire_note(&conn, &uuid4).unwrap();
    assert!(
        stored.is_none() || stored.unwrap().title == "honest note",
        "the forged version must not have been applied"
    );
}

/// Deleting on one device hides the note everywhere, and the deletion is
/// final: the tombstone survives further syncs from both directions.
#[tokio::test(flavor = "multi_thread")]
async fn deletes_propagate_and_stay_deleted() {
    let server_pool = new_pool("del_server");
    let client_pool = new_pool("del_client");
    let addr = server_addr();

    let uuid4 = {
        let conn = server_pool.get().unwrap();
        let note = queries::insert_note(&conn, "to delete", "", "", "private body", "", b"", false)
            .unwrap();
        note.uuid4
    };
    pair(addr, &server_pool, &client_pool).await;
    assert_eq!(count(&client_pool), 1);

    {
        let conn = server_pool.get().unwrap();
        let rowid: i64 = conn
            .query_row("SELECT rowid FROM note WHERE uuid4 = ?1", [&uuid4], |r| {
                r.get(0)
            })
            .unwrap();
        queries::delete_note(&conn, rowid).unwrap();
    }

    rpc::sync(&addr.to_string(), None, &client_pool)
        .await
        .expect("sync after delete");
    assert_eq!(count(&client_pool), 0, "delete propagated");

    // Sync back the other way; the note stays deleted and its content never
    // travels.
    {
        let conn = server_pool.get().unwrap();
        let tomb = dbsync::get_wire_note(&conn, &uuid4).unwrap().unwrap();
        assert!(tomb.deleted && tomb.description.is_empty());
    }
    rpc::start(&addr.to_string(), &server_pool).await.unwrap();
    tokio::time::sleep(Duration::from_millis(100)).await;
    rpc::sync(&addr.to_string(), None, &client_pool)
        .await
        .expect("client-initiated sync after delete");
    assert_eq!(count(&client_pool), 0, "still deleted");
}

/// A peer whose clock runs fast must not be able to resurrect a deletion:
/// the local clock advances past every observed token.
#[tokio::test(flavor = "multi_thread")]
async fn skewed_clock_cannot_resurrect_a_delete() {
    let server_pool = new_pool("skew_server");
    let client_pool = new_pool("skew_client");
    let addr = server_addr();

    let uuid4 = {
        let conn = server_pool.get().unwrap();
        let note = queries::insert_note(&conn, "skewed", "", "", "", "", b"", false).unwrap();
        // The server's clock is 10 minutes fast.
        let fast = chrono::Utc::now().timestamp_millis() + 10 * 60 * 1000;
        conn.execute(
            "UPDATE note SET updated_at = ?1 WHERE uuid4 = ?2",
            rusqlite::params![format!("{fast:020}-node-server"), note.uuid4],
        )
        .unwrap();
        note.uuid4
    };
    pair(addr, &server_pool, &client_pool).await;

    // The client deletes the note (its clock is correct).
    {
        let conn = client_pool.get().unwrap();
        let rowid: i64 = conn
            .query_row("SELECT rowid FROM note WHERE uuid4 = ?1", [&uuid4], |r| {
                r.get(0)
            })
            .unwrap();
        queries::delete_note(&conn, rowid).unwrap();
    }
    rpc::sync(&addr.to_string(), None, &client_pool)
        .await
        .expect("push the delete");
    assert_eq!(count(&server_pool), 0, "fast clock accepted the delete");

    // And a subsequent pull cannot bring it back.
    rpc::sync(&addr.to_string(), None, &client_pool)
        .await
        .expect("pull after delete");
    assert_eq!(count(&client_pool), 0);
}

/// Edits and new notes flow both directions in one session.
#[tokio::test(flavor = "multi_thread")]
async fn edits_flow_both_directions() {
    let server_pool = new_pool("edit_server");
    let client_pool = new_pool("edit_client");
    let addr = server_addr();

    let server_uuid = {
        let conn = server_pool.get().unwrap();
        queries::insert_note(
            &conn,
            "from server",
            "https://s.example",
            "",
            "",
            "",
            b"",
            false,
        )
        .unwrap()
        .uuid4
    };
    pair(addr, &server_pool, &client_pool).await;
    assert_eq!(count(&client_pool), 1);

    let client_uuid = {
        let conn = client_pool.get().unwrap();
        queries::insert_note(
            &conn,
            "from client",
            "https://c.example",
            "",
            "",
            "",
            b"",
            false,
        )
        .unwrap()
        .uuid4
    };
    // The client also edits the server's note.
    {
        let conn = client_pool.get().unwrap();
        let newer = format!(
            "{:020}-node-client",
            chrono::Utc::now().timestamp_millis() + 1
        );
        conn.execute(
            "UPDATE note SET title = 'edited by client', updated_at = ?1 WHERE uuid4 = ?2",
            rusqlite::params![newer, server_uuid],
        )
        .unwrap();
    }

    rpc::sync(&addr.to_string(), None, &client_pool)
        .await
        .expect("bidirectional sync");

    let server_conn = server_pool.get().unwrap();
    assert_eq!(
        dbsync::get_wire_note(&server_conn, &client_uuid)
            .unwrap()
            .expect("client note on server")
            .title,
        "from client"
    );
    assert_eq!(
        dbsync::get_wire_note(&server_conn, &server_uuid)
            .unwrap()
            .expect("edited note on server")
            .title,
        "edited by client"
    );
}

/// Stopping a server is a local action; a stopped server refuses connections.
#[tokio::test(flavor = "multi_thread")]
async fn server_stop_is_local() {
    let server_pool = new_pool("stop_server");
    let client_pool = new_pool("stop_client");
    let addr = server_addr();
    pair(addr, &server_pool, &client_pool).await;

    rpc::stop_local(&addr.to_string()).await.expect("stop");
    tokio::time::sleep(Duration::from_millis(150)).await;

    let err = rpc::sync(&addr.to_string(), None, &client_pool).await;
    assert!(err.is_err(), "stopped server must refuse new sessions");
}

/// A wrong pairing code never completes: the handshake itself fails, and the
/// server does not reveal whether the code was close.
#[tokio::test(flavor = "multi_thread")]
async fn wrong_pairing_code_fails_the_handshake() {
    let server_pool = new_pool("badcode_server");
    let client_pool = new_pool("badcode_client");
    let addr = server_addr();

    let code = rpc::start_pairing(&addr.to_string(), &server_pool)
        .await
        .unwrap();
    tokio::time::sleep(Duration::from_millis(150)).await;

    // Same length, wrong content: flip the first character to another from
    // the pairing alphabet. (The real code pairs; this one must not.)
    let mut chars: Vec<char> = code.chars().collect();
    chars[0] = if chars[0] == 'A' { 'B' } else { 'A' };
    let wrong: String = chars.into_iter().collect();
    assert_ne!(wrong, code);

    let err = tokio::time::timeout(
        Duration::from_secs(10),
        rpc::sync(&addr.to_string(), Some(&wrong), &client_pool),
    )
    .await
    .expect("handshake fails fast, not by hanging")
    .expect_err("wrong code must not pair");
    // The server tears the connection down during or right after the
    // handshake; which surface the client sees depends on timing.
    assert!(
        matches!(
            err,
            localnative_core::SyncError::Handshake(_)
                | localnative_core::SyncError::Rejected(_)
                | localnative_core::SyncError::IoError(_)
                | localnative_core::SyncError::Timeout(_)
        ),
        "got: {err:?}"
    );
    assert_eq!(count(&client_pool), 0);
}

/// Re-pairing after unpairing works with a fresh code.
#[tokio::test(flavor = "multi_thread")]
async fn repairing_after_forgetting() {
    let server_pool = new_pool("re_server");
    let client_pool = new_pool("re_client");
    let addr = server_addr();
    pair(addr, &server_pool, &client_pool).await;

    // Server forgets the client.
    {
        let conn = server_pool.get().unwrap();
        for peer in peers::list(&conn).unwrap() {
            peers::forget(&conn, &peer.public_key).unwrap();
        }
    }
    rpc::sync(&addr.to_string(), None, &client_pool)
        .await
        .expect_err("old pairing is gone");

    // A fresh code re-pairs.
    pair(addr, &server_pool, &client_pool).await;
}
