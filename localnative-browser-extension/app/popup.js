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
// global state
let LIMIT = 10;
let offset = 0;
let count = 0;

// kicked off while <head> parses so prefs are resolved by DOMContentLoaded and
// the popup does not paint the light theme before the stored one applies
const prefsReady = chrome.storage.local.get(['darkTheme', 'saveContent', 'ssbify']).catch(function () { return {}; });

// Raw protocol traffic is developer-facing: the count it carried is already in
// the pagination indicator and failures raise a toast, so it goes to the
// console and to the indicator's tooltip rather than taking a row in the popup.
function logTraffic(direction, payload) {
  var line = direction + ' ' + JSON.stringify(payload);
  if (direction === '<<') {
    document.getElementById('indicator').title = line.substring(0, 500);
  }
  console.debug('[localnative]', direction, payload);
}

var toastTimer;

function showToast(text) {
  var el = document.getElementById('toast');
  // built as nodes rather than markup: the message comes from
  // chrome.runtime.lastError, and this way there is no HTML string to escape
  el.textContent = text + ' ';

  var link = document.createElement('a');
  link.href = 'https://localnative.app';
  link.target = '_blank';
  link.textContent = 'setup help';
  el.appendChild(link);

  var close = document.createElement('button');
  close.className = 'toast-close';
  close.title = 'Dismiss';
  close.textContent = '\u00d7';
  close.onclick = hideToast;
  el.appendChild(close);

  el.hidden = false;
  clearTimeout(toastTimer);
  toastTimer = setTimeout(hideToast, 8000);
}

function hideToast() {
  clearTimeout(toastTimer);
  document.getElementById('toast').hidden = true;
}

