# Roadmap — "Remember the web, locally"

Strategic plan of 2026-11-14. The goal is **growth of the open source project**:
more users and contributors; the app stays free (AGPL-3.0), revenue is not a
near-term driver.

The pivot in one line: LocalNative stops being "a bookmark app" and becomes
**the local-first home for what you read** — save the readable text permanently
(Archive), let your own AI tools retrieve it (MCP), read and highlight it
anywhere with no account (Read later), ask your history questions offline
(Recall), all carried by an engine worth extracting (Sync). Each phase ships
user-visible value, is a release + blog beat, and feeds the next phase.

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
  feature" pointers. The migration-chain unit and integration tests should be
  presented as the contributor safety net.
- Umbrella naming for marketing only — no repo renames: LocalNative Archive,
  Read Later, Recall, Sync all name features of "LocalNative".
- GitHub Discussions on; every phase opens a set of `good first issue`
  tickets scoped in advance.
- Distribution audit: Play Store + F-Droid for Android, AMO + Chrome Web
  Store for the extension, Tauri releases for desktop — all listing copy
  updated to the new narrative at the Phase 1 release.

## Phase 1 — Text Archive: "your own archive.org"

**User story:** pressing save keeps the page's readable text — searchable
forever, on my disk, synced to my devices.

### Engineering

- **Storage (core):** readable UTF-8 article text lives in the existing
  `annotations` column. It therefore uses the existing note payload, HLC
  conflict resolution, export and Noise sync unchanged. Binary annotations
  remain supported but are not indexed as article text. No object store, new
  blob protocol, or raw HTML snapshot is in this phase.
- **FTS:** schema 0.12.0 adds text annotations to the trigram index and the
  short-term LIKE fallback. A hit inside an article surfaces its note.
- **Capture (extension):** save content extracts `article`, `main`, or body
  text in the active tab, removes non-content elements, normalizes whitespace,
  and caps oversized captures. Pages that disallow injection still save the
  note. Protocol logs redact note and article content.
- **Retrieval (core):** a `get` command returns one live note by UUID for the
  reader and MCP. Unknown UUIDs return the normal error envelope.
- **Reader (Tauri):** an in-app dialog renders article text as text, never as
  HTML, with comfortable measure, scrolling, dark mode, and a source link.
  No remote content or new webview protocol is involved.
- **Verification:** real migration-chain tests cover old article text,
  insert/update/delete indexing, short terms, CJK and binary annotations;
  the extension passes its syntax gate and the Tauri frontend passes check
  and build.

### Brand promise note

The promise stays "your archive in one SQLite file." Article text inherits the
database's SQLCipher behavior when encryption is enabled and remains plaintext
in the default build, exactly like the rest of a note. Raw HTML, screenshots,
content-addressed objects and their encryption format are later decisions, not
hidden work in this release.

### Release beat

Versioned release + blog post "Remember the web, locally" + landing demo. The
launch hook is link rot without a server: save readable text, search it, and
sync it using the existing open database and peer protocol.

## Phase 2 — MCP: "your reading history, available to your agents"

**User story:** an AI tool I already use can search my saved reading and open a
result with its source URL, without LocalNative hosting a model or uploading my
library.

### Engineering

- **Command:** `localnative mcp` in `localnative_cli`, speaking MCP JSON-RPC
  over stdio. Stdout carries protocol messages only; diagnostics go to stderr.
  The transitional `localnative-mcp` alias dispatches to the same implementation.
- **Read-only tools:** `search_notes(query, limit, offset)` returns compact
  result metadata without article bodies; `get_note(uuid4)` returns the full
  note, including archived text and its source URL. There is no save, update,
  delete, sync, or remote command in the first server.
- **Core reuse:** calls go through the same JSON dispatcher as every other
  shell and honor `LOCALNATIVE_DB`; there is no second query language or
  platform-specific business logic.
- **No model dependency:** embeddings, local endpoints and synthesis stay out
  of this phase. MCP clients supply their own agent or model.
- **Verification:** unit tests cover initialize, tools/list, argument
  validation, compact search results and get-by-UUID; a real stdio session
  runs against a scratch database containing an archived article.

### Release beat

"Ask your AI about everything you've read — locally." Show an MCP client
finding an article and citing its original URL.

## Phase 3 — Read later + highlights: "Pocket's ghost, with no account"

**User story:** save now, read later on any device — offline on my phone —
and highlight the good parts; highlights come back when I revisit the page.

### Engineering

- **Reading state:** table `note_state(note_uuid4 TEXT PRIMARY KEY, state
  TEXT, token TEXT)` — `unread | reading | done`, LWW via token, syncs like
  any note-derived record. Queue views (Unread/Reading/Done) in Tauri and
  mobile; the extension's primary button becomes "Save for later".
- **Reader view:** typography-first rendering of the archived plain text in
  Tauri and native mobile scrolling views (fonts, measure, dark mode, font
  size). Reading on a phone offline over existing note sync is the demo.
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

## Phase 4 — Recall: local AI over your own corpus

**User story:** "where did I read about X?" — semantic search, then optional
answers, entirely on my machine.

### Engineering

- **Embeddings (core, opt-in):** table `note_embedding(note_uuid4 TEXT,
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

## Phase 5 — Sync: extract the engine

**User story (developer):** "SQLite sync with Noise encryption and mDNS, zero
servers, as a crate."

- Carve `localnative-sync` out of `localnative_core` (`secure.rs`,
  `discovery.rs`, `wire.rs`, HLC, bucket reconciliation) generic over a small
  `Record` trait (content hash, version token, chunked serialization);
  note-specific validation stays in the core, which becomes the crate's first
  consumer. A later second record type is the proof that the abstraction
  holds; it is not invented just to make the SDK look general.
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

Text Archive → MCP → Read later → Recall → Sync. The archive corpus is what
MCP exposes, read-later organizes and recall searches; real application use,
not a desire to publish a crate, is what makes a credible extracted engine.
Every phase ends in a public beat aimed at a different audience (self-hosters,
AI-tool users, Pocket refugees, local-AI crowd, app developers) — cumulative
rather than repetitive.

## Risks and open decisions

- **Capture size**: list responses now carry `has_archive` instead of article
  bodies, and readers fetch full text with `get`. Keep capture capped to bound
  database size, `get` responses, and peer sync payloads.
- **Legacy HTML annotations**: older saves may contain HTML rather than plain
  text. They remain searchable and render escaped, never as live markup; a
  one-click re-capture/migration is a follow-up.
- **MV3 service-worker lifecycle** during large captures: extraction runs in
  the active tab and returns one bounded result; failure still saves the note.
- **Highlight anchor drift**: quote+context anchors degrade gracefully;
  accept "unanchored highlight keeps its text" as the fallback state.
- **Desktop shell:** Tauri is the only desktop front-end. Iced was removed after
  its supported workflows and preferences were covered; these phases do not
  maintain a second desktop UI.
- **Out of scope for now**: PKM/backlinks, team/multi-user sync, commercial
  licensing, any hosted relay. B1-style evidence hashing (BLAKE2 + timestamp
  + signed bundle) remains a cheap future *mode* on top of Phase 1, not a
  phase of its own.

## Living document

Execution tracking stays in TODO.md (the live backlog); this file changes
only when strategy changes. Phase kickoff = a TODO.md section with the
tickets, including the `good first issue` set promised in Phase 0.
