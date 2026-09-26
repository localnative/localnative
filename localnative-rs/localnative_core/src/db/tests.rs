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

//! Database tests. Every database goes through the real migration chain
//! ([`super::init_db_at`]) on a scratch file — no hand-rolled schemas, so a
//! broken migration fails here instead of on a user's device.

use super::migrations;
use super::queries;
use super::sync as dbsync;
use super::*;
use crate::wire::NoteV1;
use rusqlite::Connection;
use std::path::PathBuf;

fn scratch_path(tag: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "ln_core_test_{}_{}_{tag}.sqlite3",
        std::process::id(),
        line!()
    ))
}

/// A migrated database on a scratch file.
fn test_db(tag: &str) -> (PathBuf, Connection) {
    let path = scratch_path(tag);
    for suffix in ["", "-wal", "-shm"] {
        let _ = std::fs::remove_file(format!("{}{suffix}", path.display()));
    }
    let conn = init_db_at(&path).expect("open + migrate scratch db");
    (path, conn)
}

/// A database file from an older schema, to exercise the migration chain.
/// `create` builds the era's tables — including `meta` with its version row
/// where that era had one (0.3.x had no `meta` table at all, which is why
/// `migrations::migrate_schema` can use a plain INSERT).
fn legacy_db(tag: &str, create: &str) -> PathBuf {
    let path = scratch_path(tag);
    let _ = std::fs::remove_file(&path);
    let conn = Connection::open(&path).unwrap();
    conn.execute_batch(create).unwrap();
    drop(conn);
    path
}

// ── Schema ────────────────────────────────────────────────────────────────

#[test]
fn fresh_database_reaches_current_version_in_one_open() {
    let (_path, conn) = test_db("fresh");
    let version = migrations::get_meta_version(&conn).unwrap();
    assert_eq!(version, migrations::current_version().to_string());
}

#[test]
fn upgrade_is_idempotent() {
    let (_path, conn) = test_db("idempotent");
    migrations::upgrade(&conn).expect("second upgrade is a no-op");
    migrations::upgrade(&conn).expect("third upgrade is a no-op");
}

#[test]
fn legacy_0_3_10_database_migrates_all_the_way() {
    // No meta table, no uuid4 — exactly what a 0.3.10 database looked like.
    let path = legacy_db(
        "legacy_0_3",
        "CREATE TABLE note (
             rowid INTEGER PRIMARY KEY AUTOINCREMENT,
             title TEXT NOT NULL, url TEXT NOT NULL, tags TEXT NOT NULL,
             description TEXT NOT NULL, comments TEXT NOT NULL, annotations TEXT NOT NULL,
             created_at TEXT NOT NULL, is_public BOOLEAN NOT NULL DEFAULT 0);
         INSERT INTO note (title, url, tags, description, comments, annotations, created_at, is_public)
         VALUES ('Old note', 'https://old.example', 'rust, rust', 'old', '', '',
                 '2018-01-01 10:00:00:123 UTC', 0);",
    );
    let conn = init_db_at(&path).expect("migrations from 0.3.10");
    assert_eq!(
        migrations::get_meta_version(&conn).unwrap(),
        migrations::current_version().to_string()
    );
    let result = queries::do_search(&conn, "old", 10, 0).unwrap();
    assert_eq!(result.count, 1, "migrated note is searchable");
    let note = &result.notes[0];
    assert!(!note.uuid4.is_empty(), "uuid4 backfilled");
    assert_eq!(note.tags, "rust", "duplicate tag collapsed");
    // Every row carries a usable last-write-wins token.
    let stored = dbsync::get_wire_note(&conn, &note.uuid4)
        .unwrap()
        .expect("stored note");
    assert!(super::token_millis(&stored.updated_at).is_some());
}

// ── Search ────────────────────────────────────────────────────────────────

