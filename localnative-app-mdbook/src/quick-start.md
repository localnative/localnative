# Quick Start

Install and run the desktop application to create below directory and files.

Only install browser extension alone without the web extension host binary below will not work.

## Directory structure
Desktop
```
LocalNative # folder at user home directory
├── bin
│   ├── localnative-nodejs-X.Y.Z # nodejs cli for ssb sync feature
│   └── localnative-web-ext-host-X.Y.Z{-mac,-gnu-linux,.exe} # web extension host (rust binary)
└── localnative.sqlite3 # user's database
```
## Sync

### via attach file
You can copy the SQLite database file from mobile device to desktop device via [File Sharing](https://support.apple.com/en-us/HT201301) and vice versa.

Local Native desktop implemented wired sync based on files via exchange and appending items from one to another and vice versa.

### via ssb
On desktop, with proper setup, you can replicate your data with other devices via [ssb](https://www.scuttlebutt.nz) [protocal](https://ssbc.github.io/scuttlebutt-protocol-guide/).

By default, note is your private message in ssb, you can also publish public note to ssb if you explicitly choose so.

#### ssb pubkey setup
Add the pubkey (other peers you want to sync with) as `author` in `ssb` table, set other fields as `0`.

You can use [DB Browser for SQLite](http://sqlitebrowser.org/) to explore the database.

## Note
Windows currently does not have a standalone desktop app due to an upstream [issue](https://github.com/neon-bindings/neon/issues/357).

There is a Mac App Store version of Local Native but due to sandbox nature, it is NOT usable because node.js `fs.mkdirSync` call got permission denied to create the above directories (and most likely permission issue for those browser extension manifest file as well). I am curious if there is a way to do so.
