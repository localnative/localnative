# Videos

## Local Native demo on mac
- Install Local Native browser extension to Chromium, showing desktop application is needed.
  - Run Local Native desktop application to setup web extension host binary.
  - Local Native browser extension works for Chromium: Add, search, pagination and delete.
  - Local Native desktop application: Add, search, pagination and delete.

- Firefox with Local Native browser extension.
- Brave with Local Native browser extension.

- Sync via attach file
  - Copy `localnative.sqlite3` from iOS device via File Sharing to `~/Document/other.sqlite3`.
  - Click `sync via attach file` button and open `other.sqlite3`.
  - Notice `sync-via-attah-done` in response, now `other.sqlite3` is synced with `~/LocalNative/localnative.sqlite3`.
  - Clear the search, note increased `count` in response, new items from `other.note` shows in search result.
  - Copy `other.sqlite3` back to iOS device via File Sharing and replace iOS device's `localnative.sqlite`.