#[test]
fn search_finds_cjk_words_and_substrings() {
    let (_p, conn) = test_db("search_cjk");
    for (title, url) in [
        ("如何学习Rust编程", "https://a.example/rust"),
        ("JavaScript tips and tricks", "https://b.example/js"),
        ("Go语言入门教程", "https://c.example/go"),
        ("Weekly AI digest", "https://d.example/ai"),
    ] {
        queries::insert_note(&conn, title, url, "", "", "", b"", false).unwrap();
    }

    let hits = |q: &str| queries::do_search(&conn, q, 10, 0).unwrap().count;
    // CJK words the unicode61 tokenizer could never match.
    assert_eq!(hits("学习"), 1, "two-character CJK term");
    assert_eq!(hits("编程"), 1);
    assert_eq!(hits("语言"), 1);
    assert_eq!(hits("入门教程"), 1);
    // Substrings inside words.
    assert_eq!(hits("script"), 1, "script finds JavaScript");
    assert_eq!(hits("Script"), 1, "substring match is case-insensitive");
    assert_eq!(hits("学习Rust"), 1, "mixed-script term");
    // Multi-term AND across strategies.
    assert_eq!(hits("rust 学习"), 1);
    assert_eq!(hits("rust go"), 0);
    // FTS operators in user input stay literal.
    assert_eq!(hits("OR"), 0);
    // Empty query lists everything.
    assert_eq!(hits(""), 4);
}

#[test]
fn search_escapes_like_wildcards() {
    let (_p, conn) = test_db("search_like");
    queries::insert_note(&conn, "100% done", "", "", "", "", b"", false).unwrap();
    queries::insert_note(&conn, "a_b", "", "", "", "", b"", false).unwrap();
    queries::insert_note(&conn, "back\\slash", "", "", "", "", b"", false).unwrap();
    queries::insert_note(&conn, "unrelated", "", "", "", "", b"", false).unwrap();

    // Wildcard characters in short terms are matched literally, not as
    // patterns: "%" finds the note containing a percent sign and nothing else.
    assert_eq!(queries::do_search(&conn, "100%", 10, 0).unwrap().count, 1);
    assert_eq!(queries::do_search(&conn, "%", 10, 0).unwrap().count, 1);
    assert_eq!(queries::do_search(&conn, "a_b", 10, 0).unwrap().count, 1);
    assert_eq!(queries::do_search(&conn, "_", 10, 0).unwrap().count, 1);
    assert_eq!(queries::do_search(&conn, "back\\", 10, 0).unwrap().count, 1);
}

#[test]
fn search_matches_tags_and_urls() {
    let (_p, conn) = test_db("search_fields");
    queries::insert_note(
        &conn,
        "plain",
        "https://example.com/page",
        "travel",
        "",
        "",
        b"",
        false,
    )
    .unwrap();
    assert_eq!(queries::do_search(&conn, "travel", 10, 0).unwrap().count, 1);
    assert_eq!(queries::do_search(&conn, "page", 10, 0).unwrap().count, 1);
}

#[test]
fn filter_honors_date_range_with_empty_query() {
    let (_p, conn) = test_db("filter_range");
    queries::insert_note_with_timestamp(
        &conn,
        "Day One",
        "https://one.example",
        "a,b",
        "",
        "",
        b"",
        true,
        "2026-06-20 09:00:00",
    )
    .unwrap();
    queries::insert_note_with_timestamp(
        &conn,
        "Day Two",
        "https://two.example",
        "b,c",
        "",
        "",
        b"",
        true,
        "2026-06-21 09:00:00",
    )
    .unwrap();
    queries::insert_note_with_timestamp(
        &conn,
        "Day Two Again",
        "https://three.example",
        "c",
        "",
        "",
        b"",
        true,
        "2026-06-21 18:30:00",
    )
    .unwrap();

    let day = queries::do_filter(&conn, "", 10, 0, "2026-06-21", "2026-06-21").unwrap();
    assert_eq!(day.count, 2);
    assert!(
        day.notes
            .iter()
            .all(|n| n.created_at.starts_with("2026-06-21"))
    );
    // Tag aggregation is date-scoped too: 'a' (only on 2026-06-20) is absent.
    assert!(day.tags.iter().all(|t| t.tag != "a"));
    assert!(day.tags.iter().any(|t| t.tag == "c"));

    // The day histogram spans every day, so the range can be widened again.
    assert_eq!(day.days.len(), 2, "histogram ignores the range");

    let other = queries::do_filter(&conn, "", 10, 0, "2026-06-20", "2026-06-20").unwrap();
    assert_eq!(other.count, 1);
    assert_eq!(other.notes[0].title, "Day One");
}

