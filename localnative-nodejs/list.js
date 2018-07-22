'use strict'
var sodium     = require('chloride')
var ssbClient = require('ssb-client')
var pull = require('pull-stream')
var pb = require('private-box')
var ssbKeys = require('ssb-keys')

const os = require('os')
const homedir = os.homedir()
var keys = ssbKeys.loadOrCreateSync(homedir + '/.ssb/secret')

const fs = require('fs')
var pubkeys = JSON.parse(fs.readFileSync(homedir + '/.ssb/localnative-pub-keys'))

ssbClient(function (err, sbot) {
  if (err)
    throw err
  pubkeys.forEach(function(id){
    pull(
      sbot.createUserStream({id: keys.id}),
      pull.collect(function (err, msgs){
        if (err) throw err
        msgs.forEach(function(msg){
          if(typeof msg.value.content == 'string'){
            var decoded = ssbKeys.unbox(msg.value.content, keys)
            // filter localnative
            if (decoded && decoded.type && decoded.type === 'post'
            && decoded.localnative ){
              decoded.key = msg.key
              decoded.prev = msg.value.previous
              decoded.author = msg.value.author
              console.log(decoded)
            }
          }
        })
      })
    )
  })
  sbot.close()
})

