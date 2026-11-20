---
id: release
title: Release
---

## Desktop installers

[https://github.com/localnative/localnative/releases](https://github.com/localnative/localnative/releases)

Tauri builds the desktop app and includes the unified CLI and permanent browser
host as sidecars. Desktop versions are independent of CLI package versions.

## CLI packages

The CLI package family is named `localnative` on npm and PyPI, and
`localnative_cli` on crates.io. From the `cli-v0.8.0` release onward, install
with any of:

```bash
cargo install localnative_cli --bin localnative --bin localnative-web-ext-host
python -m pip install localnative
npm install -g localnative
```

Each package installs `localnative` and `localnative-web-ext-host` for the
current platform. Run `localnative --help` for search, retrieval, sync,
import/export, MCP, and browser-host commands. To configure Chrome and Firefox:

```bash
localnative host install
```

The CLI's `cli-vX.Y.Z` tags are independent of application releases. The Rust
workspace, npm package, and PyPI package share one version; see
[Versioning](https://github.com/localnative/localnative/blob/main/docs/VERSIONING.md).

## Automated package verification

`.github/workflows/cli-release.yml` builds and tests:

- macOS arm64 and x64, Linux x64 and arm64, and Windows x64 binaries;
- Cargo and npm installation on each of those five platforms;
- one PyPI wheel per platform and a clean source-distribution installation;
- the assembled npm package containing all five platforms.

Every install check performs a real native-messaging insert, archive search,
full-note retrieval, and MCP session against a temporary database. It does not
read or modify the developer's library.

Rehearse a tagged release without publishing:

```bash
gh workflow run cli-release.yml -f tag=cli-v0.8.0 -f publish=false
```

## One-time registry setup

Trusted publishing uses this repository, `.github/workflows/cli-release.yml`,
and three GitHub environments: `crates-io`, `pypi`, and `npm`. No registry tokens
belong in the repository.

- **crates.io:** `localnative_core` and `localnative_cli` already exist. Publish
  `localnative_hostinstall` once manually with a maintainer's API token, then
  configure all three crates for trusted publishing:
  ```bash
  cargo publish --manifest-path localnative-rs/Cargo.toml -p localnative_hostinstall --locked
  ```
  The workflow skips exact crate versions that already exist, so this initial
  upload does not need a different version.
- **PyPI:** create the pending trusted publisher for `localnative`, then the
  project on first upload.
- **npm:** configure trusted publishing for the existing `localnative` package.

Publication is a separately approved release action. Preparing this workflow did
not publish any package.

## Publish a CLI release

1. Choose the next workspace version and update Rust/npm together:
   ```bash
   script/set-version 0.8.1
   ```
2. Review and commit the version change, then create and push a `cli-v0.8.1`
   tag. A tag push is the explicit publication trigger.
3. The workflow publishes only after every platform and package check passes.
   It publishes `localnative_hostinstall`, `localnative_core`, and
   `localnative_cli` in dependency order, then PyPI and npm.
4. To rerun a release explicitly, use:
   ```bash
   gh workflow run cli-release.yml -f tag=cli-v0.8.1 -f publish=true
   ```

Registry versions and uploaded files are immutable. Never reuse a version for
different source; rehearse first and fix failures before asking for publication.
