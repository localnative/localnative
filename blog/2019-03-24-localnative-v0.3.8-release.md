---
title: "Local Native v0.3.8 发布"
author: Yi Wang
authorURL: https://www.yi-wang.me
authorImageURL: https://secure.gravatar.com/avatar/1484b2bde1c0027dab9b135a1e051b3b?s=180&d=identicon

categories: ["公告 Announcement"]
tags: ["all", "localnative", "release"]
---
![localnative-desktop-v0.3.8.png](/img/localnative-desktop-v0.3.8.png)
## What is new?
- This [Local Native](https://localnative.app/) release marks as a personal milestone to satisfy my day-to-day use case for web bookmarking and note-taking with title, URL, tags, and description.
- Search, create, read, delete and pagination are implemented for [desktop](https://gitlab.com/yiwang/localnative-release/tree/master/v0.3.8/) (gnu/linux, mac), browser extension ([firefox](https://addons.mozilla.org/en-US/firefox/addon/localnative/), [chrome/brave](https://chrome.google.com/webstore/detail/local-native/oclkmkeameccmgnajgogjlhdjeaconnb)) and mobile ([ios](https://itunes.apple.com/us/app/local-native/id1443968309), [android](https://play.google.com/store/apps/details?id=app.localnative)).
- On desktop, time-series charts are shown to allow filtering on time range, and tag cloud is visualized.
<!--truncate-->

## About sync with other devices
Maybe the biggest drawback of not using a centralized service is losing the "convenience" of everything synced on every device.

- Currently, the Local Native desktop implemented wired sync based on files via exchange and appending items from one to another and vice versa.
- Also, the Local Native desktop implemented `ssb` sync by setting up target keys.
- Wireless via [bluetooth](https://gitlab.com/staltz/manyverse) seems [achievable](https://github.com/googlesamples/android-BluetoothChat) (on android).
- [safe network](https://safenetwork.tech)?

## Known quirks could be improved
There is definitely plenty of room for improvements, some known ones among many others:
- No windows desktop build because of an upstream [issue](https://github.com/neon-bindings/neon/issues/357).
- On iOS, the layout of the tags is not optimal as it does not scroll, and each tag is the same width regardless of character length.
- Crash when adding note on Nexus 5X. (on Pixel 2 and Nexus 6 is fine)
- Mac app store build has a sandbox issue, so it is not quite usable. ([DMG](https://gitlab.com/yiwang/localnative-release/tree/master/v0.3.8/mac) works)
- ...

However, given this release now satisfied my day-to-day bookmarking needs, my limited spear time will be mostly on trying more aspirational things.

## Conclusion
This serves as proof that one has a choice to not use a centralized service to manage web bookmarking and note-taking.
