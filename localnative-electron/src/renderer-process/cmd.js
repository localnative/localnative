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
const _ = require('underscore');
const neon = require('localnative-neon');

const LIMIT = 10;
exports.LIMIT = LIMIT;
exports.cmdFilter = _.debounce(filterImp, 500);
exports.cmdSelect = cmdSelect;
exports.cmdInsert = cmdInsert;
exports.cmdSearch = _.debounce(cmdSearchImp, 300);
exports.cmdSearchOrFilter = cmdSearchOrFilter;
exports.cmdSsbSync = cmdSsbSync;
exports.cmdDelete = cmdDelete;
exports.cmd = cmd;

// count
var count = 0;
exports.getCount = function(){
  return count;
}
exports.setCount = function(val){
  count = val;
}

// offset
var offset = 0;
exports.getOffset = function(){
  return offset;
}
exports.setOffset = function(val){
  offset = val;
}

const {onNativeMessage} = require('./ctrl');

let isFilter =  false;
function cmdSearchOrFilter(){
  if (isFilter){
    filterImp(range.from, range.to);
  } else {
    cmdSearch();
  }
}

var range = {};
function filterImp(from, to) {
  let message = {
    action: 'filter',
    query: document.getElementById('search-text').value,
    limit: LIMIT,
    offset: offset,
    from: from,
    to: to
  };
  range.from = from;
  range.to = to;
  cmd(message);
  isFilter = true;
}

function cmdSearch() {
  isFilter = false;
  document.getElementById('search-text').focus();
  var message = {
    action: "search",

    query: document.getElementById('search-text').value,
    limit: LIMIT,
    offset: offset
  };
  cmd(message);
  console.error("isFilter", isFilter);
}

function cmdSelect() {
  isFilter = false;
  var message = {
    action: "select",
    limit: LIMIT,
    offset: offset
  };
  cmd(message);
}

function cmdDelete(rowid) {
  var message = {
    action: "delete",

    query: document.getElementById('search-text').value,
    rowid: rowid,
    limit: LIMIT,
    offset: offset
  };
  cmd(message);
}

function cmd(message){
  let input = JSON.stringify(message, null, 2);
  requestMessage(">> " + input);
  var resp = JSON.parse(neon.run(input));
  onNativeMessage(resp);
}

function requestMessage(text) {
  document.getElementById('response-text').innerHTML = '<< running or failed :-( run ssb-server like <a href="https://github.com/ssbc/patchwork/releases">Patchwork</a> or <a href="https://github.com/ssbc/patchbay/releases">Patchbay</a> for ssb sync :-)';
  document.getElementById('request-text').innerHTML = Sanitizer.escapeHTML`${text}`;
}

function makeTags(str) {
  let s = str.replace(/,+/g, " ").trim();
  let l = s.replace(/\s+/g, ",").split(",");
  var set = {};
  l.forEach(function(tag){
    set[tag] = 1;
  });
  var arr = []
  for (var key in set){
    if(set.hasOwnProperty(key)) arr.push(key);
  }
  return arr.join(",");
}

function cmdInsert(annotations, is_public) {
  var message = {
    action: "insert",

    title: document.getElementById('title').value,
    url: document.getElementById('url').value,
    tags: makeTags(document.getElementById('tags-text').value),
    description: document.getElementById('desc-text').value,
    comments: "",
    annotations: annotations,

    limit: LIMIT,
    offset: offset,
    is_public: is_public
  };
  console.log(message);
  cmd(message);
}

function cmdSearchImp() {
  document.getElementById('search-text').focus();
  var message = {
    action: "search",

    query: document.getElementById('search-text').value,
    limit: LIMIT,
    offset: offset
  };
  cmd(message);

}

function cmdSelect() {
  var message = {
    action: "select",
    limit: LIMIT,
    offset: offset
  };
  cmd(message);
}

function cmdDelete(rowid) {
  var message = {
    action: "delete",

    query: document.getElementById('search-text').value,
    rowid: rowid,
    limit: LIMIT,
    offset: offset
  };
  cmd(message);
}

function cmdSsbSync() {
  var message = {
    action: "ssb-sync"
  };
  cmd(message);
}

function cmdSyncViaAttach(uri) {
  var message = {
    action: "sync-via-attach",
    uri: uri
  };
  cmd(message);
}

