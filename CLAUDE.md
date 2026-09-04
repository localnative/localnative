# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

> `AGENTS.md` and `GEMINI.md` both point here — this is the single source of truth for AI assistants.

## Project Overview

Local Native is a cross-platform tool for saving and syncing notes in a local SQLite database without going through any centralized service. A shared Rust core (`localnative_core`) is wrapped by every platform front-end: native Rust GUIs (Iced, plus a newer egui/eframe front-end), CLI, Electron, Tauri, Flutter, Android, iOS, macOS, and a browser extension.

## Big-Picture Architecture

Everything funnels through one entry point. Each front-end serializes a command to **JSON**, hands the string to the core, and gets a JSON string back. There is no per-platform business logic — platforms are thin shells over the core.

### The single dispatch path (`localnative_core/src/lib.rs`)
- C FFI: `localnative_run(*const c_char) -> *mut c_char` and `localnative_free` (header generated at `localnative_core/src/localnative-core.h`).
- Android JNI: `Java_app_localnative_android_RustBridge_localnativeRun` (behind `#[cfg(target_os = "android")]`).
- All of these call `run`/`run_sync` → `process`. A single shared Tokio `Runtime` (lazily built via `OnceLock`) lets the synchronous FFI callers `block_on` async work.
- `process` matches on the top-level `Cmd` enum (`#[serde(tag = "action", rename_all = "kebab-case")]`):
  - `Server` / `ClientSync` / `ClientStopServer` → RPC/sync (`rpc.rs`), each opens an r2d2 `Pool` via `db::init_pool()`.
  - `DbCmd` (untagged) → delegates to `db::process_cmd` with a single `Connection` from `db::init_db()`.
- Errors are never propagated across the FFI boundary; they are serialized into the returned JSON (`serialize_error`).

### Database layer (`localnative_core/src/db.rs`)
This is **synchronous `rusqlite`** (bundled SQLite) with an **`r2d2` / `r2d2_sqlite` connection pool** — *not* SQLx, despite what older docs may say. The whole file is one module tree:
- `models` — request/response structs and the inner `Cmd` enum. **Add new database commands as variants here.**
- `queries` — the actual SQL/CRUD, search, filtering, tag aggregation.
- `migrations` — version-keyed migration table (`MIGRATIONS: [(semver::Version, fn)]`) run automatically by `migrations::upgrade` on every `init_db`/`init_pool`. Schema history: 0.4.0 → 0.10.0, including the move to **FTS5** full-text search (`migrate_fts5`, later `migrate_fts5_trigram`). To change the schema, append a new `(Version, migrate_fn)` entry — do **not** hand-edit existing migrations.
- `sync` — note diffing/merge logic used by RPC.
- `encryption` — optional **SQLCipher** support, gated behind `#[cfg(feature = "encryption")]`. Disabled by default; enabling it requires **both** the `localnative_core` `encryption` cargo feature **and** switching the `rusqlite` feature from `bundled` to `bundled-sqlcipher` in `localnative-rs/Cargo.toml`. The module exposes `init_db_encrypted(key)` (open + unlock + migrate), `set_encryption_key` (apply `PRAGMA key`, must run first on a connection), and `change_encryption_key` (`PRAGMA rekey`).

Notes carry: title, URL, tags, description, comments, annotations (binary), timestamps, a UUID4, and a public/private flag.

### Import / Export (`localnative_core/src/import.rs`, `export.rs`)
Standalone core modules (not part of the JSON `Cmd` dispatch) driven by dedicated CLI binaries:
- `import.rs` parses external sources into `ImportedNote`s and bulk-inserts them with URL-based de-duplication (`import_notes`): Pocket HTML (`parse_pocket_html`), Omnivore JSON (`parse_omnivore_json`), Raindrop CSV (`parse_raindrop_csv`), Plinky JSON (`parse_plinky_json`).
- `export.rs` writes notes out as individual Markdown files with YAML frontmatter (`export_notes`), slugifying titles into filename-safe names.

