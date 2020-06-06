---
title: "Local Native iOS v0.4.1 发布, with SwiftUI"
author: Yi Wang
authorURL: http://twitter.com/yi_wang
authorImageURL: https://secure.gravatar.com/avatar/1484b2bde1c0027dab9b135a1e051b3b?s=180&d=identicon

categories: ["公告 Announcement"]
tags: ["all", "localnative", "release", "ios"]
---
<img src="/img/localnative-ios-v0.4.1-swiftui.png" width="300" />

## prelude
It has been another few months since last [Local Native](https://localnative.app) v0.4.1 release, which was not able to include iOS version due to [crashing](https://localnative.app/blog/2019/11/16/localnative-v0.4.1-release-without-ios#the-bug) in TestFlight. Now iOS v0.4.1 is finally released!

## the bug?
I still could not identify how the crash occurs by staring at the crash log.

## the fix.
It turns out the fix is to rewrite iOS code in SwiftUI.

Fortunately thanks to existing solutions for [data flow](https://medium.com/@azamsharp/the-complete-guide-to-state-management-in-swiftui-8759add64bcf), [search bar](https://stackoverflow.com/questions/56490963/how-to-display-a-search-bar-with-swiftui), [QR](https://www.hackingwithswift.com/books/ios-swiftui/generating-and-scaling-up-a-qr-code) [code](https://github.com/twostraws/CodeScanner), [hiding keyboard](https://stackoverflow.com/questions/56491386/how-to-hide-keyboard-when-using-swiftui), the rewrite is very pleasant and code becomes more concise by removing storyboard and view controllers.

One thing maybe a drawback from previous iOS version is that the bookmark note text is not [selectable](https://stackoverflow.com/questions/58005434/how-do-i-allow-text-selection-on-a-text-label-in-swiftui).
