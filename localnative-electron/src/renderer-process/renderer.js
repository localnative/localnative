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

offset = 0;
count = 0;
const neon = require('localnative-neon');
const {ipcRenderer} = require('electron');
const {cmdChart} = require('./chart');
const {LIMIT, cmdSyncViaAttach, cmdSelect, cmdInsert, cmdSearch} = require('./cmd');

document.addEventListener('DOMContentLoaded', function () {
  // focus on tags
  document.getElementById('search-text').focus();

  // ssbify
  document.getElementById('ssbify').checked = JSON.parse(localStorage.getItem('ssbify'))

  document.getElementById('ssbify').onchange = function (e) {
    localStorage.setItem('ssbify', e.target.checked);
    console.log('ssbify is set to ' + e.target.checked);
  };

  // cb-public
  document.getElementById('cb-public').onchange = function (e) {
    document.getElementById('ssbify').disabled = e.target.checked;
  };

  // register cmdInsert
  document.getElementById('save-input').addEventListener('keypress', function (e) {
    var key = e.which || e.keyCode;
    if (key === 13) { // 13 is enter
      var annotations = "";
      if(document.getElementById('ssbify').checked && !document.getElementById('cb-public').checked){
        chrome.tabs.query({active: true, currentWindow: true}, function(tabs) {
          chrome.tabs.sendMessage(tabs[0].id, "get_content", function(content){
            annotations = content || "";
            cmdInsert(annotations, false);
          });
        });
      }else if(document.getElementById('cb-public').checked){
        cmdInsert("", true);
      }else{
        cmdInsert("", false);
      }
    }
  });

  // register prev and next
  document.getElementById('prev-btn').onclick = function(){
    if(offset - LIMIT >= 0){
      offset -= LIMIT;
      cmdSearch();
      document.getElementById('page-idx-input').value = Number(document.getElementById('page-idx-input').value) - 1;
    }
  };
  document.getElementById('next-btn').onclick = function(){
    if(offset + LIMIT <= count){
      offset += LIMIT;
      cmdSearch();
      document.getElementById('page-idx-input').value = Number(document.getElementById('page-idx-input').value) + 1;
    }
  };

  // register ssb-sync
  document.getElementById('ssb-sync-btn').onclick = function(){
      cmdSsbSync();
  };

  // sync-via-attach
  document.getElementById('sync-via-attach-btn').onclick = function(){
    ipcRenderer.send('open-file-dialog');
  };
  ipcRenderer.on('selected-directory', (event, path) => {

    cmdSyncViaAttach(path[0]);
  });

  // register cmdSearch
  document.getElementById('search-text').addEventListener('keyup', function (e) {
      offset = 0;
      document.getElementById('page-idx-input').value = 1;
      cmdSearch();
  });

  document.getElementById('search-clear-btn').onclick = function(){
    document.getElementById('search-text').value = '';
    offset = 0;
    document.getElementById('page-idx-input').value = 1;
    cmdSearch();
  };

  // initial query
  cmdSelect();
  document.getElementById('page-idx-input').value = 1;

  // Open all links in external browser
  let shell = require('electron').shell
  document.addEventListener('click', function (event) {
    if (event.target.tagName === 'A' && event.target.href.startsWith('http')) {
      event.preventDefault()
      shell.openExternal(event.target.href)
    }
  })

});
