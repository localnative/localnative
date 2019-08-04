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
const path = require('path')
const neon = require('localnative-neon');
const {ipcRenderer} = require('electron');
const {BrowserWindow, dialog} = require('electron').remote;
const {cmdChart} = require('./chart');
const appState = require('./app-state');
const cmd = require('./cmd');
const {LIMIT, cmdSsbSync, cmdSyncViaAttach, cmdSelect, cmdInsert, cmdSearch
  , cmdSearchOrFilter, cmdServer, cmdClientSync, cmdClientStopServer} = require('./cmd');

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
    appState.decOffset();
    cmd.cmdSearchOrFilter();
  };

  document.getElementById('next-btn').onclick = function(){
    appState.incOffset();
    cmd.cmdSearchOrFilter();
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

  document.getElementById('start-server-btn').onclick = function(){
    const serverWinPath = path.join('file://', __dirname, '../server.html')
    let win = new BrowserWindow({
      title: "Local Native Server",
      width: 600,
      height: 400,
      webPreferences: {
        nodeIntegration: true
      }
    });

    // server stopped
    win.webContents.on('crashed', () => {
      win.close();
    })

    win.on('close', () => { win = null })
    win.loadURL(serverWinPath)
    win.show()
  };

  document.getElementById('start-client-sync-btn').onclick = function(){
    var addr = document.getElementById('addr').value
    cmdClientSync(addr)
  };

  document.getElementById('stop-server-btn').onclick = function(){
    cmdClientStopServer("127.0.0.1:2345")
  };
  // register cmdSearch
  document.getElementById('search-text').addEventListener('keyup', function (e) {
      appState.clearOffset();
      appState.clearRange();
      cmd.cmdSearch();
      lnDayChart.filterAll();
      lnMonthChart.filterAll();
  });

  document.getElementById('search-clear-btn').onclick = function(){
    document.getElementById('search-text').value = '';
    appState.clearOffset();
    appState.clearRange();
    cmd.cmdSearch();
    lnDayChart.filterAll();
    lnMonthChart.filterAll();
  };

  // initial query
  cmdSelect();

  // Open all links in external browser
  let shell = require('electron').shell
  document.addEventListener('click', function (event) {
    if (event.target.tagName === 'A' && event.target.href.startsWith('http')) {
      event.preventDefault()
      shell.openExternal(event.target.href)
    }
  })

});
