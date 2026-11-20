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

`localnative_core`, `localnative_cli`, `localnative_hostinstall` and `xtask`
ship together and carry one version, declared once in `[workspace.package]` in
`localnative-rs/Cargo.toml`; members inherit it with `version.workspace = true`.

The crates.io packages are `localnative_hostinstall`, `localnative_core`, and
`localnative_cli`, published in that dependency order. The registry forbids
reuse and downgrades, so this number must only ever move forward.

The npm and PyPI packages are both named `localnative` and use the same CLI
version, independently of the Tauri/browser/mobile releases. Their tags are
`cli-vX.Y.Z`. `script/set-version` also updates CLI path-dependency versions,
Rust/Tauri lockfiles and `packaging/npm/package.json`; PyPI reads the Cargo
version through `pyproject.toml`.

## Which script bumps what

| Command | Touches |
|---|---|
| `script/set-version <v>` | Rust workspace and npm CLI versions; PyPI inherits Cargo |
| `script/set-version-extension <v>` | Browser extension manifest only |
| `script/release-browser-extension <v>` | Extension manifest, then packages the upload zip |

No script writes a version to a platform it does not release. Text manifests
use anchored matches that fail if the anchor is not unique; npm's package manifest
is updated as structured JSON — never by line number.

## Why this policy exists

`set-version` used to write one number to the Rust crates, the extension
manifest, and the popup markup at once, by hardcoded line offset.

The consequences, all real:

- Nobody wanted to bump the published Rust crates just to ship an extension, so
  the extension **stayed at 0.5.1 from 2022 until the Chrome Web Store removed
  it** in the Manifest V2 deprecation. The MV3 code migration had landed; only
  the version had not.
- Running it to release the extension at 0.5.2 would have **walked
  `localnative_core` and `localnative_cli` backwards** from the workspace's
  0.7.0; crates.io releases had already stopped at 0.3.7.
- Its `popup.html` line-60 target had long since drifted onto a CSS custom
  property (`--danger`), so a release would have silently corrupted the dark
  theme. The popup now reads its version from the manifest at runtime via
  `chrome.runtime.getManifest()`, so no markup carries a version at all.

Coupling versions did not keep anything in sync. It stopped releases from
happening.