// ── Tags ──────────────────────────────────────────────────────────────────

#[test]
fn tags_are_normalized_on_write() {
    assert_eq!(normalize_tags("rust, web"), "rust,web");
    assert_eq!(normalize_tags("rust, Rust, RUST"), "rust");
    assert_eq!(normalize_tags(",, ,"), "");
    assert_eq!(normalize_tags("a, b , c"), "a,b,c");

    let (_p, conn) = test_db("tags_write");
    queries::insert_note(&conn, "n1", "", "Rust, web", "", "", b"", false).unwrap();
    queries::insert_note(&conn, "n2", "", "rust,web ,", "", "", b"", false).unwrap();
    queries::insert_note(&conn, "n3", "", "", "", "", b"", false).unwrap();

    let result = queries::do_select(&conn, 10, 0).unwrap();
    let mut tags: Vec<(String, i64)> = result
        .tags
        .iter()
        .map(|t| (t.tag.clone(), t.count))
        .collect();
    tags.sort();
    assert_eq!(tags, vec![("rust".into(), 2), ("web".into(), 2)]);
}

// ── Delete / tombstones ───────────────────────────────────────────────────

#[test]
fn delete_strips_content_and_propagates() {
    let (_p, conn) = test_db("tombstone");
    let note = queries::insert_note(
        &conn,
        "Therapy session notes",
        "https://private.example/x",
        "private",
        "very personal description",
        "my comments",
        b"attachment",
        false,
    )
    .unwrap();
    queries::delete_note(&conn, note.rowid).unwrap();

    // Hidden from reads...
    assert_eq!(queries::do_select(&conn, 10, 0).unwrap().count, 0);
    // ...retained as a tombstone with no content, still advertised to peers.
    let versions = dbsync::note_versions(&conn).unwrap();
    assert_eq!(versions.len(), 1);
    let tomb = dbsync::get_wire_note(&conn, &note.uuid4)
        .unwrap()
        .expect("tombstone exists");
    assert!(tomb.deleted);
    assert!(tomb.title.is_empty());
    assert!(tomb.description.is_empty());
    assert!(tomb.comments.is_empty());
    assert!(tomb.annotations.is_empty());

    // A peer holding the live note hides it once the tombstone arrives, and
    // the peer's copy carries no content afterwards either.
    let (_pp, peer) = test_db("tombstone_peer");
    let mut live = tomb.clone();
    live.deleted = false;
    live.title = "Therapy session notes".into();
    live.description = "very personal description".into();
    live.annotations = b"attachment".to_vec();
    live.updated_at = "00000000000000000001-old".into();
    dbsync::apply_notes(&peer, &[live]).unwrap();
    assert_eq!(queries::do_select(&peer, 10, 0).unwrap().count, 1);
    dbsync::apply_notes(&peer, &[tomb]).unwrap();
    assert_eq!(queries::do_select(&peer, 10, 0).unwrap().count, 0);
    let stored = dbsync::get_wire_note(&peer, &note.uuid4).unwrap().unwrap();
    assert!(stored.deleted && stored.title.is_empty() && stored.description.is_empty());
}

#[test]
fn received_tombstone_with_content_is_stored_stripped() {
    let (_p, conn) = test_db("tomb_forge");
    let mut forged = NoteV1 {
        uuid4: "550e8400-e29b-41d4-a716-446655440000".into(),
        title: "should not survive".into(),
        url: "https://x.example".into(),
        tags: "t".into(),
        description: "body".into(),
        comments: "c".into(),
        annotations: b"attachment".to_vec(),
        created_at: "2026-01-01 00:00:00".into(),
        is_public: false,
        metadata: "{}".into(),
        updated_at: format!("{:020}-peer", now_millis()),
        deleted: true,
    };
    dbsync::apply_notes(&conn, &[forged.clone()]).unwrap();
    let stored = dbsync::get_wire_note(&conn, &forged.uuid4)
        .unwrap()
        .unwrap();
    assert!(stored.deleted);
    assert!(stored.title.is_empty() && stored.description.is_empty());

    // And a live note that arrives with an empty token gets a local one.
    forged.deleted = false;
    forged.updated_at = String::new();
    // An empty token is invalid by the wire rules; apply_notes must reject it.
    let outcome = dbsync::apply_notes(&conn, &[forged]).unwrap();
    assert_eq!(outcome.rejected, 1);
}

