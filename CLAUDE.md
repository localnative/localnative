# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

> `AGENTS.md` and `GEMINI.md` both point here — this is the single source of truth for AI assistants.

## Project Overview

Local Native is a cross-platform tool for saving and syncing notes in a local SQLite database without going through any centralized service. A shared Rust core (`localnative_core`) is wrapped by every platform front-end: the Rust GUI (Iced), CLI, Tauri (the desktop app), Android, iOS, and a browser extension.

## Big-Picture Architecture

Everything funnels through one entry point. Each front-end serializes a command to **JSON**, hands the string to the core, and gets a JSON string back. There is no per-platform business logic — platforms are thin shells over the core.

### The single dispatch path (`localnative_core/src/lib.rs`)
- C FFI: `localnative_run(*const c_char) -> *mut c_char`, `localnative_set_db_path`, and `localnative_free` (header at `localnative_core/src/localnative-core.h`).
- Android JNI: `Java_app_localnative_android_RustBridge_localnativeRun` / `localnativeSetDbPath` (behind `#[cfg(target_os = "android")]`). The app passes its real `filesDir` via `RustBridge.init(context)`; `db::set_db_path`/`LOCALNATIVE_DB`/`--db` select the database everywhere.
- All of these call `run`/`run_sync` → `process`. A single shared Tokio `Runtime` (lazily built via `OnceLock`) lets the synchronous FFI callers `block_on` async work.
- `process` matches on the top-level `Cmd` enum (`#[serde(tag = "action", rename_all = "kebab-case")]`):
  - `Server` / `ServerPairing` / `ServerStop` / `ClientSync` / `Peers` / `PeerForget` → sync (`rpc.rs`), each opens an r2d2 `Pool` via `db::init_pool()`. Stopping a server is local-only (`server-stop`); there is no remote stop.
  - `DbCmd` (untagged) → delegates to `db::process_cmd` with a single `Connection` from `db::init_db()`.
- Errors never cross the FFI boundary as-is; every failure is one envelope: `{"error": <message>, "code": <stable machine code>}`.

### Database layer (`localnative_core/src/db/`)
This is **synchronous `rusqlite`** (bundled SQLite) with an **`r2d2` / `r2d2_sqlite` connection pool** — *not* SQLx. One module directory:
- `models` — request/response structs and the inner `Cmd` enum. **Add new database commands as variants here.**
- `queries` — SQL/CRUD, search, filtering, tag aggregation, file merge/import/export.
- `migrations` — version-keyed migration table (`MIGRATIONS: [(semver::Version, fn)]`) run automatically by `migrations::upgrade` on every open; each migration runs in its own transaction. Schema history: 0.4.0 → 0.11.0. To change the schema, append a new `(Version, migrate_fn)` entry — do **not** hand-edit existing migrations or the frozen baseline.
- `sync` — version sets, 256-bucket reconciliation hashes, and note apply/load in wire form.
- `peers` — the paired-device table used by the secure transport.
- `encryption` — optional **SQLCipher** support, gated behind the `encryption` cargo feature (which itself switches `rusqlite` to `bundled-sqlcipher`). Keying verifies `PRAGMA cipher_version`, so a plain build fails loudly instead of pretending to encrypt.

