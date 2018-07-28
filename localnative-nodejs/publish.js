#!/usr/bin/env node

var ssbClient = require('ssb-client')
var ssbKeys = require('ssb-keys')

const os = require('os')
const homedir = os.homedir()
var keys = ssbKeys.loadOrCreateSync(homedir + '/.ssb/secret')

const fs = require('fs')
var pubkeys = JSON.parse(fs.readFileSync(homedir + '/.ssb/localnative-pub-keys'))

var note = JSON.parse(process.argv[2])

console.error(note)
console.error(mkMsg(note))

//process.exit()

ssbClient(function (err, sbot) {
  if (err)
    throw err

  // sbot is now ready. when done:
  sbot.private.publish(
    // message:
    mkMsg(note),
    // recipient PKs:
    pubkeys,
    // cb:
    function (err, msg) {
      // privateMsg.value.content is
      // an encrypted string
      if (err) throw err
      console.error(msg)
      var out = {
        note_title: note.title,
        note_url: note.url,
        note_tags: note.tags,
        author: msg.value.author,
        ts: msg.value.timestamp,
        key: msg.key,
        prev: msg.value.previous,
        author: msg.value.author,
        seq: msg.value.sequence
      }
      console.log(JSON.stringify(out))
    }
  )

  sbot.close()
})

function mkMsg(note){
  var tags = note.tags.split(',')

  var mentions = tags.map(function(t){
    return {link: `#${t}`}
  })

  var tagsText = tags.map(function(t){
    return `#${t}`
  }).join(' ')

  return {
    type: 'post',
    text: `${tagsText} ${note.title}
[${note.url}](${note.url})
`,
    mentions: mentions,
    recps: pubkeys,
    localnative: {
      note: note
    }
  }
}