// ── Last-write-wins and the clock ─────────────────────────────────────────

#[test]
fn lww_newer_wins_older_ignored() {
    let (_p, conn) = test_db("lww");
    let original = queries::insert_note(&conn, "Original", "", "", "", "", b"", false).unwrap();

    // A token slightly in the future is valid (clock tolerance); one second
    // after the original is enough to win.
    let mut newer = NoteV1::from_stored(&conn, &original.uuid4);
    newer.title = "Edited".into();
    newer.updated_at = format!("{:020}-peer", now_millis() + 1_000);
    dbsync::apply_notes(&conn, &[newer]).unwrap();
    assert_eq!(stored_title(&conn, &original.uuid4), "Edited");

    let mut older = NoteV1::from_stored(&conn, &original.uuid4);
    older.title = "Stale".into();
    older.updated_at = "00000000000000000001-peer".into();
    dbsync::apply_notes(&conn, &[older]).unwrap();
    assert_eq!(stored_title(&conn, &original.uuid4), "Edited");
}

fn stored_title(conn: &Connection, uuid4: &str) -> String {
    dbsync::get_wire_note(conn, uuid4).unwrap().unwrap().title
}

impl NoteV1 {
    /// Test helper: the stored note as wire form.
    fn from_stored(conn: &Connection, uuid4: &str) -> NoteV1 {
        dbsync::get_wire_note(conn, uuid4)
            .unwrap()
            .expect("note exists")
    }
}

#[test]
fn forged_tokens_are_rejected() {
    let (_p, conn) = test_db("forge");
    for token in ["~", "x", "", "123-peer", "999999999999999999999999-peer"] {
        let mut note = NoteV1 {
            uuid4: "550e8400-e29b-41d4-a716-446655440099".into(),
            title: token.into(),
            ..Default::default()
        };
        note.updated_at = token.to_string();
        note.title = "forged".into();
        let outcome = dbsync::apply_notes(&conn, &[note]).unwrap();
        assert_eq!(outcome.rejected, 1, "token {token:?} must be rejected");
    }
    assert_eq!(queries::do_select(&conn, 10, 0).unwrap().count, 0);

    // A token in the near future is accepted (clock tolerance), the far
    // future is not.
    let now = now_millis();
    let soon = format!("{:020}-peer", now + 60_000);
    let far = format!("{:020}-peer", now + MAX_CLOCK_SKEW_MS + 86_400_000);
    let note = NoteV1 {
        uuid4: "550e8400-e29b-41d4-a716-446655441099".into(),
        created_at: "2026-01-01 00:00:00".into(),
        updated_at: soon,
        ..Default::default()
    };
    assert_eq!(dbsync::apply_notes(&conn, &[note]).unwrap().applied, 1);
    let note = NoteV1 {
        uuid4: "550e8400-e29b-41d4-a716-446655441199".into(),
        created_at: "2026-01-01 00:00:00".into(),
        updated_at: far,
        ..Default::default()
    };
    assert_eq!(dbsync::apply_notes(&conn, &[note]).unwrap().rejected, 1);
}

