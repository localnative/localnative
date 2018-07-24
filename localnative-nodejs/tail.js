'use strict'
var sodium     = require('chloride')
var ssbClient = require('ssb-client')
var pull = require('pull-stream')
var pb = require('private-box')
var ssbKeys = require('ssb-keys')

const os = require('os')
const homedir = os.homedir()
var keys = ssbKeys.loadOrCreateSync(homedir + '/.ssb/secret')

let id = process.argv[2]
let gt = Number(process.argv[3])

ssbClient(function (err, sbot) {
  if (err)
    throw err
    pull(
      sbot.createUserStream({id: id
        , gt: gt
      }),
      pull.collect(function (err, msgs){
        if (err) throw err
        msgs.forEach(function(msg){
          if(typeof msg.value.content == 'string'){
            var decoded = ssbKeys.unbox(msg.value.content, keys)
            // filter localnative
            if (decoded && decoded.type && decoded.type === 'post'
            && decoded.localnative ){
              var out = {
                note_title: decoded.localnative.note.title,
                note_url: decoded.localnative.note.url,
                note_tags: decoded.localnative.note.tags,
                author: msg.value.author,
                ts: msg.value.timestamp,
                key: msg.key,
                prev: msg.value.previous,
                author: msg.value.author,
                seq: msg.value.sequence
              }
              console.error(msg)
              console.log(JSON.stringify(out))
              // only output 1 item
              process.exit(0)
            }
          }
        })

      })
    )
  sbot.close()
})

