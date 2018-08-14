# Local Native App
version: 0.1.3

A note/bookmark taking tool to save your notes in a local sqlite database, synced with other devices via [ssb](https://ssbc.github.io/secure-scuttlebutt/) [protocal](https://ssbc.github.io/scuttlebutt-protocol-guide/).

## (not so) Quick start
It is developer friendly now, and requires some fiddling.

Below componets must all exist and correctly setup.

#### Install browser extension
- from browser extension site
  - [Firefox Add-on](https://addons.mozilla.org/addon/localnative/)
  - [Chrome Extension](https://chrome.google.com/webstore/detail/local-native/oclkmkeameccmgnajgogjlhdjeaconnb)

- or from source
```
cd localnative-browser-extension/app
npm i -g web-ext
web-ext run # firefox
```

#### Setup browser extension host binary
- use `cargo install localnative_cli`, and find the binary at `~/.cargo/bin/localnative-web-ext-host`

- or build from source, via [cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html)
```
cd localnative-rs
cargo build
```
- or download from [release archive](https://localnative.app/release.html)

    currently only for GNU/Linux, built on Ubuntu 18.04.1 LTS.

#### Setup native messaging manifest to point to extension host binary
- Copy manifest json template file `app.localnative.json` from `localnative-browser-extension/host` to your browser's specific manifest location
    - [Firefox](https://developer.mozilla.org/en-US/Add-ons/WebExtensions/Native_manifests#Manifest_location)
    - [Chrome](https://developer.chrome.com/extensions/nativeMessaging#native-messaging-host-location)
- Change the host `path` in `app.localnative.json` to where `localnative-web-ext-host` binary is from previous step


#### Setup Node.js global binaries
currently the rust host binary calls some node.js binaries for ssb work, so

```
npm i -g localnative
```
the host may not know the global node bin path, so cd to where you node global bin directory, and
```
(sudo) ln -s localnative-ssb* /usr/local/bin/
```


#### Start ssb
Currently it assumes sbot server running, as ssb related error handling [needs improvement](/todo.html).

So use those softwares:

[Patchwork](https://github.com/ssbc/patchwork/releases) user friendly UI.

[Patchbay](https://github.com/ssbc/patchbay/releases) more advanced, and show the `raw` json of each message for easiler debugging.

#### Database

If above things are correctly setup, `localnative.sqlite3` database file is created at the directory `~/.ssb/localnative.sqlite3` the first time you click the web ext popup.

hint to see what `localnative-web-ext-host` went wrong:
```
RUST_BACKTRACE=1 chromium-browser
RUST_BACKTRACE=1 web-ext --verbose
```

You can use [DB Browser for SQLite](http://sqlitebrowser.org/) to explore the database, and adding other device's ssb public key in the `ssb` table.
