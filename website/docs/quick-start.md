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

### via attach file
You can copy the SQLite database file from mobile device to desktop device via [File Sharing](https://support.apple.com/en-us/HT201301) and vice versa.

Local Native desktop implemented wired sync based on files via exchange and appending items from one to another and vice versa.

### via LAN/WiFi
From release [v0.4.0](https://localnative.app/blog/2019/08/24/localnative-v0.4.0-release), you can sync between desktop and desktop, also desktop and mobile.

## Note
Windows currently does not have a standalone desktop app due to an upstream [issue](https://github.com/neon-bindings/neon/issues/357).

There is a Mac App Store version of Local Native but due to sandbox nature, it is NOT usable because node.js `fs.mkdirSync` call got permission denied to create the above directories (and most likely permission issue for those browser extension manifest file as well). I am curious if there is a way to do so.
