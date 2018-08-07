function statusMessage(text) {
  document.getElementById('status').innerHTML = '';
  document.getElementById('status').insertAdjacentHTML('beforeend', Sanitizer.escapeHTML`<p>${text}</p>`);
}

function onNativeMessage(message) {
  document.getElementById('notes').innerHTML = '';
  var notesHTML = message.notes.forEach(function(i){
    // render one item
    document.getElementById('notes').insertAdjacentHTML('beforeend', Sanitizer.escapeHTML`
    <div class="note-sep"></div>
    <div class="note">
      <div class="note-created-at">
        ${i.created_at}
        rowid ${i.rowid}
        <button id="btn-delete-rowid-${i.rowid}" title="delete">
        X
        </button>
      </div>

      <div class="note-title">
        ${i.title}
      </div>

      <div class="note-url"><a target="_blank" href="${i.url}">${i.url}</a></div>

      <div class="note-desc">
        ${i.description}
      </div>

      <div class="note-tags">
        <span id="note-tags-rowid-${i.rowid}">
        </span>
      </div>
    </div>
      `);

    // delete button
    document.getElementById('btn-delete-rowid-' + i.rowid).onclick = function(){
      cmdDelete(i.rowid);
    };

    // tags
    i.tags.split(',').forEach(function(tag){
      document.getElementById('note-tags-rowid-' + i.rowid ).insertAdjacentHTML('beforeend', Sanitizer.escapeHTML`
          <button id="note-tags-rowid-${i.rowid}-tag-${tag}">
           ${tag}
          </button>
          `);
      // tag search
      document.getElementById('note-tags-rowid-' + i.rowid + '-tag-' + tag).onclick = function(){
        document.getElementById('search-text').value = tag;
        cmdSearch();;
      }
    });

  });
}

function onDisconnected() {
  console.log("Disconnected: " + chrome.runtime.lastError.message);
}

function connect() {
  var hostName = "app.localnative";
  port = chrome.runtime.connectNative(hostName);
  port.onMessage.addListener(onNativeMessage);
  port.onDisconnect.addListener(onDisconnected);
  return port;
}

document.addEventListener('DOMContentLoaded', function () {
  // setup content script
  chrome.tabs.executeScript({
    file: 'contentScript.js'
  });

  // focus on tags
  document.getElementById('tags-text').focus();

  // ssbify
  document.getElementById('ssbify').checked = JSON.parse(localStorage.getItem('ssbify'))

  document.getElementById('ssbify').onchange = function (e) {
    localStorage.setItem('ssbify', e.target.checked);
    console.log('ssbify is set to ' + e.target.checked);
  };

  // register cmdInsert
  document.getElementById('save-input').addEventListener('keypress', function (e) {
    var key = e.which || e.keyCode;
    if (key === 13) { // 13 is enter
      var annotations = "";
      if(document.getElementById('ssbify').checked){
        chrome.tabs.query({active: true, currentWindow: true}, function(tabs) {
          chrome.tabs.sendMessage(tabs[0].id, "get_content", function(content){
            annotations = content || "";
            cmdInsert(annotations);
          });
        });
      }else{
        cmdInsert("");
      }
    }
  });

  // register cmdSearch
  document.getElementById('search-text').addEventListener('keyup', function (e) {
      cmdSearch();
  });

  // initial query
  cmdSelect();

  chrome.tabs.query({'active': true, 'lastFocusedWindow': true}, function (tabs) {
    var title = tabs[0].title;
    var url = tabs[0].url;
    document.getElementById('title').value =  title ;
    document.getElementById('url').value =  url ;
  });

});

let LIMIT = 10;

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

function cmdInsert(annotations) {
  var message = {
    action: "insert",

    title: document.getElementById('title').value,
    url: document.getElementById('url').value,
    tags: makeTags(document.getElementById('tags-text').value),
    description: document.getElementById('desc-text').value,
    comments: "",
    annotations: annotations,

    limit: LIMIT,
    offset: 0
  };
  console.log(message);
  cmd(message);
}

function cmdSearch() {
  var message = {
    action: "search",

    query: document.getElementById('search-text').value,
    limit: LIMIT,
    offset: 0
  };
  cmd(message);
}

function cmdSelect() {
  var message = {
    action: "select",
    limit: LIMIT,
    offset: 0
  };
  cmd(message);
}

function cmdDelete(rowid) {
  var message = {
    action: "delete",

    query: document.getElementById('search-text').value,
    rowid: rowid,
    limit: LIMIT,
    offset: 0
  };
  cmd(message);
}

function cmd(message){
  var part = connect();
  port.postMessage(message);
  statusMessage("Sent command: " + JSON.stringify(message.action) );
}

