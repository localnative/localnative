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

document.addEventListener('DOMContentLoaded', function () {
  // focus on tags
  document.getElementById('search-text').focus();

  // ssbify
  document.getElementById('ssbify').checked = JSON.parse(localStorage.getItem('ssbify'));

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
      if (document.getElementById('cb-public').checked) {
        window.cmd.cmdInsert("", true);
      } else {
        window.cmd.cmdInsert("", false);
      }
    }
  });

  // register prev and next
  document.getElementById('prev-btn').onclick = function() {
    window.appState.decOffset();
    window.cmd.cmdSearchOrFilter();
  };

  document.getElementById('next-btn').onclick = function() {
    window.appState.incOffset();
    window.cmd.cmdSearchOrFilter();
  };

  // register ssb-sync
  document.getElementById('ssb-sync-btn').onclick = function() {
    window.cmd.cmdSsbSync();
  };

  // sync-via-attach using IPC through preload
  document.getElementById('sync-via-attach-btn').onclick = async function() {
    const filePath = await window.localNativeAPI.openFileDialog();
    if (filePath) {
      window.cmd.cmdSyncViaAttach(filePath);
    }
  };

  // Open server window via IPC
  document.getElementById('start-server-btn').onclick = function() {
    window.localNativeAPI.openServerWindow();
  };

  document.getElementById('start-client-sync-btn').onclick = function() {
    var addr = document.getElementById('addr').value;
    window.cmd.cmdClientSync(addr);
  };

  document.getElementById('stop-server-btn').onclick = function() {
    window.cmd.cmdClientStopServer("127.0.0.1:2345");
  };

  // register cmdSearch
  document.getElementById('search-text').addEventListener('keyup', function (e) {
    window.appState.clearOffset();
    window.appState.clearRange();
    window.cmd.cmdSearch();
    lnDayChart.filterAll();
    lnMonthChart.filterAll();
  });

  document.getElementById('search-clear-btn').onclick = function() {
    document.getElementById('search-text').value = '';
    window.appState.clearOffset();
    window.appState.clearRange();
    window.cmd.cmdSearch();
    lnDayChart.filterAll();
    lnMonthChart.filterAll();
  };

  // initial query
  window.cmd.cmdSelect();

  // Open all links in external browser
  document.addEventListener('click', function (event) {
    if (event.target.tagName === 'A' && event.target.href.startsWith('http')) {
      event.preventDefault();
      window.localNativeAPI.openExternal(event.target.href);
    }
  });
});
