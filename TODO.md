# TODO

Live backlog. The first section is the architecture review of 2026-09-26: each
item records the defect, the evidence, and the fix. `[x]` means fixed and
verified in the working tree; `[ ]` means still open.

## Architecture review (2026-09-26)

Measured against the project's own vision — notes in a local SQLite file, no
central service, instant type-ahead search, one Rust core under thin platform
shells, serverless LAN sync. Evidence was gathered with a throwaway harness
against `localnative_core` (temp databases, `HOME` redirected) plus the real
Android/iOS/Tauri toolchains.

### Sync — security and correctness

- [x] **S1 — Sync aborted after 20 notes.** (Server rate limiter + per-note
  RPC + untyped errors; pushes of a 200-note backlog stopped at 20 → 40 → 60.)
  Fixed: governor removed; one session per sync with byte-budgeted batches,
  each applied in one transaction; typed wire errors. Integration test
  `bulk_sync_moves_every_note` moves 200 notes in one session.
- [x] **S2 — Any LAN host could read, overwrite and wipe every note.** Fixed:
  Noise-encrypted transport (`secure.rs`), `Noise_XX` between paired devices,
  `Noise_XXpsk3` pairing with a one-time 16-char code (5-minute window),
  trusted peers in the `peer` table, nothing before authentication, remote
  `stop` removed (`server-stop` is local-only). Integration tests:
  `unpaired_client_is_refused`, `wrong_pairing_code_fails_the_handshake`,
  `repairing_after_forgetting`, `server_stop_is_local`.
- [x] **S3 — Forged tokens were accepted.** Fixed: `NoteV1::validate` checks
  UUID, sizes, and `updated_at` (shape + at most 7 days in the future) on every
  received note; migration 0.11.0 rewrites malformed stored tokens. Tests:
  `forged_tokens_are_rejected`, `forged_tokens_are_refused_end_to_end`.
- [x] **S4 — Clock skew undid deletes.** Fixed: hybrid logical clock — a
  persisted `hlc` high-water mark in `meta`, advanced past every observed
  remote token; every local write is strictly newer than the version it
  replaces. Tests: `local_delete_survives_a_skewed_peer_clock`,
  `skewed_clock_cannot_resurrect_a_delete`.
- [x] **S5 — Deleting didn't delete.** Fixed: content is blanked on delete, on
  received tombstones and in file merges; migration strips existing
  tombstones. Tests: `delete_strips_content_and_propagates`,
  `received_tombstone_with_content_is_stored_stripped`,
  `deletes_propagate_and_stay_deleted`.
- [x] **S6 — Protocol couldn't scale or evolve.** Fixed: one session per sync
  (no per-note RPCs), 256-bucket BLAKE2s reconciliation before version
  exchange, byte-budgeted batches with chunked annotations, a dedicated
  `wire::NoteV1` decoupled from the API model, protocol-version negotiation
  (the schema version no longer gates sync), exports strip `node_id`/keys.
- [x] **S7 — mDNS discovery never found a peer.** Fixed: `enable_addr_auto()`,
  `<name>.local.` host, TXT `proto`/`node`/`name`, self filtered by node id.
- [x] **S8 — The core popped desktop notifications.** `notify-rust` removed
  (it dragged D-Bus into mobile builds); sync returns counts, shells report.

### Search

