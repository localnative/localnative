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

exports.filter = _.debounce(filterImp, 500);
exports.cmdSearch = cmdSearch;
exports.cmdDelete = cmdDelete;
exports.LIMIT = LIMIT;

const {onNativeMessage} = require('./ctrl');

function filterImp(from, to) {
  let message = {
    action: 'filter',
    query: document.getElementById('search-text').value,
    limit: LIMIT,
    offset: 0,
    from: from,
    to: to
  };
  console.log(message);
  let input = JSON.stringify(message);
  let resp = JSON.parse(neon.run(input));
  console.log(resp);
  onNativeMessage(resp);
}

function cmdSearch() {
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

function cmd(message){
  let input = JSON.stringify(message);
  requestMessage(">> " + input.substring(0,180) + " ...");
  var resp = JSON.parse(neon.run(input));
  onNativeMessage(resp);
  // cmdChart(message);
}

function requestMessage(text) {
  document.getElementById('response-text').innerHTML = '<< running or failed :-( run ssb-server like <a href="https://github.com/ssbc/patchwork/releases">Patchwork</a> or <a href="https://github.com/ssbc/patchbay/releases">Patchbay</a> for ssb sync :-)';
  document.getElementById('request-text').innerHTML = Sanitizer.escapeHTML`${text}`;
}
