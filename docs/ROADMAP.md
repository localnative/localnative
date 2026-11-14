# Roadmap — "Remember the web, locally"

Strategic plan of 2026-11-14. The goal is **growth of the open source project**:
more users and contributors; the app stays free (AGPL-3.0), revenue is not a
near-term driver.

The pivot in one line: LocalNative stops being "a bookmark app" and becomes
**the local-first home for what you read** — save it permanently (Archive),
read and highlight it anywhere with no account (Read later), ask your own
history questions offline (Recall), all carried by an engine worth extracting
(Sync). Each phase ships user-visible value, is a release + blog beat, and
feeds the next phase.

## What we already own (the unfair advantage)

- One Rust core (`localnative_core`) behind every shell: Android, iOS, Tauri,
  browser extension, CLI. Hardened by the 2026-09-26 review.
- Serverless P2P sync: Noise-encrypted transport (`secure.rs`), mDNS
  discovery (`discovery.rs`), BLAKE2s 256-bucket reconciliation, HLC tokens,
  byte-budgeted batches, protocol-version negotiation (`wire.rs`, `db/sync.rs`).
- Local SQLite with trigram FTS including CJK; SQLCipher opt-in.
- `annotations` column that already round-trips binary content as base64
  data: URLs (migration-era C6), and a `metadata` JSON column per note.
- An MV3 extension with `scripting` + `activeTab` already granted, talking to
  the core through an allow-listed native-messaging host.
- A trademark on the name, and CI that builds every platform from core
  changes (R1–R3).

Nothing below requires a central server, an account, or a hosted relay. That
constraint *is* the brand.

---

## Phase 0 — Positioning and contributor on-ramp (ships with Phase 1)

Not a code phase; the packaging around the pivot.

- Rewrite README + localnative.app landing around the new narrative and one
  tagline: **"Remember the web, locally."** New demo GIF/video on the landing
  page (current screenshots show the 0.4.0 era).
- `docs/ARCHITECTURE.md`: core layout, one diagram of the shells around the
  core, the sync protocol in five paragraphs, and "where to add your first
  feature" pointers. The test culture (69 unit + 9 integration tests, real
  migration chain) should be presented as the contributor safety net.
- Umbrella naming for marketing only — no repo renames: LocalNative Archive,
  Read Later, Recall, Sync all name features of "LocalNative".
- GitHub Discussions on; every phase opens a set of `good first issue`
  tickets scoped in advance.
- Distribution audit: Play Store + F-Droid for Android, AMO + Chrome Web
  Store for the extension, Tauri releases for desktop — all listing copy
  updated to the new narrative at the Phase 1 release.

## Phase 1 — Archive: "your own archive.org"

**User story:** pressing save keeps the page itself — readable text now,
faithful snapshot when possible — searchable forever, on my disk, synced to
my devices.

### Engineering

- **Object store (core, new `objects.rs` + migration):** content-addressed
  store at `~/LocalNative/objects/<blake2s>`; tables
  `object(hash TEXT PRIMARY KEY, size INTEGER, media_type TEXT, created_at TEXT)`
  and `note_snapshot(note_uuid4 TEXT, object_hash TEXT, kind TEXT,
  created_at TEXT, token TEXT)` with the usual HLC write token. `kind` ∈
  `article_html | article_text | raw_html | screenshot`.
  Article *text* (tens of KB) is note-adjacent data and syncs through the
  existing note payload from day one; *objects* (raw HTML, screenshots) sync
  through a new manifest reconciliation (below).
- **FTS:** new trigram FTS table over article text keyed by object hash,
  joined into search results so a hit inside a page body surfaces the note.
  Snippets come from the article text, not just the description.
- **Capture (extension):** default save flow becomes capture: Readability.js
  parse in the popup (DOMParser) for `article_html` + `article_text`; raw
  `fetch()` of the URL for `raw_html` when same-origin policy allows;
  `tabs.captureVisibleTab` screenshot as an opt-in later step. SingleFile-style
  full DOM serialization is a follow-up, not a blocker — readable article +
  raw HTML is 90% of the archival value.
- **Allow-list:** the native-messaging host's action allow-list (P1) gains
  `snapshot`/`object` commands; still no note content on any log path.