- [x] **Q1 — CJK and substring search were broken; the trigram index was never
  queried.** Fixed: trigram `MATCH` for terms of 3+ characters, escaped `LIKE`
  for shorter ones (trigram can't match them); the unicode61 `note_fts` index
  was dropped (0.11.0). Tests: `search_finds_cjk_words_and_substrings`,
  `search_escapes_like_wildcards`, `search_matches_tags_and_urls`.
- [x] **Q2 — Tags were normalized only in some shells.** Fixed: the core
  normalizes (trim, dedupe case-insensitively) on every write and migrates
  existing rows; empty tags never reach the facet lists. Tests:
  `tags_are_normalized_on_write`, migration chain tests.

### Privacy

- [x] **P1 — The web-ext host logged note bodies to `debug.log`** and died
  silently on a read-only working directory. Fixed: no file logging, no
  content on stderr, a reply on every failure path, and an allow-list of the
  actions the extension may send.
- [x] **P2 — Android backed the database up to Google Drive** and release
  builds logged commands. Fixed: `allowBackup="false"` +
  `data_extraction_rules` (cloud excluded, device-to-device transfer allowed),
  no content logging.
- [x] **P3 — `--features encryption` could produce a plaintext DB that
  believed it was encrypted.** Fixed: the feature enables
  `rusqlite/bundled-sqlcipher` itself, and `set_encryption_key` fails unless
  `PRAGMA cipher_version` answers.
- [x] **P4 — Tauri phoned home on every launch** and ran with `csp: null`.
  Fixed: update check is manual (Settings → Check for Updates); CSP
  `default-src 'self'` (+ IPC origins) in `tauri.conf.json`.

### Core API and schema

- [x] **C1 — Three error shapes crossed the FFI.** Fixed: one envelope
  `{"error": <message>, "code": <stable code>}`; the failing command is no
  longer echoed; egui/iced/extension read it. Tests: `test_error_envelope`,
  `test_error_codes_are_stable`.
- [x] **C2 — The core hard-coded the database path.** Fixed:
  `db::set_db_path`, `LOCALNATIVE_DB`, CLI `--db`, C/JNI setters; Android
  passes its real `filesDir` (`RustBridge.init`); `temp_store=MEMORY` replaces
  the hard-coded temp dir.
- [x] **C3 — A fresh database was stamped 0.8.0 until its second open** and
  migrations weren't transactional. Fixed: frozen baseline + every migration
  in one call, each migration in a transaction, all tests run the real chain.
  Tests: `fresh_database_reaches_current_version_in_one_open`,
  `legacy_0_3_10_database_migrates_all_the_way`.
- [x] **C4 — Unintegrated or dead code.** Removed: `search_with_snippets`, the
  metadata helpers, `get_server_addr`, `get_if_addrs` (replaced by one
  maintained `if-addrs` helper).
- [x] **C5 — Import/export defects.** Fixed: read-only `ATTACH` (sources
  untouched, read-only backups work), legacy columns synthesized in SQL,
  tombstoned URLs re-import, export truncates long filenames and
  skip-and-reports, YAML values are JSON-quoted. Tests: `import_db_leaves_the
  _source_untouched`, `import_db_accepts_a_read_only_source`,
  `import_skips_notes_the_user_deleted`, export tests.
- [x] **C6 — `annotations` was hex on read.** Fixed: read paths return UTF-8
  text as-is and binary content as a base64 `data:` URL — the same forms the
  write side accepts. Test: `annotations_round_trip_text_and_binary`.
- [x] **C7 — The C header was stale.** Rewritten: correct constness, the new
  `localnative_set_db_path` entry point.
- [x] **C8 — Unchecked `as` casts in the core.** Audited; remaining ones are
  `saturating_add`/`try_from` or provably lossless (u32→usize), with comments.

### Front-ends

- [x] **F1 — Iced.** Sync errors reach the screen (the core's envelope, no
  downcast); "Close server" calls the core's local `server-stop` instead of
  awaiting a token that never fires; IP input parses with `IpAddr` instead of
  a broken regex; right-clicking the chart clears the date range; a new
  search/tag click starts on page 1; sync-from-file failures are shown; the
  server address comes from the core (no 8.8.8.8 route needed); the window
  icon is RGBA. New: pairing-code field and a pairing-code display. The
  ouroboros self-referencing `SyncView` is a plain struct.
- [x] **F2 — egui.** Reads the `{"error","code"}` envelope; the OS CJK font
  loads as a fallback (Chinese titles no longer render as boxes); pairing-code
  field wired.
- [x] **F3 — Tauri.** Pagination keeps the search query (`search`, not
  `select`); sync results and errors surface on the sync page; the address
  accepts any host:port; dead `ssb-sync` removed; the server stops locally
  (no loopback RPC); the broadcast `local_ip` trick is gone; crate metadata
  fixed. Pairing UI wired.
- [x] **F4 — Android.** JNI calls run on `Dispatchers.IO` (no ANR on sync);
  commands are built with `JSONObject` (a quote in a query no longer breaks
  search); sync/search errors surface in the UI; `RustBridge.init` passes the
  real `filesDir` database path.
- [x] **F5 — iOS didn't compile.** Fixed: `MMWormhole` import removed from the
  compiled `AppDelegate.swift`; the share-extension handoff is a queue drained
  on launch/foreground/notification (no overwrite, no lost shares); commands
  built with `JSONSerialization`; sync/search run off the main thread;
  orphaned uncompiled files deleted; `Helpers/AppGroupsHelper.swift` added to
  the Xcode project. Verified: `xcodebuild` simulator build succeeds.
- [x] **F6 — CLI.** `-a/--addr` no longer panics (`get_one::<String>`);
  `localnative-rpc-server` keeps running (parks) and prints the pairing code
  with `--pair`; results print and exit codes are real; `--db` everywhere.
- [x] **F7 — The native-messaging host installer was duplicated.** Fixed: one
  shared crate, `localnative_hostinstall`, used by Iced and Tauri; Tauri's
  `unwrap()`s are gone.
- [ ] **F8 — The unshipped WASM popup asks for more than the shipped one**
  (`tabs`, `<all_urls>`). Open: depends on the product decision below.

### Release engineering and docs

- [x] **R1 — Android release builds never compiled the core.** Fixed: the
  `buildRustCore` Gradle task runs `cargo ndk -o` into `jniLibs` on every
  build; CI and the Play deploy workflow install the Rust toolchain, NDK and
  cargo-ndk and fail if any ABI's `liblocalnative_core.so` is missing from the
  artifact; `xtask ndkbd` uses `-o`. Verified locally: `assembleDebug`
  produced an APK with all four ABIs.
- [x] **R2 — iOS linked a symlink into deprecated `cargo lipo` output.**
  Fixed: `script/build-ios.sh` builds device + simulator libraries into
  `localnative-ios/LocalNativeCore.xcframework`, which the Xcode project now
  links; `ci_post_clone.sh` builds it on Xcode Cloud. Verified: simulator
  build succeeds.
- [x] **R3 — CI blind spots.** Fixed: clippy runs with `--all-features`; a new
  `mobile-core` job compiles the core for Android and both iOS targets; core
  changes trigger Android and Tauri CI.
- [x] **R4 — Dead or broken scripts.** Removed the four obsolete
  `build-android-*` scripts (one contained the cross-wired ABI copy),
  `release-web-ext-host` (copied into the removed `localnative-neon`), and
  `localnative-docker` (Rust 1.55 + GitLab). Fixed `build_linux.sh` (cwd) and
  `release-iced-mac` (any arch).
- [x] **R5 — Docs drift.** Fixed: DB path is `~/LocalNative` (+ overrides);
  sync quick-start describes pairing; Electron/Flutter/neon instructions
  removed; README's sub-directory list corrected; VERSIONING.md documents the
  protocol-version gate; CLAUDE.md describes the new core layout and sync;
  changelog entry written.

### Tests

- [x] **T1 — No integration tests.** Fixed: `localnative_core/tests/
  sync_integration.rs` runs real server+client pairs: bulk sync, pairing,
  unpaired refusal, forged tokens, delete propagation, clock skew, both-way
  edits, local stop. Unit tests moved onto the real migration chain (scratch
  files). 69 unit + 9 integration tests pass.

### Product decisions (recommendations, not code changes)

- **One desktop shell.** Recommendation: Tauri (OS webview gives CJK fonts,
  IME and accessibility; shares UI with the extension popup). If a pure-Rust
  UI matters more, egui — the CJK font fix in F2 applies either way. Freeze
  Iced once the choice is made; delete the 38-line `localnative-mac` stub.
- **`is_public`** has had no meaning since SSB was removed. Either give it one
  (for example "never leaves this device") or drop it from the UI.
- **One extension popup.** Ship the WASM popup or delete it; maintaining both
  doubles the work. Its git dependencies on personal-fork branches need an
  upstream release first (this is also F8).
- **Typed bindings (UniFFI).** Replacing hand-written JSON on mobile with
  generated Kotlin/Swift bindings would prevent the F4/F5 class of bugs.

## Backlog (carried over)

### egui Desktop Front-end (`localnative_egui/`)
Remaining work to reach Iced parity:
- Note editing — core only inserts/deletes; needs an update path
- Arbitrary date-range filtering / a date-picker widget (single-day filtering is wired)
- Localization via Fluent (Iced uses `translate.rs` + `locales/`)
- Day-histogram chart visualization (Iced uses `plotters_bridge.rs`)
- Hosting a sync server + mDNS peer discovery
- Import/export entry points (core `import.rs`, `export.rs`)
- Wire into `xtask release` packaging and CI once it reaches parity

### Reduce Excessive `.clone()` in GUI Layer
- `localnative_iced/src/chart.rs`: `raw.clone()` in `fold_map` callers, `data.clone().into_iter()`, `will_draw.days.clone()`
- `localnative_iced/src/tags.rs`: `self.tag.tag.clone()` — consider `Arc<String>`
- Profile first to confirm these are real bottlenecks

### Test gaps outside the core
- No tests for GUI state management (`localnative_iced/src/lib.rs`)
- No tests for the browser extension / WASM app

### Out of scope (major architecture)
- **Async SQLx**: the core is synchronous `rusqlite`; searches take ~10 ms at
  13k notes, so there is no current performance case for it.
