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

## ssb pubkey setup
Add the pubkey (other peers you want to sync with) as `author` in `ssb` table, set other fields as `0`.

You can use [DB Browser for SQLite](http://sqlitebrowser.org/) to explore the database.

## Note
Windows currently does not have a standalone desktop app due to an upstream [issue](https://github.com/neon-bindings/neon/issues/357).

There is a Mac App Store version of Local Native but due to sandbox nature, it is NOT usable because node.js `fs.mkdirSync` call got permission denied to create the above directories (and most likely permission issue for those browser extension manifest file as well). I am curious if there is a way to do so.