#[test]
fn local_delete_survives_a_skewed_peer_clock() {
    // Peer A saved a note while its clock ran 10 minutes fast; B (correct
    // clock) deletes it. B's delete must win even though A's token carries a
    // higher millisecond value, and B must never resurrect the note.
    let (_pa, a) = test_db("skew_a");
    let (_pb, b) = test_db("skew_b");

    let note = queries::insert_note(
        &a,
        "Saved on A",
        "https://a.example",
        "",
        "",
        "",
        b"",
        false,
    )
    .unwrap();
    let fast_ms = now_millis() + 10 * 60 * 1000;
    a.execute(
        "UPDATE note SET updated_at = ?1 WHERE uuid4 = ?2",
        rusqlite::params![format!("{fast_ms:020}-node-a"), note.uuid4],
    )
    .unwrap();

    // B receives it and deletes it locally.
    let wire = dbsync::get_wire_note(&a, &note.uuid4).unwrap().unwrap();
    dbsync::apply_notes(&b, &[wire]).unwrap();
    let rowid: i64 = b
        .query_row(
            "SELECT rowid FROM note WHERE uuid4 = ?1",
            [&note.uuid4],
            |r| r.get(0),
        )
        .unwrap();
    queries::delete_note(&b, rowid).unwrap();
    assert_eq!(queries::do_select(&b, 10, 0).unwrap().count, 0);

    // Both directions of the next sync keep it deleted.
    let tomb = dbsync::get_wire_note(&b, &note.uuid4).unwrap().unwrap();
    dbsync::apply_notes(&a, &[tomb]).unwrap();
    assert_eq!(
        queries::do_select(&a, 10, 0).unwrap().count,
        0,
        "the delete propagated to the fast-clock peer"
    );
    let resurrect = dbsync::get_wire_note(&a, &note.uuid4).unwrap().unwrap();
    dbsync::apply_notes(&b, &[resurrect]).unwrap();
    assert_eq!(
        queries::do_select(&b, 10, 0).unwrap().count,
        0,
        "the delete is never undone on B"
    );

    // Any later local write outranks the skewed token.
    let again = queries::insert_note(&a, "New note", "", "", "", "", b"", false).unwrap();
    let token = dbsync::get_wire_note(&a, &again.uuid4)
        .unwrap()
        .unwrap()
        .updated_at;
    assert!(
        super::token_millis(&token).unwrap() > fast_ms,
        "HLC advanced past the skewed value: {token} vs {fast_ms}"
    );
}

// ── Reconciliation helpers ────────────────────────────────────────────────

#[test]
fn buckets_partition_and_diff() {
    let (_p, conn) = test_db("buckets");
    for i in 0..50 {
        queries::insert_note(&conn, &format!("note {i}"), "", "", "", "", b"", false).unwrap();
    }
    let versions = dbsync::note_versions(&conn).unwrap();
    assert_eq!(versions.len(), 50);
    assert!(versions.iter().all(|(_, ts)| !ts.is_empty()));

    let hashes = dbsync::bucket_hashes(&versions);
    assert_eq!(hashes.len(), dbsync::BUCKETS);

    // Identical sets → no differing buckets; a changed row moves exactly its
    // own bucket.
    assert!(dbsync::differing_buckets(&hashes, &hashes).is_empty());
    let mut changed = versions.clone();
    changed[0].1.push('x');
    let changed_hashes = dbsync::bucket_hashes(&changed);
    let differing = dbsync::differing_buckets(&hashes, &changed_hashes);
    assert_eq!(
        differing,
        vec![dbsync::bucket_of(&versions[0].0)],
        "only the bucket holding the changed row differs"
    );

    // An empty client pulls everything it is missing from those buckets —
    // which may legitimately hold more than the one changed note.
    let bucket = dbsync::bucket_of(&versions[0].0);
    let expected = versions
        .iter()
        .filter(|(uuid4, _)| dbsync::bucket_of(uuid4) == bucket)
        .count();
    let diff = dbsync::diff_versions(&conn, &differing, vec![]).unwrap();
    assert_eq!(diff.pull.len(), expected);
    assert!(diff.pull.contains(&versions[0].0));
}

#[test]
fn load_notes_respects_budget() {
    let (_p, conn) = test_db("budget");
    for i in 0..20 {
        queries::insert_note(
            &conn,
            &format!("note {i}"),
            "",
            "",
            "",
            "",
            &vec![0u8; 4096],
            false,
        )
        .unwrap();
    }
    let versions = dbsync::note_versions(&conn).unwrap();
    let uuids: Vec<String> = versions.iter().map(|(u, _)| u.clone()).collect();

    let (notes, consumed) = dbsync::load_notes(&conn, &uuids, 16 * 1024).unwrap();
    assert_eq!(consumed, 3, "three 4KiB notes fit a 16KiB budget");
    assert_eq!(notes.len(), 3);
    assert!(notes[0].annotations.len() == 4096);

    // Unknown UUIDs are consumed without a note, so the loop can progress.
    let mut with_ghost = vec!["ghost-uuid".to_string()];
    with_ghost.extend(uuids);
    let (notes, consumed) = dbsync::load_notes(&conn, &with_ghost, 16 * 1024).unwrap();
    assert_eq!(consumed, 4);
    assert_eq!(notes.len(), 3);
}

