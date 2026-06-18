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
git clone https://gitlab.com/localnative/localnative.git
cd localnative-browser-extension/app
npm i -g web-ext
web-ext run --verbose # firefox
```

#### Setup browser extension host binary
- Download and run the desktop applcation from [release archive](https://gitlab.com/localnative/localnative-release)

    this will create `~/LocalNative/bin` directory containing the host binary
- or use `cargo install localnative_cli`, and find the binary at `~/.cargo/bin/localnative-web-ext-host`

- or build from source, via [cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html)
```
cd localnative-rs/localnative_cli
cargo build
```
  If build succeed, the web extension host debug executable location is
```
localnative-rs/target/debug/localnative-web-ext-host
```

#### Setup native messaging manifest to point to extension host binary
- Copy manifest json template file `app.localnative.json` from `localnative-browser-extension/host` to your browser's specific manifest location
    - [Firefox](https://developer.mozilla.org/en-US/Add-ons/WebExtensions/Native_manifests#Manifest_location)
    - [Chrome](https://developer.chrome.com/extensions/nativeMessaging#native-messaging-host-location)
- Change the host `path` in `app.localnative.json` to where `localnative-web-ext-host` binary is from previous step

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

If above things are correctly setup, `localnative.sqlite3` database file is created at the directory `~/.ssb/localnative.sqlite3` the first time you click the web ext popup.

hint to see what `localnative-web-ext-host` went wrong:
```
RUST_BACKTRACE=1 chromium-browser
RUST_BACKTRACE=1 web-ext run --verbose
```

You can use [DB Browser for SQLite](http://sqlitebrowser.org/) to explore the database.

##  Desktop

#### Rust GUI using Iced
see tutorial

#### Tauri (Svelte frontend) — recommended desktop target
```
cd localnative-tauri
yarn install
yarn dev          # vite dev
yarn build        # vite build (production)
yarn check        # svelte-check type check
```
Tauri uses `yarn` (`yarn.lock` is the committed lockfile).

#### Electron - only Mac and GNU/Linux - deprecated
```
cd localnative-electron
npm i
npm run build
npm run dev
```

## Mobile
#### Android

#### iOS

#### Flutter (Android / iOS / macOS)
```
cd localnative-flutter
make setup        # install Rust targets + flutter_rust_bridge_codegen
make bridge       # regenerate Dart↔Rust bindings
make run-android  # or run-ios / run-macos
```
See `localnative-flutter/SETUP.md` for prerequisites (Flutter SDK, NDK, Xcode).

## Script
There are scripts to bump version and release
```
script
├── release-appimage
├── release-mac
├── release-web-ext-host
└── set-version
```
