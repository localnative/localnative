# TODO — Remaining Improvement Items

Items identified during code review that require larger effort or separate planning.

## High Effort

### Test Coverage Expansion
- 93 tests in `localnative_core` (26 in `db.rs`, 10 in `rpc.rs`, rest spread across the crate) — up from 34 at the last review, but still concentrated in unit tests
- No integration tests for RPC sync (client ↔ server round-trip)
- No tests for GUI state management (`localnative_iced/src/lib.rs`)
- No tests for browser extension / WASM app
- Consider adding `tokio::test` integration tests for `rpc::sync()` with an in-memory server

### Sync Hardening — follow-ups
Conflict-resolving sync (per-row last-write-wins + tombstones, schema 0.10.0) has
landed in `localnative_core`; edits and deletes now propagate. Remaining work:
- **Hybrid Logical Clock**: `updated_at` is currently a monotonic *physical*
  clock with a `node_id` tiebreak (`db::next_update_token`). This is adequate
  only when peer clocks are roughly synced; a fast/skewed clock can always win
  and could resurrect a tombstone. Replace the token source with an HLC (e.g.
  `uhlc`) that advances the local clock past timestamps observed from peers.
  Single seam to change: `next_update_token`. Verify the HLC builds and behaves
  on `wasm32-unknown-unknown` (browser extension has no system clock).
- **Encrypted + authenticated transport** (the other critical gap): RPC traffic
  is still plaintext Bincode over TCP bound to `0.0.0.0:3456` with no pairing —
  any LAN host can enumerate UUIDs and read note bodies. Wrap the tarpc
  transport in Noise (`snow`, pairing-code-as-PSK) or rustls/TLS with pinned
  per-device certs, plus a device-pairing UX and a trusted-key store. Must be
  pure-Rust and build on wasm/Android/iOS (rules out iroh/libp2p for the
  browser extension's no-relay LAN path).
- **Scale**: replace the full `(uuid4, updated_at)` list exchange with a
  merkle/range-hash diff once correctness and security are in place.
- The exact-string `meta_version` gate in `rpc.rs` is intentionally strict: it
  blocks sync between peers with incompatible `Note` wire formats (e.g. pre-0.10
  vs 0.10). Only relax it alongside a real wire-compatibility scheme.

## Medium Effort

### egui Desktop Front-end (`localnative_egui/`)
An egui/eframe front-end was scaffolded as the first step of the desktop
consolidation onto egui (retiring Iced and the Mac stub). It wraps
`localnative_core` directly: FTS search, tag filter, day-histogram filter
(`do_filter`), add note, soft-delete, pagination, and peer sync (off the UI
thread via `run_sync`). Remaining work to reach Iced parity:
- Note editing — core only inserts/deletes; needs an update path (or delete + re-insert)
- Arbitrary date-range filtering / a date-picker widget (single-day filtering is wired)
- Localization via Fluent (Iced uses `translate.rs` + `locales/`)
- Day-histogram chart visualization (Iced uses `plotters_bridge.rs`)
- Hosting a sync server + mDNS peer discovery (core `rpc::start`, `discovery`)
- Import/export entry points (core `import.rs`, `export.rs`)
- Wire into `xtask release` packaging and CI once it reaches parity

### Reduce Excessive `.clone()` in GUI Layer
- `localnative_iced/src/chart.rs`: `raw.clone()` at lines 107, 126, 136 — change `fold_map` to accept `&Vec<Day>`
- `localnative_iced/src/chart.rs`: `data.clone().into_iter()` at line 232 — use `data.iter()` or `data.into_iter()`
- `localnative_iced/src/chart.rs`: `will_draw.days.clone()` at lines 441, 450, 459 — pass by reference
- `localnative_iced/src/tags.rs`: `self.tag.tag.clone()` at line 24 — consider `Arc<String>` for tag strings
- These require profiling to confirm they're actual bottlenecks

### RPC Rate Limiting Enhancements
- Current implementation uses server-wide limiters; consider per-IP keyed rate limiting with `governor::RateLimiter<IpAddr, ...>`
- Add configurable rate limit values (currently hardcoded 100/20 req/s)
- Add rate limit headers or error details in `RpcError::RateLimited` response
- Log rate-limited requests with client IP for monitoring

### Type Cast Safety Audit
- `localnative_core/src/db.rs`: Remaining `as` casts should be audited
- Search for `as u32`, `as i64`, `as usize` across the codebase
- Replace with `try_from()` where overflow is possible

## Low Effort

### Clippy Lint Fixes
- 2 `too_many_arguments` warnings from `ouroboros` `#[self_referencing]` macro in `sync.rs` — these are macro-generated and cannot be suppressed without a file-level allow

### CI Pipeline
- ~~`.gitlab-ci.yml` lint/fmt commands were fixed but pipeline hasn't been validated~~ — clippy/fmt gate is green (now `.github/workflows/rust.yml`; the GitLab pipeline was retired along with the GitLab hosting); Tauri CI bumped to Node 20 and lockfile fixed (4ae9d58, 8efc68e)
- Consider adding `cargo test` step if not already present

## Out of Scope (Major Architecture)

- **Database migration to async SQLx**: Currently uses synchronous `rusqlite` with `spawn_blocking`; async SQLx would remove mutex contention
- **RPC protocol upgrade**: tarpc is functional but consider gRPC or QUIC for better cross-platform sync
- **Browser extension modernization**: Manifest V3 migration for Chrome extension