// ── Import / export ───────────────────────────────────────────────────────

/// Restore ordinary write permission on a scratch file (readonly(true) sets
/// only the owner bit; this restores the standard 0o644 rather than
/// world-writable).
fn make_writable(path: &std::path::Path) {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o644)).unwrap();
    }
    #[cfg(not(unix))]
    {
        let mut perms = std::fs::metadata(path).unwrap().permissions();
        perms.set_readonly(false);
        std::fs::set_permissions(path, perms).unwrap();
    }
}

fn legacy_0_9_backup(tag: &str) -> PathBuf {
    legacy_db(
        tag,
        "CREATE TABLE note (
             rowid INTEGER PRIMARY KEY AUTOINCREMENT,
             uuid4 TEXT NOT NULL UNIQUE, title TEXT NOT NULL, url TEXT NOT NULL,
             tags TEXT NOT NULL, description TEXT NOT NULL, comments TEXT NOT NULL,
             annotations TEXT NOT NULL, created_at TEXT NOT NULL,
             is_public BOOLEAN NOT NULL DEFAULT 0, metadata TEXT NOT NULL DEFAULT '{}');
         CREATE TABLE meta (meta_key TEXT PRIMARY KEY, meta_value TEXT NOT NULL);
         INSERT INTO meta VALUES ('version', '0.9.0');
         INSERT INTO note (uuid4, title, url, tags, description, comments, annotations, created_at, is_public)
         VALUES ('550e8400-e29b-41d4-a716-446655440000', 'Backup note', 'https://b.example', ' Rust , rust ', '', '', '', '2024-01-01 00:00:00', 0);",
    )
}

#[test]
fn import_db_leaves_the_source_untouched() {
    let (_p, conn) = test_db("import_ro");

    let src = legacy_0_9_backup("import_src");
    let before = std::fs::read(&src).unwrap();
    let merged = queries::import_db(&conn, src.to_str().unwrap()).unwrap();
    assert_eq!(merged, 1);
    assert_eq!(
        std::fs::read(&src).unwrap(),
        before,
        "source bytes unchanged"
    );

    // Legacy columns were synthesized on the fly: the source file still has
    // no updated_at column of its own.
    let check = Connection::open(&src).unwrap();
    let columns: Vec<String> = check
        .prepare("SELECT name FROM pragma_table_info('note')")
        .unwrap()
        .query_map([], |r| r.get(0))
        .unwrap()
        .collect::<Result<_, _>>()
        .unwrap();
    assert!(!columns.contains(&"updated_at".to_string()));

    // The note arrived with tags normalized and a legacy token.
    let note = dbsync::get_wire_note(&conn, "550e8400-e29b-41d4-a716-446655440000")
        .unwrap()
        .unwrap();
    assert_eq!(note.tags, "Rust");
    assert!(note.updated_at.ends_with("-legacy"));

    // Re-importing the same (not-newer) backup is idempotent under LWW.
    let again = queries::import_db(&conn, src.to_str().unwrap()).unwrap();
    assert_eq!(again, 0);
}

#[test]
fn import_db_accepts_a_read_only_source() {
    let (_p, conn) = test_db("import_readonly");
    let src = legacy_0_9_backup("import_src_ro");
    let mut perms = std::fs::metadata(&src).unwrap().permissions();
    perms.set_readonly(true);
    std::fs::set_permissions(&src, perms).unwrap();

    let merged = queries::import_db(&conn, src.to_str().unwrap());
    assert!(merged.is_ok(), "read-only source: {merged:?}");

    make_writable(&src);
}

