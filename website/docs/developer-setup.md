---
id: developer-setup
title: Developer Setup
---
## Web Browser Extension

#### Install browser extension
- from browser extension site
  - [Firefox Add-on](https://addons.mozilla.org/addon/localnative/)
  - [Chrome Extension](https://chrome.google.com/webstore/detail/local-native/oclkmkeameccmgnajgogjlhdjeaconnb)

- or from source
```
git clone https://github.com/localnative/localnative.git
cd localnative-browser-extension/app
npm i -g web-ext
web-ext run --verbose # firefox
```

#### Setup browser extension host binary
- Download and run the desktop application from the [release archive](https://github.com/localnative/localnative/releases), which installs the browser host.
- Or install from a source checkout with Cargo, then install the Chrome and Firefox manifests:
```
cargo install --path localnative-rs/localnative_cli --bin localnative --bin localnative-web-ext-host
localnative host install
```
- To use an unpacked extension with a different Chrome ID, repeat its origin with `--allowed-origin`:
```
localnative host install --allowed-origin chrome-extension://YOUR_EXTENSION_ID/
```

To build and install from a source checkout instead:
```
cd localnative-rs
cargo build -p localnative_cli --bin localnative --bin localnative-web-ext-host
./target/debug/localnative host install
```

The installer copies the unified executable to the permanent
`localnative-web-ext-host` name and writes manifests with that absolute path.
This alias is required because browser native-messaging manifests cannot add a
`host` command argument.

#### Manually configure a native messaging manifest
The automatic installer is recommended. To configure a browser by hand instead:
- Copy manifest json template file `app.localnative.json` from `localnative-browser-extension/host` to your browser's specific manifest location
    - [Firefox](https://developer.mozilla.org/en-US/Add-ons/WebExtensions/Native_manifests#Manifest_location)
    - [Chrome](https://developer.chrome.com/extensions/nativeMessaging#native-messaging-host-location)
- Change the host `path` to the installed `localnative-web-ext-host` executable

##### Firefox example manifest file
`~/.mozilla/native-messaging-hosts/app.localnative.json`
```
{
  "name": "app.localnative",
  "description": "Local Native Host",
  "path": "PREFIX/localnative/localnative-rs/target/debug/localnative-web-ext-host",
  "type": "stdio",
  "allowed_extensions": [
    "localnative@example.org"
  ]
}
```

##### Chromium example manifest file
` ~/.config/chromium/NativeMessagingHosts/app.localnative.json`
```
{
  "name": "app.localnative",
  "description": "Local Native Host",
  "path": "PREFIX/localnative/localnative-rs/target/debug/localnative-web-ext-host",
  "type": "stdio",
  "allowed_origins": [
    // use this ID if you install from chrome web store,
    // or add/change to what the actual ID is if you "LOAD UNPACKED" from source.
    "chrome-extension://oclkmkeameccmgnajgogjlhdjeaconnb/"
  ]
}
```

#### Database

If above things are correctly setup, `localnative.sqlite3` database file is created at `~/LocalNative/localnative.sqlite3` the first time you click the web ext popup. The `LOCALNATIVE_DB` environment variable (or the unified CLI's `--db` flag) overrides the location.

hint to see what `localnative-web-ext-host` went wrong:
```
RUST_BACKTRACE=1 chromium-browser
RUST_BACKTRACE=1 web-ext run --verbose
```

You can use [DB Browser for SQLite](http://sqlitebrowser.org/) to explore the database.

##  Desktop

#### Tauri (Svelte frontend) — recommended desktop target
```
cd localnative-tauri
yarn install
yarn dev          # vite dev
yarn build        # vite build (production)
yarn check        # svelte-check type check
```
Tauri uses `yarn` (`yarn.lock` is the committed lockfile). Linux desktop builds
require GTK 3 and WebKitGTK 4.1; the Ubuntu CI installs:
```
sudo apt-get install -y libgtk-3-dev libwebkit2gtk-4.1-dev libayatana-appindicator3-dev librsvg2-dev patchelf libxdo-dev
```

## Mobile
#### Android

The Gradle build compiles the Rust core itself (`cargo ndk` through the
`buildRustCore` task in `app/build.gradle`); install `cargo-ndk` and the NDK
first. `cargo xtask ndkbd` does the same by hand.

#### iOS

`script/build-ios.sh` builds the Rust core for device and simulator as
`localnative-ios/LocalNativeCore.xcframework`, which the Xcode project links.
Run it after any change to `localnative_core`.

## Import / Export

The unified `localnative` CLI moves notes in and out of the local SQLite
database. It uses `~/LocalNative/localnative.sqlite3` by default; `--db` or
`LOCALNATIVE_DB` selects another file.

#### Import from another service
Supported formats: `pocket` (HTML export), `omnivore` (JSON), `raindrop` (CSV),
`plinky` (JSON). Notes are de-duplicated by URL, so re-running an import is safe.
```
cd localnative-rs
cargo run -p localnative_cli -- import --format pocket path/to/export.html
```

#### Export to Markdown
Exports each note as an individual `.md` file with YAML frontmatter. An optional
search query filters which notes are exported.
```
cd localnative-rs
cargo run -p localnative_cli -- export --output ./exported-notes
cargo run -p localnative_cli -- export --output ./exported-notes --query rust
```

## Script
There are scripts to bump versions and package releases — see
`docs/VERSIONING.md` for which script touches which platform.
```
script
├── set-version            # Rust workspace version
├── set-version-extension  # browser extension version
├── release-browser-extension
├── build_linux.sh
└── build-ios.sh           # Rust core → LocalNativeCore.xcframework
```
