---
title: Add uuid4 and tarpc in Local Native
author: Yi Wang
authorURL: https://www.yi-wang.me
authorImageURL: https://secure.gravatar.com/avatar/1484b2bde1c0027dab9b135a1e051b3b?s=180&d=identicon

categories: ["设计 Design"]
tags: ["all", "localnative", "uuid", "tarpc", "grpc"]
---

## uuid4
I have not cut a release yet, but the work-in-progress [Local Native](https://localnative.app) v0.4.0 introduced a new `uuid4` field into `note` table.

Why?

This serves as a unique key for each record, syncing between different devices could leverage this new column. Actual code for syncing is still yet to be written. The idea is uuid4 could be used as a record identifier to compare with records with other devices.

I considered using key based on actual content, but decided that's too much and preferred true randomness.

<!--truncate-->

## Upgrade schema
To migrate v0.3.x schema to v0.4.0, an upgrade sub module is also written to create a `meta` table with `version` and `is_upgrading` value, so far on all my devices it worked. It is truly written the Rust code once, and run it everywhere, otherwise how many lines of Swift, Java or JavaScript need to be written!

This schema change might already break the ssb sync code.

## Back to sync
I tried to tackle the syncing problem [before](@/2019-03-localnative-v0.3.8-release.md#about-sync-with-other-devices). The wired sync is not convenient. Sync via ssb the keys setup steps seems cumbersome (Yes, some UI could be put on top of it to make it feel better), but there are 3 real issues I see it is not fit to at least my desire:
- private data is going out wide to other peers, (Yes, can be encrypted, but see below)
- ssb private key loss or theft could expose private data
- performance seems not great, it takes hours to sync my last 10+ years notes

I decided sync over LAN or WiFi when devices are on the same local network with a different protocol should be my next bet.

So what protocol?

## tarpc

I started higher on the stack, even looked at [GraphQL](https://github.com/graphql-rust/juniper), then just try to spin up [HTTP server](https://hyper.rs) in rust, then hit some build error on Android for `openssl`. The dead body of `localnative_sync` is the experiment.

Maybe I don't need full HTTP(s) server but just [RPC](https://github.com/grpc/grpc)?

[Then](https://medium.com/@KevinHoffman/streaming-grpc-with-rust-d978fece5ef6) [looked](https://github.com/stepancheg/grpc-rust) [at](https://github.com/pingcap/grpc-rs) [gRPC](https://github.com/stepancheg/grpc-rust), seems also too heavy, I have not tried but it seems could also be challenging to compile natively on mobile device. Then come [to](https://www.reddit.com/r/rust/comments/7xyu88/tarpc_vs_capnprotorpcrust/) [tarpc](https://github.com/google/tarpc). I can get it running on iOS!

Promising.

So here it is, the code now have tarpc as a dependency.

## Random thoughts

Like any FLOSS libraries, those supported by large players might have a better chance of being actively maintained and being aligned with the "trend".

I recalled [Bytemaster](https://bytemaster.github.io/) once had chosen [Wren](https://github.com/wren-lang/wren) but then [switched](https://steemit.com/eos/@dantheman/web-assembly-on-eos-50-000-transfers-per-second) to [WebAssembly](https://eosio.stackexchange.com/questions/1003/why-wasm-instead-of-custom-virtual-machine) [for](https://medium.com/@eosforce/the-blockchain-industry-will-have-the-first-technical-standard-debate-eos-vm-take-the-lead-in-8619c5b84601) [EOS VM](https://github.com/EOSIO/eos-vm).

gRPC *is* HTTP/2. It is good for bidirectional communication between components in server farm(s). [gRPC-Web ](https://github.com/grpc/grpc-web) now allows web client directly connected to centralized server(s).

There are many JavaScript web frameworks, there are also [Node.js for](https://stackoverflow.com/questions/36632649/running-node-js-on-android) [mobile apps](https://code.janeasystems.com/nodejs-mobile). Rust should do the same.

[MANETs](https://staltz.com/a-plan-to-rescue-the-web-from-the-internet.html)

I also recall in the old days we 8 teenagers rushing into an internet cafe, ironically it's not connected to internet but just 8 PC connected to a router. Then the group just playing StarCraft for hours, no cloud subscription, no need for earphone but directly shout to your teammates, and you can also shamelessly perform physical distraction to your enemy commanders for the win.

I miss those days.