### RPC / Sync (`localnative_core/src/rpc.rs`, `discovery.rs`)
- **tarpc** peer-to-peer sync. Any client can `start` a server; others `sync` against it. Bi-directional, UUID-based conflict resolution; UUID lists are compared to decide what to transfer. Version compatibility is checked before transfer.
- Rate limiting via **`governor`** (currently server-wide, hardcoded quotas — see `TODO.md`).
- **mDNS** service discovery via `mdns-sd` in `discovery.rs` so peers can find each other on the LAN.

### Native GUI (`localnative_iced/`)
- Built on **Iced 0.14** (`iced::application(...).run()`); binary entry in `bin.rs`, state/update/view in `lib.rs`.
- Charts are rendered with a local **`plotters_bridge.rs`** — `plotters-iced` was dropped because it is incompatible with iced 0.14, so chart drawing is reimplemented against the raw `plotters` backend.
- Localization uses **Fluent** (`fluent-bundle`); translation strings live in `localnative-rs/locales/` and are wired through `translate.rs`.

### egui GUI (`localnative_egui/`)
- Newer desktop front-end on **eframe/egui 0.34** (wgpu backend), scaffolded as the first step of consolidating the desktop GUIs onto egui (retiring Iced, Electron, the Mac stub). Entry in `main.rs`; state/UI in `app.rs`.
- Unlike the JSON-dispatch FFI front-ends, it calls the synchronous `db::queries` layer directly on the UI thread (same as Iced); only peer sync runs off-thread, via `localnative_core::run_sync` polled through an `mpsc` channel.
- eframe 0.34 drives the app through `App::ui` (not the deprecated `update`); panels use `Panel::*` + `show_inside`. Not yet at Iced parity — see `TODO.md` ("egui Desktop Front-end").

## Common Commands

### Rust core (primary work happens here)
```bash
cd localnative-rs

cargo build                      # build the workspace
cargo run -p localnative_iced    # run the native GUI (Iced)
cargo run -p localnative_egui    # run the native GUI (egui/eframe)
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

# CI-equivalent lint/format gate (matches .gitlab-ci.yml / xtask header):
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
# Electron
cd localnative-electron && npm install && npm run dev      # (npm run build for native modules)

# Tauri (Svelte frontend) — uses yarn (yarn.lock is the committed lockfile)
cd localnative-tauri && yarn install && yarn dev           # build / lint / format scripts also available

# Flutter (flutter_rust_bridge 2.x; see localnative-flutter/SETUP.md)
cd localnative-flutter && make                             # check the Makefile for codegen + run targets

# Android
cd localnative-android && ./gradlew assembleDebug          # installDebug to push to a device
```

Other platform front-ends: `localnative-ios`, `localnative-mac`, `localnative-neon` (Node↔Rust bridge for Electron), `localnative-browser-extension` (+ `wasm-app/`), `localnative-docker`. Build scripts for packaging/cross-compiling live in `script/`.

## Conventions & Gotchas

- **Adding a database feature** = new variant in the `db::models` `Cmd` enum + handling in `db::process_cmd`/`queries`, plus a new `migrations` entry if the schema changes. Front-ends only need to learn the new JSON shape.
- **Cross-FFI errors** must be returned as JSON, never panicked across the boundary — follow `serialize_error`.
- The core is **synchronous rusqlite under a Tokio shim**; do not assume `async fn` query helpers exist. (`TODO.md` tracks a possible async migration — not yet done.)
- CI runs on **GitLab** (`.gitlab-ci.yml`: fmt + clippy `-D warnings`, then per-crate builds) and **GitHub Actions** (`.github/workflows/`: rust, android, tauri, browser-extension, website, Play Store deploy). Keep both green; the clippy gate is strict.
- `TODO.md` is a live backlog of known tech debt (outdated Electron/neon deps, sparse test coverage, `.clone()` hot spots, rate-limiter improvements) — consult it before proposing large refactors.
- **Versioning**: platforms version independently — see `docs/VERSIONING.md`. The only version shared across platforms is the **database schema version** in the `meta` table, which gates peer sync in `rpc.rs`. The Rust crates share one `[workspace.package]` version; the browser extension, Android, iOS, Tauri and Electron each carry their own. Never bump one artifact to match another.
- License is **AGPL-3.0**; preserve the license header at the top of Rust source files.
