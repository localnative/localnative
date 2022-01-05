---
title: "2021 4th quarter summary"
author: Cupnfish
authorURL: https://github.com/Cupnfish
authorImageURL: https://avatars.githubusercontent.com/u/40173605?v=4

categories: ["公告 Announcement"]
tags: ["summary", "localnative"]
draft: true
---

The year 2021 has come and gone, and localnative has made some progress in the past year, the biggest progress being the release of our cross-platform desktop based on the `iced` framework. We also updated the `localnative_core` dependency.

We have also released a tutorial on this code base.

Of course we learned some things in the process, such as the fact that the early pr commits were not very strong, which caused some difficulties in organizing them when writing the summary.

For more details on the changes, please see below:

## New Features

- [#](https://gitlab.com/localnative/localnative/-/tree/master/localnative-rs/localnative_iced) Released a cross-platform desktop version based on `iced`, supporting linux, macos, and windows platforms
- [#](https://gitlab.com/localnative/localnative/-/tree/master/localnative-rs/xtask) Added `cargo xtask ndkbd` to simplify compiling `localnative_core` to Android platform

## Fixed

- [#](https://gitlab.com/localnative/localnative/-/commit/fc471bb501eb1d8c67282ecd7bb1e8d0243edd05) Fix Android 11 and newer edition app crashing issue.
- [#](https://gitlab.com/localnative/localnative/-/commit/0082f369481da97500899ca5931ae2831362452c) Fix the problem that the browser sharing feature does not work in Android platform.
- [#](https://gitlab.com/localnative/localnative/-/commit/827529d12e23261e7f1397d7d781c94ef3c49af4)  Fix the problem that the android platform sqlite can't be accessed normally caused by the creation of temporary files.

## Internal enhancement

- [#](https://gitlab.com/localnative/localnative/-/commit/1359e04dbfd90719ef10cf3cacd92aaefecf5ff6) Update `localnative_core` dependency version
- [#](https://gitlab.com/localnative/localnative/-/tree/master/localnative-rs/xtask) Added `xtask` project to provide cross-platform scripting
- [#](https://gitlab.com/localnative/localnative/-/commit/a44914e37c3366787b9f5839254363a164f41b73) Better error handling, refactoring of `localnative_core`