- **Viewer (Tauri):** objects served through a custom `lnarchive://` protocol
  into a sandboxed webview; article HTML sanitized (ammonia) at store time;
  snapshot viewing strictly offline (`default-src 'self'`, no remote loads —
  mirrors P4's CSP discipline).
- **Mobile:** Android/iOS render `article_html` in the platform WebView via
  the core; raw HTML opens in a viewer screen. No new permissions.
- **Export/import:** export bundles objects (or skips and reports, per C5
  behavior); import adopts by hash — dedupe is free because everything is
  content-addressed.
- **Sync:** the wire protocol gains an object-manifest exchange (hash →
  have/need) beside the existing note reconciliation; blobs move in
  byte-budgeted batches inside the same Noise session. Protocol version bump
  per docs/VERSIONING.md. **This is the phase's main engineering risk** —
  budget an interop test like `bulk_sync_moves_every_note` but for objects.

### Brand promise note

"Notes in one SQLite file" becomes "your archive in open files on your disk":
one SQLite database plus a content-addressed object directory. Still local,
inspectable, greppable, yours — and no server. The website copy should say
exactly that. Open decision to settle early: under `--features encryption`,
objects stay plaintext (hash-named) or get an encrypted container; the first
release documents whichever is true, the second release fixes it.

### Release beat

Versioned release + blog post "Your own archive.org" + landing demo. The
launch hook for outside audiences: link rot and platform decay; ArchiveBox needs
a server, SingleFile has no search/sync/mobile.

## Phase 2 — Read later + highlights: "Pocket's ghost, with no account"

**User story:** save now, read later on any device — offline on my phone —
and highlight the good parts; highlights come back when I revisit the page.

### Engineering

- **Reading state:** table `note_state(note_uuid4 TEXT PRIMARY KEY, state
  TEXT, token TEXT)` — `unread | reading | done`, LWW via token, syncs like
  any note-derived record. Queue views (Unread/Reading/Done) in Tauri and
  mobile; the extension's primary button becomes "Save for later".
- **Reader view:** typography-first rendering of `article_html` in Tauri and
  the mobile WebViews (fonts, measure, dark mode, font size). Reading on a
  phone offline over the P2P sync from Phase 1 is the demo.
- **Highlights:** a content script annotates selections; anchors stored
  Hypothesis-style (text quote + prefix/suffix context, robust to DOM drift)
  as typed entries in the note's `annotations` JSON (`type: "highlight"`) —
  no schema migration, the column already round-trips. Re-render highlights
  on revisit by URL lookup through the host.
- **Mobile highlights:** text selection in the reader produces the same
  annotation entries — one format, every platform.
- **UniFFI:** the deferred typed bindings (see TODO.md product decisions)
  land here, because this phase is exactly the "mobile-facing core feature"
  the deferral was waiting for. Kotlin/Swift move onto typed records before
  the reader UIs are written against them.

### Release beat

"Pocket died in 2025; here is a read-later app that never uploads your
library." That sentence is the blog post. Show phone + desktop syncing on a
home LAN with the cable unplugged.

## Phase 3 — Recall: local AI over your own corpus

**User story:** "where did I read about X?" — semantic search, then optional
answers, entirely on my machine.

### Engineering

- **Embeddings (core, opt-in):** table `object_embedding(object_hash TEXT,
  model TEXT, vec BLOB)`. **No new native dependencies in v1** — pure-Rust
  brute-force cosine over f32 blobs is instant at personal scale (100k docs ×
  384 dims ≈ 150 MB scan), keeps the iOS/Android builds green, and defers
  sqlite-vec until measurements ask for it.
- **Providers:** v1 targets a user-configured local endpoint (Ollama or any
  OpenAI-compatible localhost server) for embeddings and optional synthesis —
  zero bundling/licensing questions, zero build risk. A bundled model
  (candle/ort MiniLM class) is a per-platform follow-up. Default off; there
  is never a cloud endpoint in the settings screen.
- **Hybrid retrieval:** trigram FTS candidates + vector rerank; results cite
  the notes (deep-link to the snapshot viewer). "Ask your archive" panel in
  Tauri; `localnative ask "…"` in the CLI. Synthesis (local LLM) is optional
  and always shows its citations.
- **Privacy line, stated:** embedding happens on-device or on the user's own
  localhost model; nothing leaves the machine; the feature ships off.

### Release beat

"Chat with everything you've read — offline." The 2026 demo that travels:
airplane-mode Q&A over a personal archive.

## Phase 4 — Sync: extract the engine

**User story (developer):** "SQLite sync with Noise encryption and mDNS, zero
servers, as a crate."

- Carve `localnative-sync` out of `localnative_core` (`secure.rs`,
  `discovery.rs`, `wire.rs`, HLC, bucket reconciliation) generic over a small
  `Record` trait (content hash, version token, chunked serialization);
  note-specific validation stays in the core, which becomes the crate's first
  consumer. The object-manifest exchange from Phase 1 is the second record
  type and the proof the abstraction holds.
- `docs/SYNC.md`: the wire spec (handshake, framing, version negotiation per
  VERSIONING.md) published for interop; interop tests that pin a client
  against a frozen byte stream.
- Bindings: UniFFI (shared with mobile from Phase 2), WASM later.
- Stay AGPL-3.0 — the goal is contributors and users; dual licensing is a
  decision for a day someone asks.
- Sneaker-net escape hatch instead of a relay: formalize "sync via file" for
  devices that never share a LAN (export/import already nearly does this);
  the no-server story stays unbroken.

### Release beat

Crates.io release + spec post: "serverless SQLite sync, and the reference
implementation is a note app you can read end to end."

---

## Sequencing rationale

Archive → Read later → Recall → Sync. Each phase's data model is the next
phase's substrate: the archive corpus is what read-later organizes and recall
searches; three phases of real traffic over blobs and records is what makes a
credible extracted engine. Every phase ends in a public beat aimed at a
different audience (self-hosters, Pocket refugees, local-AI crowd, app
developers) — cumulative rather than repetitive.

## Risks and open decisions

- **Object sync over the wire** (Phase 1's hard part): new manifest
  reconciliation + blob batching; mitigate by reusing the byte-budget/session
  machinery and an interop test from day one.
- **Encryption vs. objects**: decide plaintext-hash-named vs. container
  before Phase 1 ships; document honestly in the first release either way.
- **MV3 service-worker lifecycle** during large captures: offscreen document
  for parsing; capture must complete or cleanly resume.
- **Highlight anchor drift**: quote+context anchors degrade gracefully;
  accept "unanchored highlight keeps its text" as the fallback state.
- **Iced** stays frozen (per 2026-09-26 product decisions); none of these
  phases touch it.
- **Out of scope for now**: PKM/backlinks, team/multi-user sync, commercial
  licensing, any hosted relay. B1-style evidence hashing (BLAKE2 + timestamp
  + signed bundle) remains a cheap future *mode* on top of Phase 1, not a
  phase of its own.

## Living document

Execution tracking stays in TODO.md (the live backlog); this file changes
only when strategy changes. Phase kickoff = a TODO.md section with the
tickets, including the `good first issue` set promised in Phase 0.