#[test]
fn import_skips_notes_the_user_deleted() {
    let (_p, conn) = test_db("import_deleted");
    let note = queries::insert_note(
        &conn,
        "Once saved",
        "https://re.example",
        "",
        "",
        "",
        b"",
        true,
    )
    .unwrap();
    queries::delete_note(&conn, note.rowid).unwrap();

    let result = crate::import::import_notes(
        &conn,
        vec![crate::import::ImportedNote {
            title: "Once saved".into(),
            url: "https://re.example".into(),
            tags: String::new(),
            description: String::new(),
            created_at: None,
        }],
    )
    .unwrap();
    // A tombstoned URL is not a duplicate: the user deleted it on purpose.
    assert_eq!(result.imported, 1, "{result:?}");
}

#[test]
fn export_strips_device_identity() {
    let (path, conn) = test_db("export_identity");
    queries::insert_note(
        &conn,
        "Exported",
        "https://e.example",
        "x",
        "",
        "",
        b"",
        true,
    )
    .unwrap();

    let dest = scratch_path("export_identity_dest");
    let _ = std::fs::remove_file(&dest);
    queries::export_db(&conn, dest.to_str().unwrap()).unwrap();
    // Re-export over an existing target is idempotent.
    queries::export_db(&conn, dest.to_str().unwrap()).unwrap();

    let copy = Connection::open(&dest).unwrap();
    let count: i64 = copy
        .query_row("SELECT count(*) FROM note", [], |r| r.get(0))
        .unwrap();
    assert_eq!(count, 1);
    for key in ["node_id", "hlc", "sync_private_key", "sync_public_key"] {
        let value: Option<String> = copy
            .query_row(
                "SELECT meta_value FROM meta WHERE meta_key = ?1",
                [key],
                |r| r.get(0),
            )
            .ok();
        assert!(value.is_none(), "export must not carry {key}");
    }
    let peers: i64 = copy
        .query_row("SELECT count(*) FROM peer", [], |r| r.get(0))
        .unwrap();
    assert_eq!(peers, 0, "export must not carry paired devices");

    // Empty destination is rejected.
    assert!(queries::export_db(&conn, "  ").is_err());

    drop(conn);
    let _ = std::fs::remove_file(&path);
}

// ── Pagination & facets ───────────────────────────────────────────────────

#[test]
fn pagination() {
    let (_p, conn) = test_db("pagination");
    for i in 0..5 {
        queries::insert_note(
            &conn,
            &format!("Note {i}"),
            "https://example.com",
            "tag",
            "",
            "",
            b"",
            false,
        )
        .unwrap();
    }
    assert_eq!(queries::do_select(&conn, 2, 0).unwrap().count, 5);
    assert_eq!(queries::do_select(&conn, 2, 0).unwrap().notes.len(), 2);
    assert_eq!(queries::do_select(&conn, 2, 2).unwrap().notes.len(), 2);
    assert_eq!(queries::do_select(&conn, 2, 4).unwrap().notes.len(), 1);
}

#[test]
fn day_histogram_groups_by_day() {
    let (_p, conn) = test_db("days");
    for (uuid, date) in [
        ("550e8400-e29b-41d4-a716-446655440001", "2024-01-15"),
        ("550e8400-e29b-41d4-a716-446655440002", "2024-01-15"),
        ("550e8400-e29b-41d4-a716-446655440003", "2024-01-16"),
    ] {
        conn.execute(
            "INSERT INTO note (uuid4, title, url, tags, description, comments, annotations, created_at, is_public)
             VALUES (?1, 'A', '', '', '', '', '', ?2 || ' 10:00:00', 0)",
            rusqlite::params![uuid, date],
        )
        .unwrap();
    }
    let days = queries::do_select(&conn, 10, 0).unwrap().days;
    assert_eq!(days.len(), 2);
    assert_eq!(days[0].date.to_string(), "2024-01-15");
    assert_eq!(days[0].count, 2);
    assert_eq!(days[1].count, 1);
}

// ── Annotations round-trip ────────────────────────────────────────────────

