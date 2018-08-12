# Local Native App
version: 0.1.1

A note/bookmark taking tool using a local sqlite database. The browser extension is built with native messaging.

## (not so) Quick start

### Browser extension

Below 3 componets must all exist and correctly setup for browser extension to work.

- Install browser extension

  - from browser extension site
    - [Firefox Add-on](https://addons.mozilla.org/addon/localnative/)
    - [Chrome Extension](https://chrome.google.com/webstore/detail/local-native/oclkmkeameccmgnajgogjlhdjeaconnb)

  - or from source at `localnative-browser-extension/app`

- Setup browser extension host binary
    - use `cargo install localnative_cli`, and find the binary at `.cargo/bin/localnative-web-ext-host`

    - download from [release archive](https://localnative.app/release.html)

        Currently built on Ubuntu Ubuntu 18.04.1 LTS.

- Setup native messaging manifest to point to extension host binary

    - Copy manifest json template file `app.localnative.json` from `localnative-browser-extension/host` to your browser's specific manifest location
        - [Firefox](https://developer.mozilla.org/en-US/Add-ons/WebExtensions/Native_manifests#Manifest_location)
        - [Chrome](https://developer.chrome.com/extensions/nativeMessaging#native-messaging-host-location)
    - Change the host `path` in `app.localnative.json` to where `localnative-web-ext-host` binary is from previous step

### Database

`localnative.sqlite3` database file is created at the directory where `localnative-web-ext-host` exists.

You can use [DB Browser for SQLite](http://sqlitebrowser.org/) to explore the database.
