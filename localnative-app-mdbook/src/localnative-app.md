# Local Native App
version: 0.2.0

A note/bookmark taking tool to save your notes in a local sqlite database, synced with other devices via [ssb](https://ssbc.github.io/secure-scuttlebutt/) [protocal](https://ssbc.github.io/scuttlebutt-protocol-guide/).

## Web extension UI
![Local Native web extension popup screenshot](/img/localnative-web-ext-popup.png)

## Search your notes
![Local Native patchbay screenshot](/img/localnative-ssb-patchbay.png)

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
web-ext run --verbose # firefox
```

#### Setup browser extension host binary
- use `cargo install localnative_cli`, and find the binary at `~/.cargo/bin/localnative-web-ext-host`

- or build from source, via [cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html)
```
cd localnative-rs
cargo build
```
- or download from [release archive](https://localnative.app/release.html)

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
  "path": "/home/USER/.cargo/bin/localnative-web-ext-host",
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
  "path": "/home/USER/.cargo/bin/localnative-web-ext-host",
  "type": "stdio",
  "allowed_origins": [
    // use this ID if you install from chrome web store,
    // or add/change to what the actual ID is if you "LOAD UNPACKED" from source.
    "chrome-extension://oclkmkeameccmgnajgogjlhdjeaconnb/"
  ]
}
```


#### Setup Node.js global binaries
currently the rust host binary calls some node.js binaries for ssb work, so

```
npm i -g localnative
```
the host may not know the global node bin path, so cd to where you node global bin directory, and
```
(sudo) ln -s localnative-ssb* /usr/local/bin/
```


#### Start a sbot server
The web extension send message to `localnative-web-ext-host` and writes new note to sqlite first, then response to display the search result, and finally try to sync with ssb. If sbot server is not running, the host will not sync with ssb at that time.

So use those softwares:

[Patchwork](https://github.com/ssbc/patchwork/releases) simpler user friendly UI.

[Patchbay](https://github.com/ssbc/patchbay/releases) more advanced, and show the `raw` json of each message for easiler debugging, combined with custom filter and good enough free text search.

#### Database

If above things are correctly setup, `localnative.sqlite3` database file is created at the directory `~/.ssb/localnative.sqlite3` the first time you click the web ext popup.

hint to see what `localnative-web-ext-host` went wrong:
```
RUST_BACKTRACE=1 chromium-browser
RUST_BACKTRACE=1 web-ext run --verbose
```

You can use [DB Browser for SQLite](http://sqlitebrowser.org/) to explore the database, and adding other device's ssb public key as `authoer` in the `ssb` table.


