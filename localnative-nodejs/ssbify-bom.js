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


var tomd = require('to-markdown')
var striptags = require('striptags')
var request = require('request')
var pull = require('pull-stream')
var createHash = require('crypto').createHash
var cheerio = require('cheerio')
var async = require('async')

var addBlob = function (sbot, buf, cb) {
  var hash = createHash('sha256')
  hash.update(buf)
  var blobId = '&' + hash.digest('base64') + '.sha256'
  pull(pull.values([buf]),
       sbot.blobs.add(blobId, function (err, res) {
         if (err) return cb(err)
         cb(null, blobId)
       }))
}

var waitUntilHas = function (sbot, hash, cb) {
  // i could not get this to work reliably
  // in any other way. The add command seems to
  // callback before the blob has actually been written.
  pull(
    sbot.blobs.get(hash),
    pull.collect(function (errIgnore, res) {
      // here we ignore the error of the blob not being
      // found in the filesystem.
      if (!res[0]) {
        setTimeout(function () {
          waitUntilHas(sbot, hash, cb)
        }, 100)
      } else {
        cb(null, hash)
      }
    }))
}

module.exports = function (sbot, htmlString, opts, cb) {
  var markItDown = function(blobbedHTML) {
    var md = '\ufeff# ' + opts.title + '\n\n' +
      striptags(tomd(doc.html())) + '\n\n' +
      '[source](' + opts.url + ')\n'

    // remove superflous newlines
    md = md.replace(/\n\s*\n/g, '\n\n')

    return md
  }

  var postMarkdownWithBlobs = function (blobRes) {
    var md = markItDown(blobRes)

    addBlob(sbot, new Buffer(md), function (err, res) {
      if (err) return cb(err)
      waitUntilHas(sbot, res, cb)
    })
  }

  if (typeof opts === 'string') opts = { url: opts }

  var doc = cheerio.load(htmlString)

  var modify = []
  doc('img').map(function (i, el) { modify.push(el) })

  async.map(modify, function (el, mcb) {
    var src = el.attribs.src
    request.get(src, { encoding: null }, function (err, res, body) {
      // if (err) throw err
      if ( res && res.statusCode === 200) {
        addBlob(sbot, new Buffer(body), function (err, res) {
          if (err) throw err
          el.attribs.src = res
          mcb(null)
        })
      } else {
        if (opts.ignoreBrokenImgLinks) {
          el.attribs.src = ''
          mcb(null)
        } else {
          mcb(null)
          //cb(new Error('broken image link at: ' + src))
        }
      }
    })
  }, function(blobRes) {
       if (opts.xmitAsBlob) postMarkdownWithBlobs(blobRes)
       else cb(null, markItDown(blobRes))
     })
}
