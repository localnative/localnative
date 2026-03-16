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

window.cmd = (function() {
  let LIMIT = window.appState.getLIMIT();

  function cmdInsertImage(dataURL) {
    let message = {
      action: "insert-image",
      title: "Screenshot_" + (new Date()).toISOString(),
      url: "mime://image/png",
      tags: "screenshot",
      description: "",
      comments: "",
      annotations: dataURL,
      limit: LIMIT,
      offset: window.appState.getOffset(),
      is_public: false
    };
    runCmd(message);
  }

  function cmdSearchOrFilter() {
    let range = window.appState.getRange();
    if (range) {
      filterImp(range[0], range[1]);
    } else {
      cmdSearchImp();
    }
  }

  function filterImp(from, to) {
    let message = {
      action: 'filter',
      query: document.getElementById('search-text').value,
      limit: LIMIT,
      offset: window.appState.getOffset(),
      from: from,
      to: to
    };
    runCmd(message);
  }

  async function runCmd(message) {
    let input = JSON.stringify(message, null, 2);
    requestMessage(">> " + input);
    try {
      let result = await window.localNativeAPI.neonRun(input);
      let resp = JSON.parse(result);
      if (resp.count) {
        window.appState.setCount(resp.count);
      }
      window.ctrl.onNativeMessage(resp);
    } catch (err) {
      console.error('neon-run error:', err);
    }
  }

  function requestMessage(text) {
    document.getElementById('response-text').innerHTML = '<< running or failed :-( run ssb-server like <a href="https://github.com/ssbc/patchwork/releases">Patchwork</a> or <a href="https://github.com/ssbc/patchbay/releases">Patchbay</a> for ssb sync :-)';
    document.getElementById('request-text').innerHTML = Sanitizer.escapeHTML`${text}`;
  }

  function makeTags(str) {
    let s = str.replace(/,+/g, " ").trim();
    let l = s.replace(/\s+/g, ",").split(",");
    var set = {};
    l.forEach(function(tag) {
      set[tag] = 1;
    });
    var arr = [];
    for (var key in set) {
      if (set.hasOwnProperty(key)) arr.push(key);
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
      offset: window.appState.getOffset(),
      is_public: is_public
    };
    console.log(message);
    runCmd(message);
  }

  function cmdSearchImp() {
    window.appState.clearOffset();
    window.appState.clearRange();
    document.getElementById('search-text').focus();
    var message = {
      action: "search",
      query: document.getElementById('search-text').value,
      limit: LIMIT,
      offset: window.appState.getOffset()
    };
    runCmd(message);
  }

  function cmdSelect() {
    window.appState.clearOffset();
    window.appState.clearRange();
    var message = {
      action: "select",
      limit: LIMIT,
      offset: window.appState.getOffset()
    };
    runCmd(message);
  }

  function cmdDelete(rowid) {
    var message = {
      action: "delete",
      query: document.getElementById('search-text').value,
      rowid: rowid,
      limit: LIMIT,
      offset: window.appState.getOffset()
    };
    runCmd(message);
  }

  function cmdSsbSync() {
    var message = {
      action: "ssb-sync"
    };
    runCmd(message);
  }

  function cmdSyncViaAttach(uri) {
    var message = {
      action: "sync-via-attach",
      uri: uri
    };
    runCmd(message);
  }

  function cmdServer() {
    var message = {
      action: "server",
      addr: "127.0.0.1:2345"
    };
    runCmd(message);
  }

  function cmdClientSync(addr) {
    var message = {
      action: "client-sync",
      addr: addr
    };
    runCmd(message);
  }

  function cmdClientStopServer(addr) {
    var message = {
      action: "client-stop-server",
      addr: addr
    };
    runCmd(message);
  }

  return {
    LIMIT: LIMIT,
    cmdFilter: _.debounce(filterImp, 500),
    cmdSelect: cmdSelect,
    cmdInsert: cmdInsert,
    cmdInsertImage: cmdInsertImage,
    cmdSearch: _.debounce(cmdSearchImp, 300),
    cmdSearchOrFilter: cmdSearchOrFilter,
    cmdSsbSync: cmdSsbSync,
    cmdDelete: cmdDelete,
    cmdSyncViaAttach: cmdSyncViaAttach,
    cmdServer: cmdServer,
    cmdClientSync: cmdClientSync,
    cmdClientStopServer: cmdClientStopServer
  };
})();
