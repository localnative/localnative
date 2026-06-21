# TODO — Remaining Improvement Items

Items identified during code review that require larger effort or separate planning.

## High Effort

### Electron Native Module Rebuild
- `localnative-neon` uses neon-cli v0.3.1 (current is v1.0+)
- `electron-build-env` v0.2.0 is extremely outdated
- Neon API changed significantly — native module may need rewrite for modern neon
- Verify N-API compatibility with Electron 28+

### Test Coverage Expansion
- 93 tests in `localnative_core` (26 in `db.rs`, 10 in `rpc.rs`, rest spread across the crate) — up from 34 at the last review, but still concentrated in unit tests
- No integration tests for RPC sync (client ↔ server round-trip)
- No tests for GUI state management (`localnative_iced/src/lib.rs`)
- No tests for Electron renderer logic
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

### Electron Dependency Updates
- ~~`@zxing/library` v0.18.6 → v1.3+~~ — done, bumped to `^0.23.0` (2026-06)
- `crossfilter2` pinned at v1.5.4 — evaluate if upgrade is safe
- `d3` v6.7.0 → v7+ (minor breaking changes in imports)
- `dc` v4.2.7 → v5+ (check compatibility with crossfilter2)
- `roddeh-i18n` v1.2.1 — verify UMD build works correctly with script tag loading
- `glob` v7.2.3 → v10+ (ESM-only in v9+, may need alternative)

## Medium Effort

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

### Electron Post-Migration Verification
- After running `npm install`, verify all UMD script paths resolve:
  - `node_modules/underscore/underscore-umd.js`
  - `node_modules/d3/dist/d3.js`
  - `node_modules/crossfilter2/crossfilter.js`
  - `node_modules/dc/dist/dc.js`
  - `node_modules/@zxing/library/umd/index.min.js`
  - `node_modules/roddeh-i18n/i18n.js`
- Test that `contextBridge` APIs work end-to-end (neon-run, file dialog, screenshots)
- Verify CSP headers don't block any legitimate functionality

### CI Pipeline
- ~~`.gitlab-ci.yml` lint/fmt commands were fixed but pipeline hasn't been validated~~ — clippy/fmt gate is green; Tauri CI bumped to Node 20 and lockfile fixed (4ae9d58, 8efc68e)
- Consider adding `cargo test` step if not already present
- Consider adding Electron build/test step

## Out of Scope (Major Architecture)

- **Full Electron → Tauri migration**: The Tauri frontend already exists at `localnative-tauri/` — consider deprecating Electron
- **Database migration to async SQLx**: Currently uses synchronous `rusqlite` with `spawn_blocking`; async SQLx would remove mutex contention
- **RPC protocol upgrade**: tarpc is functional but consider gRPC or QUIC for better cross-platform sync
- **Browser extension modernization**: Manifest V3 migration for Chrome extension
