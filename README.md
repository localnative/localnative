# Local Native
[localnative.app](https://localnative.app)

A cross-platform tool to save and sync your notes in local SQLite database without going through any centralized service.

# Videos
[Local Native YouTube Channel](https://www.youtube.com/channel/UCO3qFIyK0eSmqvMknsslWRw)

## Articles
[Updates](https://localnative.app/blog)

[Why Local Native?](https://localnative.app/blog/2019/01/04/why-local-native)

# Sub-directories

external-libs: third-party dependencies vendored for the build

localnative-android: android app

localnative-browser-extension: browser extension app and host manifest template (includes `wasm-app/`)

localnative-docker: container build/run scripts (`Dockerfile`, `build.sh`, `run.sh`)

localnative-electron: desktop app (deprecated — superseded by localnative-tauri)

localnative-flutter: cross-platform mobile/desktop app via Flutter + flutter_rust_bridge (see `SETUP.md`)

localnative-ios: ios app

localnative-mac: native macOS app

localnative-neon: nodejs to rust bridge, used by localnative-electron

localnative-rs: rust code
- to build `localnative-web-ext-host` web extension host binary
- and `localnative_core`, the core native module
- and `localnative_iced`, the Rust GUI with iced framework

localnative-sql: sql snippet

localnative-tauri: next-generation desktop app (Tauri + Svelte frontend)

script: release and version-bump scripts

# Bounty
[bountysource](https://www.bountysource.com/teams/localnative-bounty/issues)

# License
[AGPL-3.0](https://www.gnu.org/licenses/agpl-3.0.en.html)

## Screenshot

### Mobile Application
![mobile](http://localnative.app/img/localnative-mobile-android-qr.png)

### Desktop Application
![Local Native desktop application](https://localnative.app/img/localnative-0.4.0-desktop-sync.png)

### Browser Extension
![Local Native web extension popup screenshot](https://localnative.app/img/localnative-web-ext-popup.png)

## Support
<a href="https://opencollective.com/localnative/donate" target="_blank">
  <img src="https://opencollective.com/localnative/donate/button@2x.png?color=blue" width=300 />
</a>
