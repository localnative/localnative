---
title: "Local Native 2021 Summary"
author: Cupnfish
authorURL: https://github.com/Cupnfish
authorImageURL: https://avatars.githubusercontent.com/u/40173605?v=4

categories: ["公告 Announcement"]
tags: ["summary", "2021", "localnative"]
---


![2022](/img/unsplash/crazy-nana-BFBAmGePnpU-unsplash.jpg)


The year 2021 has come and gone, and [Local Native](https://localnative.app) project has made solid progress in the past year:

- The biggest achievement is the release of our cross-platform desktop based on the `iced` framework. 

- We also updated the `localnative_core` dependency.

- We have also released an [iced GUI tutorial](https://localnative.app/docs/gui-iced/tutorial0) based on this codebase.

- We learned early PR commits were not well conformed, which caused some difficulties in organizing them when writing the summary.

For more details on the changes, please see below:

## New Features

- [#](https://gitlab.com/localnative/localnative/-/tree/master/localnative-rs/localnative_iced) Released a cross-platform desktop version based on `iced`, supporting Linux, macOS, and Windows platforms
- [#](https://gitlab.com/localnative/localnative/-/tree/master/localnative-rs/xtask) Added `cargo xtask ndkbd` to simplify compiling `localnative_core` to Android platform

## Fixes

- [#](https://gitlab.com/localnative/localnative/-/commit/fc471bb501eb1d8c67282ecd7bb1e8d0243edd05) Fixed app crash issue on Android 11 and newer.
- [#](https://gitlab.com/localnative/localnative/-/commit/0082f369481da97500899ca5931ae2831362452c) Fixed the bug that prevents sharing note from browser for the Android platform.
- [#](https://gitlab.com/localnative/localnative/-/commit/827529d12e23261e7f1397d7d781c94ef3c49af4)  Fixed the problem that on android platform SQLite database can't be accessed correctly due to the creation of temporary files.

## Internal enhancement

- [#](https://gitlab.com/localnative/localnative/-/commit/1359e04dbfd90719ef10cf3cacd92aaefecf5ff6) Update `localnative_core` dependency version
- [#](https://gitlab.com/localnative/localnative/-/tree/master/localnative-rs/xtask) Added `xtask` project to provide cross-platform scripting
- [#](https://gitlab.com/localnative/localnative/-/commit/a44914e37c3366787b9f5839254363a164f41b73) Better error handling, refactoring of `localnative_core`