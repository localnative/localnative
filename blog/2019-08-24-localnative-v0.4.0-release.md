---
title: Local Native v0.4.0 发布
author: Yi Wang
authorURL: http://twitter.com/yi_wang
authorImageURL: https://secure.gravatar.com/avatar/1484b2bde1c0027dab9b135a1e051b3b?s=180&d=identicon

categories: ["公告 Announcement"]
tags: ["all", "localnative", "release"]
---

As hinted in the [previous post](https://localnative.app/blog/2019/07/10/add-uuid4-and-tarpc-in-localnative/), this v0.4.0 release is a MVP to show sync between different devices using slightly [patched](https://github.com/yiwang/tarpc/tree/ln-v0.4.0) [tarpc](https://gitter.im/tarpc/Lobby?at=5d44e9e4ff59f961b4055c07) .
Currently sync works between desktop and desktop, desktop and mobile when they are on the same LAN/WiFi.

The schema of `note` table is updated to have an extra `uuid` column to uniquely identify and dedup records for sync. At the end of sync operation, both devices will end up with same set of `uuid` by appending the diff from the other device.

Mobile device can scan a QR code encoding the server address and port to connect to the server to start sync.

iOS sync screenshot:

<img src="/img/localnative-0.4.0-ios-sync.png" width="300" />

<!--truncate-->

Desktop server screenshot:
![localnative-0.4.0-desktop-sync.png](/img/localnative-0.4.0-desktop-sync.png)

This release also hides ssb sync, public and image screenshot from UI to keep things focused.
