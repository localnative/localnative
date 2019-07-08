# Developer Setup
Below componets must all exist and correctly setup.

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
cd localnative-rs
cargo build
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
the host may not know the global node bin path, if so `cd` to where you node global bin directory, and
```
(sudo) ln -s localnative-ssb* /usr/local/bin/
```


#### Start a sbot server
The web extension send message to `localnative-web-ext-host` and writes new note to sqlite first, upon response to display the search result, and finally try to sync with ssb.

If sbot server is not running, the host will just error out, next requet will still be fine because each message is independent from each other.

You can use those softwares:

[Patchwork](https://github.com/ssbc/patchwork/releases): simpler user friendly UI.

[Patchbay](https://github.com/ssbc/patchbay/releases): more advanced, and show the `raw` json of each message for easiler debugging, combined with custom filter and good enough free text search.

#### Database

If above things are correctly setup, `localnative.sqlite3` database file is created at the directory `~/.ssb/localnative.sqlite3` the first time you click the web ext popup.

hint to see what `localnative-web-ext-host` went wrong:
```
RUST_BACKTRACE=1 chromium-browser
RUST_BACKTRACE=1 web-ext run --verbose
```

You can use [DB Browser for SQLite](http://sqlitebrowser.org/) to explore the database, and adding other device's ssb public key as `author` in the `ssb` table.
