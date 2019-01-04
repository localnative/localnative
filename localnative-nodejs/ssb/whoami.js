#!/usr/bin/env node

module.exports = function(){
  var ssbKeys = require('ssb-keys')
  const os = require('os')
  const homedir = os.homedir()
  var keys = ssbKeys.loadOrCreateSync(homedir + '/.ssb/secret')
  console.log(keys.id)
}
