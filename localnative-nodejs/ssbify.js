#!/usr/bin/env node
/*
Copyright (c) 2018 Kristoffer Ström <kristoffer@rymdkoloni.se>, dust <du5t@multiplexed.be>, Yi Wang

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
*/

var ssbClient = require('ssb-client')
var ssbifyString = require('./ssbify-bom.js')
const fs = require("fs")

if (!process.argv[2]) {
  console.error('usage: ssbify <string of valid HTML>')
  process.exit(1)
}

ssbClient(function (err, sbot) {
  if (err) throw err
  var html = process.argv[2]
  if (html === '-'){
    html = fs.readFileSync("/dev/stdin", "utf-8")
  }
  ssbifyString(sbot, html,
               { ignoreBrokenLinks: true,
                 title: process.argv[3] || 'untitled snippet',
                 url: process.argv[4] || ''
               },
               function (err, res) {
                 if (err) throw err
                 console.log(JSON.stringify(res))
                 process.exit(0)
               })
})
