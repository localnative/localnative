# Videos

## Local Native Desktop Demo on Mac - Core Functions for Note
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
  - Copy `other.sqlite3` back to iOS device via File Sharing and replace iOS device's `localnative.sqlite`. (sync is both ways so `other.sqlite3` also gets what is on desktop device).


## Local Native iOS Demo - Add a Note
<iframe width="560" height="315" src="https://www.youtube-nocookie.com/embed/3dhB5gTtXNM" frameborder="0" allow="accelerometer; autoplay; encrypted-media; gyroscope; picture-in-picture" allowfullscreen></iframe>

- Local Native share extension
  - Open Safari and go to a web site.
  - Open Local Native share extension and enter desired tags and description (you can actually also change title and url).
  - `Save` button will launch Local Native search UI and newly added note will show up.
- Local Native search
  - Click url link in any item from search result list will launch browser to that url's web page.
  - Touch upper left can go back to Local Native search UI from browser.
- File Sharing and DB Browser for SQLite
  - Copy `localnative.sqlite3` from mobile device to desktop.
  - Open DB Browser for SQLite to browse your notes (and then run SQL!).

