---
title: "Why Local Native?"
author: Yi Wang
authorURL: https://www.yi-wang.me
authorImageURL: https://secure.gravatar.com/avatar/1484b2bde1c0027dab9b135a1e051b3b?s=180&d=identicon

categories: ["设计 Design"]
tags: ["all", "why", "localnative"]
---

It seems applications today are "Cloud Native" by default. For end user, it becomes expected for a "meaningful" application to have all its data available on all the devices all the time. However this assumed convenience comes with drawbacks:

- performance for certain feature
<!--truncate-->
Network is still slow, hence caching and syncing cache with remote datastore is a normal practice to boost user experience at the cost of draining device battery.

*What if things are stored locally natively?*

When I was implementing the type ahead search for [Local Native App](https://localnative.app), a brute force database query upon every keystroke just worked. Scaling issue seems goes away if you do not need to deal with network latency and multiple users competing with each other for CPU. I start to appreciate how fast the hardware becomes today.

- your data is on someone else's server

Enough said about privacy (freedom) concerns.

I used to have tens of social media profiles out of curiosity to checkout "what's on the internet". Util time itself becomes increasingly limited resource, and I figured most of the time I just need a bookmarking tool for the internet, so I can go back to something quickly, and maybe also run some SQL against those url and tags myself.

*What if things are stored locally natively?*

Without a traditional remote server, syncing across devices does become a challenge, but opportunity may lie here as well. Local Native App currently is able to sync via [ssb](https://www.scuttlebutt.nz/), with some [serverless](https://www.sqlite.org/serverless.html) setup first.

Toolchains today seem are all for centralized services, but 15 years ago where are those toolchains for modern mobile and web apps?

<iframe width="800" height="450" src="https://www.youtube-nocookie.com/embed/3dhB5gTtXNM" frameborder="0" allow="accelerometer; autoplay; encrypted-media; gyroscope; picture-in-picture" allowfullscreen></iframe>
