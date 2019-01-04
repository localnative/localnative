#!/usr/bin/env node

var argv = process.argv.slice(2)

if (argv[0] == 'ssb-publish'){
  let pubkeys = argv[1]
  require('./ssb/publish')(pubkeys)
}

if (argv[0] == 'ssbify'){
  let html = argv[1]
  let title = argv[2]
  let url = argv[3]
  require('./ssb/ssbify')(html, title, url)
}

if (argv[0] == 'ssb-tail'){
  let id = argv[1]
  let gt = argv[2]
  require('./ssb/tail')(id, gt)
}

if (argv[0] == 'ssb-whoami'){
  require('./ssb/whoami')()
}
