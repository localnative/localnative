---
title: "Local Native v0.4.1 发布, without iOS, yet"
author: Yi Wang
authorURL: https://www.yi-wang.me
authorImageURL: https://secure.gravatar.com/avatar/1484b2bde1c0027dab9b135a1e051b3b?s=180&d=identicon

categories: ["公告 Announcement"]
tags: ["all", "localnative", "release"]
---

## prelude
It has been a few months since last [Local Native](https://localnative.app) v0.4.0 release, which introduced (wireless) syncing via tarpc, database schema change and more. That is decent amount of internal data structure change and observable new features. Everything was well and I was not worrying about rushing out more features very soon, until ..

## the bug.
In contrast, this v0.4.1 release is *only* motivated by fixing the *one* observed `bug`:

the touch input keyboard for the share extension's text fields (tag and description fields etc..) does not show up in iOS, after upgrading to newer version (iOS 13.2.2 as of writing). This prevents user to input tag or description, or change title or url for the note.

Saving note still works, but what's the fun if I can not add tags to a note?

Technically This is a new [feature](http://www.catb.org/jargon/html/B/bug.html) only in newer version of iOS. How hard could be

## the fix?
<!--truncate-->

It turns out [just](https://stackoverflow.com/questions/259819/how-do-i-show-the-keyboard-by-default-in-uitextview) [one liner](https://gitlab.com/localnative/localnative/commit/372e95697537382aabcbf5e53bac2702d6e068ab#4c2bea632ac1c24bf5ccd5e0058049f698310fbe_92_92) to not show the keyboard by default. Then keyboard will show up when user tap the input box.

However ironically now I get all other v0.4.1 release targets (android, web-ext, appimage, dmg) built and release, but still without iOS. It works in local build but crashes [only in ‎TestFlight](https://stackoverflow.com/questions/28570444/app-crashes-only-on-testflight-build).

I am still debugging this crash.

Apple has released macOS Catalina, upgraded iOS and Xcode. Multiple (minor) versions with fixes are coming frequently in the past month.

I am still debugging this crash only in ‎TestFlight.

Is this a sign that I may just *need* to switch to SwiftUI?

Not only Apple, everything is moving at

## the speed of light.

It turns out this release is just trying to catch up with *the right* dependency upgrades:
- tarpc using more recent [code](https://github.com/yiwang/tarpc/commits/ln-v0.4.1) from upstream
  - with declaring service interface using [trait syntax](https://gitlab.com/localnative/localnative/blob/v0.4.1-web-ext/localnative-rs/localnative_core/src/rpc.rs#L21)
  - with upgraded `tokio` and `futures-preview`
- electron upgraded to 7.1.1, may still suffer [this](https://github.com/electron/electron/issues/17972)
  - neon-cli 0.3.3 linker issue [on mac](https://github.com/neon-bindings/neon/issues/455), need to pin to 0.3.1
  - crossfilter2 1.5.0 require issue in electron, pinned to 1.4.8
  - electron-builder can not [create dmg](https://github.com/electron-userland/electron-builder/issues/3990) on Catalina
    - create-img does not work with [binary plist](https://github.com/sindresorhus/create-dmg/issues/35)

### 树欲静而风不止
逆水行舟，不进则退。
