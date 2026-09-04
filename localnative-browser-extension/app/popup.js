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
const prefsReady = chrome.storage.local.get(['darkTheme', 'ssbify']).catch(function () { return {}; });

function requestMessage(text) {
  document.getElementById('response-text').innerHTML = '<< running or failed :-( run <a href="https://localnative.app" target="_blank">desktop app</a> to finish setup browser extension!';
  document.getElementById('request-text').innerHTML = Sanitizer.escapeHTML`${text}`;
}

function onNativeMessage(message) {
  let resp = "<< " +  JSON.stringify(message).substring(0, 90) + " ...";
  document.getElementById('response-text').innerHTML = Sanitizer.escapeHTML`${resp}`;
  // abort if no notes
  if (!message.notes) return;

  // show count
  if (Number(message.count) >=0 ) {
    count = message.count;
    document.getElementById('indicator').innerHTML = makePaginationText();
  }

  document.getElementById('notes').innerHTML = '';
  var notesHTML = message.notes.forEach(function(i){
    // render one item
    document.getElementById('notes').insertAdjacentHTML('beforeend', Sanitizer.escapeHTML`
    <div class="note">
      <div class="note-meta">
        <span>${i.created_at}</span>
        <span>${i.uuid4.substring(0,5)}</span>
        <span>rowid ${i.rowid}</span>
        <span class="note-tags" id="note-tags-rowid-${i.rowid}"></span>
        <button class="btn-delete" id="btn-delete-rowid-${i.rowid}" title="Delete">Delete</button>
      </div>
      <div class="note-title">${i.title}</div>
      <div class="note-url"><a target="_blank" href="${i.url}">${i.url}</a></div>
      <div class="note-desc">${i.description}</div>
    </div>
      `);

    // delete button
    document.getElementById('btn-delete-rowid-' + i.rowid).onclick = function(){
      cmdDelete(i.rowid);
    };

    // tags
    if(i.tags.length > 0){
      i.tags.split(',').forEach(function(tag){
        document.getElementById('note-tags-rowid-' + i.rowid ).insertAdjacentHTML('beforeend', Sanitizer.escapeHTML`
            <button class="tag-btn" id="note-tags-rowid-${i.rowid}-tag-${tag}">${tag}</button>
            `);
        // tag search
        document.getElementById('note-tags-rowid-' + i.rowid + '-tag-' + tag).onclick = function(e){
          e.preventDefault();
          document.getElementById('search-text').value = tag;
          offset = 0;
          cmdSearch();
          document.getElementById('indicator').innerHTML = makePaginationText();
        }
      });
    }

  });
}

function onDisconnected() {
  // lastError names the actual cause -- "Specified native messaging host not
  // found." (host manifest missing), "Access to the specified native messaging
  // host is forbidden." (extension ID not in allowed_origins), "Native host has
  // exited." (binary crashed). Without surfacing it every failure collapses into
  // the same generic setup hint and is undebuggable from the popup.
  var err = chrome.runtime.lastError;
  if (!err) return;
  document.getElementById('response-text').innerHTML =
    Sanitizer.escapeHTML`<< ${err.message}` +
    ' &mdash; see <a href="https://localnative.app" target="_blank">localnative.app</a> to finish setup';
}

function connect() {
  var hostName = "app.localnative";
  port = chrome.runtime.connectNative(hostName);
  port.onMessage.addListener(onNativeMessage);
  port.onDisconnect.addListener(onDisconnected);
  return port;
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
  // single source of truth: the manifest, so the header cannot drift on release
  document.getElementById('version').textContent = 'v' + chrome.runtime.getManifest().version;

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

  // ssbify
  document.getElementById('ssbify').checked = !!prefs.ssbify;

  document.getElementById('ssbify').onchange = function (e) {
    chrome.storage.local.set({ssbify: e.target.checked});
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
      if(document.getElementById('ssbify').checked && !document.getElementById('cb-public').checked){
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
      document.getElementById('indicator').innerHTML = makePaginationText();
    }
  };
  document.getElementById('next-btn').onclick = function(){
    if(offset + LIMIT < count){
      offset += LIMIT;
      cmdSearch();
      document.getElementById('indicator').innerHTML = makePaginationText();
    }
  };

  // register ssb-sync
  document.getElementById('ssb-sync-btn').onclick = function(){
      cmdSsbSync();
  };

  // register cmdSearch
  document.getElementById('search-text').addEventListener('keyup', function (e) {
      offset = 0;
      document.getElementById('indicator').innerHTML = makePaginationText();
      cmdSearch();
  });

  document.getElementById('search-clear-btn').onclick = function(e){
    e.preventDefault();
    document.getElementById('search-text').value = '';
    offset = 0;
    document.getElementById('indicator').innerHTML = makePaginationText();
    cmdSearch();
  };

  // initial query
  cmdSelect();
  document.getElementById('indicator').innerHTML = makePaginationText();

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

function cmdSsbSync() {
  var message = {
    action: "ssb-sync"
  };
  cmd(message);
}

function cmd(message){
  var part = connect();
  port.postMessage(message);
  requestMessage(">> " + JSON.stringify(message).substring(0,180) + " ...");
}
