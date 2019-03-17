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

var exports = module.exports = {};
const LIMIT = 10
let range = null
let offset = 0
let count = 0
let query = ""

exports.getLIMIT = function(){
  return LIMIT
}

exports.makePaginationText = function(){
    let start = count > 0 ? offset + 1 : 0
    let end  = offset + LIMIT > count ? count : offset + LIMIT
    return `${start}-${end} / ${count}`
}

exports.getQuery = function() {
  return query
}

exports.setQuery = function(q) {
  query = q
}

exports.setCount =  function(c) {
  count = c
}

exports.incOffset = function() {
  if(offset + LIMIT < count) {
      offset += LIMIT
  }
  return offset
}

exports.decOffset = function() {
  if(offset - LIMIT >=0) {
      offset -= LIMIT
  }
  return offset
}

exports.getOffset = function() {
  return offset
}

exports.clearOffset = function() {
  offset = 0
}

exports.getRange = function() {
  return range
}
exports.setRange = function(r) {
  range = r
}
exports.clearRange = function() {
  range = null
}