#[test]
fn annotations_round_trip_text_and_binary() {
    let (_p, conn) = test_db("annotations");
    let text =
        queries::insert_note(&conn, "text", "", "", "", "", "正文注记".as_bytes(), false).unwrap();
    assert_eq!(text.annotations, "正文注记");

    let png = b"\x89PNG\r\n\x1a\n\x00\xff\xfe".to_vec();
    let image = queries::insert_note(&conn, "image", "", "", "", "", &png, false).unwrap();
    assert!(
        image.annotations.starts_with("data:image/png;base64,"),
        "binary annotations present as a data URL"
    );

    // ...and the image command path decodes a data URL back to bytes.
    let cmd = models::CmdInsert {
        title: "screenshot".into(),
        url: String::new(),
        tags: String::new(),
        description: String::new(),
        comments: String::new(),
        annotations: image.annotations.clone(),
        limit: 10,
        offset: 0,
        is_public: false,
    };
    cmd.process_image(&conn).unwrap();
    let stored = conn
        .query_row(
            "SELECT annotations FROM note WHERE title = 'screenshot'",
            [],
            |row| row.get::<_, Vec<u8>>(0),
        )
        .unwrap();
    assert_eq!(stored, png);
}

// ── Paired devices ────────────────────────────────────────────────────────

#[test]
fn peers_are_trusted_listed_and_forgotten() {
    let (_p, conn) = test_db("peers");
    let key = [7u8; 32];
    assert!(!peers::is_trusted(&conn, &key).unwrap());
    peers::trust(&conn, &key, "node-a", "Laptop", "192.168.1.5:2345").unwrap();
    assert!(peers::is_trusted(&conn, &key).unwrap());
    assert_eq!(
        peers::find_by_addr(&conn, "192.168.1.5:2345")
            .unwrap()
            .unwrap(),
        key.to_vec()
    );

    let list = peers::list(&conn).unwrap();
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].name, "Laptop");

    assert!(peers::forget(&conn, &hex::encode(key)).unwrap());
    assert!(!peers::is_trusted(&conn, &key).unwrap());
    assert!(!peers::forget(&conn, &hex::encode(key)).unwrap());
}

// ── Encryption (only meaningful with SQLCipher compiled in) ───────────────

#[cfg(all(test, feature = "encryption"))]
mod encryption_tests {
    use super::encryption::*;
    use super::*;

    fn scratch(tag: &str) -> PathBuf {
        scratch_path(tag)
    }

    #[test]
    fn keyed_database_round_trip() {
        let path = scratch("enc_roundtrip");
        let _ = std::fs::remove_file(&path);
        let conn = Connection::open(&path).unwrap();
        set_encryption_key(&conn, "test-secret-key").expect("set key");
        migrations::upgrade(&conn).expect("upgrade");
        queries::insert_note(
            &conn,
            "Encrypted Note",
            "https://example.com",
            "",
            "",
            "",
            b"",
            false,
        )
        .unwrap();
        assert_eq!(queries::do_select(&conn, 10, 0).unwrap().count, 1);
        drop(conn);
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn rekey_keeps_data() {
        let path = scratch("enc_rekey");
        let _ = std::fs::remove_file(&path);
        let conn = Connection::open(&path).unwrap();
        set_encryption_key(&conn, "old-key").unwrap();
        migrations::upgrade(&conn).unwrap();
        queries::insert_note(&conn, "Before rekey", "", "", "", "", b"", false).unwrap();
        change_encryption_key(&conn, "new-key").expect("rekey");
        assert_eq!(queries::do_select(&conn, 10, 0).unwrap().count, 1);
        drop(conn);
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn key_with_special_characters() {
        let path = scratch("enc_special");
        let _ = std::fs::remove_file(&path);
        let conn = Connection::open(&path).unwrap();
        set_encryption_key(&conn, "it's a \"key\" with 'quotes'").expect("special chars");
        queries::insert_note(&conn, "Special", "", "", "", "", b"", false).unwrap();
        assert_eq!(queries::do_select(&conn, 10, 0).unwrap().count, 1);
        drop(conn);
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn encrypt_existing_db_refuses_existing_destination() {
        let dir = std::env::temp_dir();
        let source = dir.join("ln_enc_test_source.sqlite3");
        let dest = dir.join("ln_enc_test_dest_exists.sqlite3");
        std::fs::write(&source, b"fake").unwrap();
        std::fs::write(&dest, b"fake").unwrap();
        let result = encrypt_existing_db(source.to_str().unwrap(), dest.to_str().unwrap(), "key");
        let _ = std::fs::remove_file(&source);
        let _ = std::fs::remove_file(&dest);
        assert!(result.is_err());
    }
}
