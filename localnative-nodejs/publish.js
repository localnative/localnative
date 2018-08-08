#!/usr/bin/env node

var ssbClient = require('ssb-client')
var ssbKeys = require('ssb-keys')

const os = require('os')
const homedir = os.homedir()
var keys = ssbKeys.loadOrCreateSync(homedir + '/.ssb/secret')

const fs = require('fs')

let note = JSON.parse(fs.readFileSync("/dev/stdin", "utf-8"))
let pubkeys = JSON.parse(process.argv[2])

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
      // msg.value.content is
      // an encrypted string
      if (err) throw err
      console.error(msg)
      var out = {
        note_title: note.title,
        note_url: note.url,
        note_tags: note.tags,
        note_description: note.description,
        note_comments: note.comments,
        note_annotations: note.annotations,
        note_created_at: note.created_at,

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

  if(tagsText == '#'){
    // no tags then no # sign show
    tagsText = ''
    mentions = {}
  }

  var text = [
    note.created_at,
    tagsText,
    "**" + note.title + "**", note.url,
    note.description, note.comments, note.annotations
  ].reduce(function(acc, i){
    if(i == ''){
      return acc
    }else{
      return acc + '\n' + i
    }
  },'')

  return {
    type: 'post',
    text: text,
    mentions: mentions,
    recps: pubkeys,
    localnative: {
      note: note
    }
  }
}
