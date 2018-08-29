#!/usr/bin/env node

var ssbClient = require('ssb-client')
var ssbKeys = require('ssb-keys')

const os = require('os')
const homedir = os.homedir()
var keys = ssbKeys.loadOrCreateSync(homedir + '/.ssb/secret')

const fs = require('fs')

let note = JSON.parse(fs.readFileSync("/dev/stdin", "utf-8"))
let pubkeys = JSON.parse(process.argv[2])

//process.exit()

ssbClient(function (err, sbot) {
  if (err)
    throw err

  // sbot is now ready. when done:
  sbot.publish(
    // message:
    mkMsg(note),
    // cb:
    cb
  )
  sbot.close()
})

function cb (err, msg) {
  // msg.value.content is
  // an encrypted string for private msg
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
    tagsText,
    "**" + note.title + "**", note.url,
    note.description, note.comments,
    note.annotations
  ].reduce(function(acc, i){
    if(i == ''){
      return acc
    }else{
      return acc + '\n' + i
    }
  }, note.created_at)

  delete note.rowid
  let is_public = note.is_public
  delete note.is_public

  var msg = {
    type: 'post',
    text: text,
    mentions: mentions,
    localnative: {
      note: note
    }
  }
  // private
  if(is_public){
    // mini branding for public msg
    msg.text = "![localnative.app](&b+Z2zC84VsUj41QsXnSVoIwkAtYrK0YoQwVajGaUC8A=.sha256) " + msg.text
  }else{
    msg.recps = pubkeys
  }
  return msg
}
