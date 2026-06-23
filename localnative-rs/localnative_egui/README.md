# localnative_egui

An [egui](https://github.com/emilk/egui)/[eframe](https://github.com/emilk/egui/tree/master/crates/eframe)
desktop front-end for Local Native, part of the desktop consolidation onto egui
(retiring the Iced, Electron, and Mac front-ends).

Like every Local Native front-end, this crate is a thin shell over
`localnative_core`. It calls the synchronous `db` query layer directly (the same
path `localnative_iced` uses) on the UI thread, since local SQLite is fast. Peer
sync is the one slow operation, so it runs on a background thread via
`localnative_core::run_sync` and is polled each frame through an `mpsc` channel.

## Run

```bash
cd localnative-rs
cargo run -p localnative_egui
```

The database lives at the same location every front-end uses
(`~/LocalNative/localnative.sqlite3` on desktop), so notes are shared with the
CLI, Iced GUI, and browser-extension host.

## Features (scaffold)

- Full-text search (FTS5) with an empty-query "show all" default, newest first
- Tag sidebar — click a tag to filter
- Add note (title, URL, tags, description, comments, public/private)
- Soft-delete (tombstone) a note
- Pagination (20 notes per page)
- Peer sync against a `host:port` address, run off the UI thread

## Not yet wired up

- Date/day histogram and date-range filtering (`do_filter` exists in core)
- Hosting a sync server / mDNS peer discovery (core: `rpc::start`, `discovery`)
- Editing an existing note (core currently inserts/deletes only)
- Localization (the Iced front-end uses Fluent; not yet ported here)
- Import/export hooks (core: `import.rs`, `export.rs`)
- Optional SQLCipher encryption (core `encryption` feature)
