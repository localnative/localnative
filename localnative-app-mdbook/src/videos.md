# Videos

## Local Native Desktop Demo on Mac - Core Functions for Note
<iframe width="560" height="315" src="https://www.youtube-nocookie.com/embed/DBsVscpSp6w" frameborder="0" allow="accelerometer; autoplay; encrypted-media; gyroscope; picture-in-picture" allowfullscreen></iframe>

- Install Local Native browser extension to Chromium.
  - Desktop application is needed to finish setup.
  - Run Local Native desktop application to finish setup (web extension host).
  - Local Native browser extension works for Chromium: Add some notes.

- Local Native desktop application: search, pagination and delete.

- Firefox with Local Native browser extension.
- Brave with Local Native browser extension.

- Sync via attach file
  - Copy `localnative.sqlite3` from iOS device via File Sharing to `~/Document/localnative.sqlite3`.
  - Click `sync via attach file` button and choose the above file.
  - Notice `sync-via-attah-done` in response, now the database from mobile device is synced with desktop's `~/LocalNative/localnative.sqlite3`.
  - Clear the search, note increased `count` in response, new items from mobile device database show in search result.
  - Copy `~/Document/localnative.sqlite3` back to iOS device via File Sharing and replace iOS device's `localnative.sqlite`. (sync is both ways so mobile device also gets what is on desktop device).



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

