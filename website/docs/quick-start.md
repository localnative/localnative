---
id: quick-start
title: Quick Start
---
Install and run the desktop application to create below directory and files.

Only install browser extension alone without the web extension host binary below will not work.

## Directory structure
Desktop
```
LocalNative # folder at user home directory
├── bin
│   └── localnative-web-ext-host-X.Y.Z{-mac,-gnu-linux,.exe} # web extension host (rust binary)
└── localnative.sqlite3 # user's database
```
You can use [DB Browser for SQLite](http://sqlitebrowser.org/) to explore the database.

## Sync

### via LAN/WiFi
Sync is peer-to-peer between paired devices on the local network; traffic is
encrypted and a device can only sync with devices it has paired with.

First contact between two devices pairs them: on the device that will act as
the server, start the server with pairing (the desktop Sync page's **Pair New
Device**, or `localnative-rpc-server --pair`), note the one-time pairing code
shown, then on the other device enter the server address and the code. The
code is valid for five minutes. After pairing, syncing needs only the
address — the pairing code field can stay empty.

### via attach file
You can copy the SQLite database file between devices (e.g. via [File
Sharing](https://support.apple.com/en-us/HT201301) on iOS) and merge with
`localnative-import-db`, or export a standalone copy with
`localnative-export-db`. Exports carry no device identity (sync keys and
paired peers are stripped), so a restored copy behaves as a new device.

## Note
Notes are stored in `~/LocalNative/localnative.sqlite3` on desktop
(`./Documents/localnative.sqlite3` inside the app container on iOS).
You can use [DB Browser for SQLite](http://sqlitebrowser.org/) to explore the database.
