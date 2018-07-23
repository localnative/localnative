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
    function (err, privateMsg) {
      // privateMsg.value.content is
      // an encrypted string
      if (err) throw err
      console.log(privateMsg)
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
