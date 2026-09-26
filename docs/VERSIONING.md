# Versioning

## The rule

**Only one version is shared across platforms: the database schema version.
Every shipped artifact versions independently.**

Attempts to keep platform versions aligned are cosmetic, and the attempt has
already cost a release — see [Why](#why-this-policy-exists).

## Tier 1 — the schema version (the only cross-platform contract)

Stored in the SQLite `meta` table under `meta_key = 'version'`, advanced by the
migration table in `localnative_core/src/db/migrations.rs`.

A schema mismatch is a real error only for local reads — each device migrates
its own database. Peer sync is gated on the **sync protocol version**
(`wire::PROTOCOL_VERSION`, negotiated during the handshake), so devices with
different schema versions sync fine as long as they speak the same protocol.
An Android 0.6.1 phone and a Chrome 0.5.2 extension sync correctly *because*
their release numbers never enter the handshake.

- Bump the schema version **only** when the schema changes.
- Bump the protocol version **only** when the wire format changes.
- Never tie either to a release.
- Neither is user-visible.

## Tier 2 — shipped artifacts (independent, one per store)

Each store imposes rules that cannot be satisfied from a single shared number,
and review latency makes platforms drift by construction. Let them drift.

| Artifact | Version source | Store constraint |
|---|---|---|
| Browser extension | `localnative-browser-extension/app/manifest.json` | Strictly increasing per listing; values never reusable |
| Android | `localnative-android/app/build.gradle` | `versionCode` monotonic integer, independent of `versionName` |
| iOS | `MARKETING_VERSION` in the Xcode project | Marketing version + build number |
| macOS | `MARKETING_VERSION` in the Xcode project | — |
| Tauri | `src-tauri/tauri.conf.json` (`version`) | Keep `package.json` in step; the conf file is authoritative |

Bump an artifact when *that artifact* ships. Never bump one to match another.

## Tier 3 — the Rust crates (one shared number)

`localnative_core`, `localnative_cli`, `localnative_iced` and `xtask` ship
together and carry one version, declared once in `[workspace.package]` in
`localnative-rs/Cargo.toml`; members inherit it with `version.workspace = true`.

`localnative_core` and `localnative_cli` are **published to crates.io**, which
like the app stores forbids reuse and downgrades — so this number must only
ever move forward.

`localnative_egui` is deliberately excluded and pinned at `0.1.0` until it
reaches parity with the Iced front-end.

## Which script bumps what

| Command | Touches |
|---|---|
| `script/set-version <v>` | Rust workspace version only (one line) |
| `script/set-version-extension <v>` | Browser extension manifest only |
| `script/release-browser-extension <v>` | Extension manifest, then packages the upload zip |

No script writes a version to a platform it does not release. Each edits by
anchored match and fails loudly if the anchor is not unique — never by line
number.

## Why this policy exists

`set-version` used to write one number to the Rust crates, the extension
manifest, and the popup markup at once, by hardcoded line offset.

The consequences, all real:

- Nobody wanted to bump the published Rust crates just to ship an extension, so
  the extension **stayed at 0.5.1 from 2022 until the Chrome Web Store removed
  it** in the Manifest V2 deprecation. The MV3 code migration had landed; only
  the version had not.
- Running it to release the extension at 0.5.2 would have **walked
  `localnative_core` and `localnative_cli` backwards** from 0.7.0 — versions
  already published to crates.io.
- Its `popup.html` line-60 target had long since drifted onto a CSS custom
  property (`--danger`), so a release would have silently corrupted the dark
  theme. The popup now reads its version from the manifest at runtime via
  `chrome.runtime.getManifest()`, so no markup carries a version at all.

Coupling versions did not keep anything in sync. It stopped releases from
happening.