function onNativeMessage(message) {
  logTraffic('<<', message);
  // abort if no notes
  if (!message.notes) return;

  // show count
  if (Number(message.count) >=0 ) {
    count = message.count;
    document.getElementById('indicator').textContent = makePaginationText();
  }

  var notesEl = document.getElementById('notes');
  notesEl.textContent = '';

  // Built as DOM nodes rather than an HTML string. Note fields are user data
  // (including imported bookmarks), so this removes the escaping question
  // entirely and lets handlers bind to real element references instead of
  // ids reconstructed from rowid.
  function el(tag, className, text) {
    var e = document.createElement(tag);
    if (className) e.className = className;
    if (text !== undefined) e.textContent = text;
    return e;
  }

  message.notes.forEach(function (i) {
    var note = el('div', 'note');

    var line = el('div', 'note-line');
    var title = el('span', 'note-title', i.title);
    title.title = i.title;
    line.appendChild(title);

    var tagsEl = el('span', 'note-tags');
    line.appendChild(tagsEl);

    var del = el('button', 'btn-delete', '\u00d7');
    del.title = 'Delete';
    del.onclick = function () { cmdDelete(i.rowid); };
    line.appendChild(del);
    note.appendChild(line);

    var sub = el('div', 'note-sub');
    var date = el('span', 'note-id',
      String(i.created_at).replace(/^\d{4}-/, '').replace(/:\d{2}$/, ''));
    date.title = i.created_at;
    sub.appendChild(date);
    sub.appendChild(el('span', 'note-id', 'rowid ' + i.rowid));
    sub.appendChild(el('span', 'note-id', String(i.uuid4).substring(0, 5)));

    var urlWrap = el('span', 'note-url');
    var link = el('a', null, String(i.url).replace(/^https?:\/\//, ''));
    // only http(s) reaches href: escaping an attribute does not stop a
    // javascript: or data: url from being a live link
    link.href = /^https?:\/\//i.test(i.url) ? i.url : '#';
    link.target = '_blank';
    link.title = i.url;
    urlWrap.appendChild(link);
    sub.appendChild(urlWrap);
    note.appendChild(sub);

    var desc = el('div', 'note-desc', i.description);
    desc.title = i.description;
    note.appendChild(desc);

    if (i.tags.length > 0) {
      i.tags.split(',').forEach(function (tag) {
        var btn = el('button', 'tag-btn', tag);
        btn.onclick = function (e) {
          e.preventDefault();
          document.getElementById('search-text').value = tag;
          offset = 0;
          cmdSearch();
          document.getElementById('indicator').textContent = makePaginationText();
        };
        tagsEl.appendChild(btn);
      });
    }

    notesEl.appendChild(note);
  });
}

function connect() {
  var hostName = "app.localnative";
  var p = chrome.runtime.connectNative(hostName);
  var answered = false;

  p.onMessage.addListener(function (message) {
    answered = true;
    hideToast();
    onNativeMessage(message);
  });

  // localnative-web-ext-host is one-shot: it reads a single message, replies and
  // exits, so Chrome reports "Native host has exited." after every command --
  // including successful ones. Only a disconnect with no reply is a real
  // failure. lastError then names the cause: "Specified native messaging host
  // not found." (host manifest missing), "Access to the specified native
  // messaging host is forbidden." (extension id not in allowed_origins), or
  // "Native host has exited." (the binary died before answering).
  p.onDisconnect.addListener(function () {
    var err = chrome.runtime.lastError;
    if (answered) return;
    showToast(err ? err.message : 'The Local Native desktop app did not respond.');
  });

  return p;
}

function getPageContent(callback) {
  chrome.tabs.query({active: true, currentWindow: true}, function (tabs) {
    chrome.scripting.executeScript({
      target: {tabId: tabs[0].id},
      func: function () { return document.body.outerHTML; }
    }, function (results) {
      // annotations are optional: pages that disallow injection still save
      if (chrome.runtime.lastError || !results || !results[0]) {
        callback("");
        return;
      }
      callback(results[0].result || "");
    });
  });
}

document.addEventListener('DOMContentLoaded', async function () {
  // The manifest version is the extension's own. Platforms version
  // independently (docs/VERSIONING.md) and the desktop app is on a different
  // number, so label it rather than implying one app-wide version.
  var extVersion = chrome.runtime.getManifest().version;
  var versionEl = document.getElementById('version');
  versionEl.textContent = 'ext v' + extVersion;
  versionEl.title = 'Browser extension v' + extVersion +
    ' \u2014 versioned independently of the desktop app';

  const prefs = await prefsReady;

  // theme toggle
  const themeToggle = document.getElementById('theme-toggle');
  if (prefs.darkTheme) {
    document.body.classList.add('dark-theme');
    themeToggle.textContent = '☀️';
  }
  themeToggle.onclick = function() {
    document.body.classList.toggle('dark-theme');
    const isDark = document.body.classList.contains('dark-theme');
    themeToggle.textContent = isDark ? '☀️' : '🌙';
    chrome.storage.local.set({darkTheme: isDark});
  };

  // focus on tags
  document.getElementById('tags-text').focus();

  // save content -- stored as "ssbify" before the rename, so fall back to the
  // old key once and drop it on the next write
  const saveContent = prefs.saveContent !== undefined ? prefs.saveContent : !!prefs.ssbify;
  document.getElementById('save-content').checked = saveContent;

  document.getElementById('save-content').onchange = function (e) {
    chrome.storage.local.set({saveContent: e.target.checked});
    chrome.storage.local.remove('ssbify');
  };

  // cb-public
  document.getElementById('cb-public').onchange = function (e) {
    document.getElementById('save-content').disabled = e.target.checked;
  };

  // register cmdInsert
  document.getElementById('save-input').addEventListener('keypress', function (e) {
    var key = e.which || e.keyCode;
    if (key === 13) { // 13 is enter
      if(document.getElementById('save-content').checked && !document.getElementById('cb-public').checked){
        getPageContent(function(annotations){
          cmdInsert(annotations, false);
        });
      }else if(document.getElementById('cb-public').checked){
        cmdInsert("", true);
      }else{
        cmdInsert("", false);
      }
      document.getElementById('search-text').value ='';
      document.getElementById('search-text').focus();
    }
  });

  // register prev and next
  document.getElementById('prev-btn').onclick = function(){
    if(offset - LIMIT >= 0){
      offset -= LIMIT;
      cmdSearch();
      document.getElementById('indicator').textContent = makePaginationText();
    }
  };
  document.getElementById('next-btn').onclick = function(){
    if(offset + LIMIT < count){
      offset += LIMIT;
      cmdSearch();
      document.getElementById('indicator').textContent = makePaginationText();
    }
  };

  // register cmdSearch
  document.getElementById('search-text').addEventListener('keyup', function (e) {
      offset = 0;
      document.getElementById('indicator').textContent = makePaginationText();
      cmdSearch();
  });

  document.getElementById('search-clear-btn').onclick = function(e){
    e.preventDefault();
    document.getElementById('search-text').value = '';
    offset = 0;
    document.getElementById('indicator').textContent = makePaginationText();
    cmdSearch();
  };

  // initial query
  cmdSelect();
  document.getElementById('indicator').textContent = makePaginationText();

  chrome.tabs.query({'active': true, 'lastFocusedWindow': true}, function (tabs) {
    var title = tabs[0].title;
    var url = tabs[0].url;
    document.getElementById('title').value =  title ;
    document.getElementById('url').value =  url ;
  });

});

function makePaginationText() {
  let start = count > 0 ? offset + 1 : 0
  let end  = offset + LIMIT > count ? count : offset + LIMIT
  return `${start}-${end} / ${count}`
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
  var p = connect();
  p.postMessage(message);
  logTraffic('>>', message);
}
