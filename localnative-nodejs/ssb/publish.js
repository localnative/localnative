/*
    Local Native
    Copyright (C) 2018-2019  Yi Wang

    This program is free software: you can redistribute it and/or modify
    it under the terms of the GNU Affero General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    This program is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU Affero General Public License for more details.

    You should have received a copy of the GNU Affero General Public License
    along with this program.  If not, see <https://www.gnu.org/licenses/>.
*/

var ssbClient = require('ssb-client')
var ssbKeys = require('ssb-keys')

const os = require('os')
const homedir = os.homedir()
const fs = require('fs')

module.exports = function(pubkeys){
  var pubkeys = JSON.parse(pubkeys)
  let keys = ssbKeys.loadOrCreateSync(homedir + '/.ssb/secret')
  let note = JSON.parse(fs.readFileSync("/dev/stdin", "utf-8"))

  //process.exit()

  ssbClient(function (err, sbot) {
    if (err)
      throw err

    let is_public = note.is_public
    // sbot is now ready. when done:
    sbot.publish(
      // message:
      mkMsg(note),
      // cb:
      cb(is_public)
    )
    sbot.close()
  })

  function cb(is_public){
    return function(err, msg) {
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
        seq: msg.value.sequence,
        is_public: is_public
      }
      console.log(JSON.stringify(out))
    }
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
}
