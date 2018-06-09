# Local Native App
version: 0.0.1

A note/bookmark taking tool using a local sqlite database. The browser extension is built with native messaging.

## Source code
[git ssb](http://localhost:7718/%q13hLJchNXz/xZi9mjWVHyIbRnkr5VjF0Y6BfhrOV6Q=.sha256): `ssb://%q13hLJchNXz/xZi9mjWVHyIbRnkr5VjF0Y6BfhrOV6Q=.sha256`

## Quick start

### Browser extension

Below 3 componets must all exist and correctly setup for browser extension to work.

- Install browser extension

  - from browser extension site
    - [Firefox Add-on](https://addons.mozilla.org/addon/localnative/)
    - [Chrome Extension](https://chrome.google.com/webstore/detail/local-native/oclkmkeameccmgnajgogjlhdjeaconnb)

  - or from source at `localnative-browser-extension/app`

- Setup browser extension host binary
    - from source at `localnative-rs`

- Setup native messaging manifest to point to extension host binary

    - Copy manifest json template file `app.localnative.json` from `localnative-browser-extension/host` to your browser's specific manifest location
        - [Firefox](https://developer.mozilla.org/en-US/Add-ons/WebExtensions/Native_manifests#Manifest_location)
        - [Chrome](https://developer.chrome.com/extensions/nativeMessaging#native-messaging-host-location)
    - Change the host `path` in `app.localnative.json` to where `localnative-web-ext-host` binary is from previous step