Search is FTS5 trigram for terms of 3+ characters and escaped `LIKE` for shorter ones (trigram can't match them). Last-write-wins tokens come from a hybrid logical clock persisted in `meta` (`hlc` row) — a local write always outranks the version it replaces and every token the device has observed.

Notes carry: title, URL, tags, description, comments, annotations (binary), timestamps, a UUID4, and a public/private flag.

### Import / Export (`localnative_core/src/import.rs`, `export.rs`)
Standalone core modules (not part of the JSON `Cmd` dispatch) driven by dedicated CLI binaries:
- `import.rs` parses external sources into `ImportedNote`s and bulk-inserts them with URL-based de-duplication (`import_notes`): Pocket HTML (`parse_pocket_html`), Omnivore JSON (`parse_omnivore_json`), Raindrop CSV (`parse_raindrop_csv`), Plinky JSON (`parse_plinky_json`).
- `export.rs` writes notes out as individual Markdown files with YAML frontmatter (`export_notes`), slugifying titles into filename-safe names.

### Sync (`localnative_core/src/rpc.rs`, `secure.rs`, `discovery.rs`, `wire.rs`)
- Peer-to-peer sync over one TCP connection per session, encrypted and mutually authenticated with **Noise** (`snow`): `Noise_XX_25519_ChaChaPoly_BLAKE2s` between paired devices, `Noise_XXpsk3` for first contact (pairing code as pre-shared key). No RPC runs before authentication; there is no remote stop.
- First contact pairs devices: the server shows a one-time 16-character code (5-minute window), the client enters it; both store the peer's static key in the `peer` table.
- Sessions: hello (protocol versions) → 256-bucket hash compare → versions for differing buckets → byte-budgeted batched push/pull, each batch applied in one transaction. Wire types live in `wire.rs`, separate from the API models, so the protocol can evolve independently.
- `sync-via-attach` (two-way file merge), `export-db` (standalone copy, identity stripped), and `import-db` (read-only one-way merge) share the same last-write-wins rules.
- **mDNS** service discovery via `mdns-sd` in `discovery.rs` (`_localnative._tcp.local.`, addresses via `enable_addr_auto`, TXT carries protocol/node id).

### Native GUI (`localnative_iced/`)
- Built on **Iced 0.14** (`iced::application(...).run()`); binary entry in `bin.rs`, state/update/view in `lib.rs`.
- Charts are rendered with a local **`plotters_bridge.rs`** — `plotters-iced` was dropped because it is incompatible with iced 0.14, so chart drawing is reimplemented against the raw `plotters` backend.
- Localization uses **Fluent** (`fluent-bundle`); translation strings live in `localnative-rs/locales/` and are wired through `translate.rs`.

## Common Commands

### Rust core (primary work happens here)
```bash
cd localnative-rs

cargo build                      # build the workspace
cargo run -p localnative_iced    # run the native GUI (Iced)
cargo test                       # run all tests
cargo test test_serde            # run a single test by name

# localnative_cli ships several binaries (src/bin/), not one — select with --bin:
cargo run -p localnative_cli --bin localnative-web-ext-host          # browser-extension native-messaging host
cargo run -p localnative_cli --bin localnative-import -- <args>      # import Pocket/Omnivore/Raindrop/Plinky
cargo run -p localnative_cli --bin localnative-export -- <args>      # export notes to Markdown
cargo run -p localnative_cli --bin localnative-export-db -- -o <file>  # VACUUM INTO single-file DB backup
cargo run -p localnative_cli --bin localnative-import-db -- -i <file>  # one-way LWW merge a DB file into the local DB
# other bins: localnative-rpc-server, localnative-rpc-client-sync,
#             localnative-rpc-client-stop-server, localnative-upgrade

# CI-equivalent lint/format gate (matches .github/workflows/rust.yml / xtask header):
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings -A clippy::type_complexity
```

### xtask (release & Android build automation)
```bash
cd localnative-rs
cargo run -p xtask -- release [-v <version>]   # package iced + web-ext-host into dist/
cargo run -p xtask -- ndkbd [--debug]          # cargo-ndk build of localnative_core .so for Android
```

### Front-ends
```bash
# Tauri (Svelte frontend) — uses yarn (yarn.lock is the committed lockfile)
cd localnative-tauri && yarn install && yarn dev           # build / lint / format scripts also available

# Android
cd localnative-android && ./gradlew assembleDebug          # installDebug to push to a device
```

Other platform front-ends: `localnative-ios`, `localnative-browser-extension`. Build scripts for packaging/cross-compiling live in `script/`.

## Conventions & Gotchas

- **Adding a database feature** = new variant in the `db::models` `Cmd` enum + handling in `db::process_cmd`/`queries`, plus a new `migrations` entry if the schema changes. Front-ends only need to learn the new JSON shape.
- **Cross-FFI errors** must be returned as JSON, never panicked across the boundary — follow `serialize_error`.
- The core is **synchronous rusqlite under a Tokio shim**; do not assume `async fn` query helpers exist. (`TODO.md` tracks a possible async migration — not yet done.)
- CI runs on **GitHub Actions** (`.github/workflows/`: rust with fmt + clippy `-D warnings` and per-crate builds, android, tauri, browser-extension, website, Play Store deploy). Keep it green; the clippy gate is strict. The code host is **github.com/localnative/localnative** (mirrors: srht, gitee, bitbucket, ssb).
- `TODO.md` is a live backlog of known tech debt — consult it before proposing large refactors.
- **Versioning**: platforms version independently — see `docs/VERSIONING.md`. Peer sync negotiates a **sync protocol version** (`wire::PROTOCOL_VERSION`), not the schema version: each device migrates its own database. The Rust crates share one `[workspace.package]` version; the browser extension, Android, iOS and Tauri each carry their own. Never bump one artifact to match another.
- License is **AGPL-3.0**; preserve the license header at the top of Rust source files.
