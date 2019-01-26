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

'use strict'
var sodium     = require('chloride')
var ssbClient = require('ssb-client')
var pull = require('pull-stream')
var pb = require('private-box')
var ssbKeys = require('ssb-keys')

const os = require('os')
const homedir = os.homedir()
var keys = ssbKeys.loadOrCreateSync(homedir + '/.ssb/secret')

module.exports = function(id, gtString){
  let gt = Number(gtString)

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
            // private
            if(typeof msg.value.content == 'string'){
              var decoded = ssbKeys.unbox(msg.value.content, keys)
              // filter localnative
              if (decoded && decoded.type && decoded.type === 'post'
              && decoded.localnative ){
                var out = {
                  note_title: decoded.localnative.note.title || '',
                  note_url: decoded.localnative.note.url || '',
                  note_tags: decoded.localnative.note.tags || '',
                  note_description: decoded.localnative.note.description || '',
                  note_comments: decoded.localnative.note.comments || '',
                  note_annotations: decoded.localnative.note.annotations || '',
                  note_created_at: decoded.localnative.note.created_at || '',

                  ts: msg.value.timestamp,
                  key: msg.key,
                  prev: msg.value.previous,
                  author: msg.value.author,
                  seq: msg.value.sequence,
                  is_public: false
                }
                console.error(msg)
                console.log(JSON.stringify(out))
                // only output 1 item
                process.exit(0)
              }
            } else {
              // public
              if (msg.value.content.type == 'post' && msg.value.content.localnative){
                let ln = msg.value.content.localnative
                let out = {
                  note_title: ln.note.title || '',
                  note_url: ln.note.url || '',
                  note_tags: ln.note.tags || '',
                  note_description: ln.note.description || '',
                  note_comments: ln.note.comments || '',
                  note_annotations: ln.note.annotations || '',
                  note_created_at: ln.note.created_at || '',

                  ts: msg.value.timestamp,
                  key: msg.key,
                  prev: msg.value.previous,
                  author: msg.value.author,
                  seq: msg.value.sequence,
                  is_public: true
                }
                console.error(out)
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
}
