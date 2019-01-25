// global state
let LIMIT = 10;
let offset = 0;
let count = 0;

function requestMessage(text) {
  document.getElementById('response-text').innerHTML = '<< running or failed :-( run <a href="https://localnative.app" target="_blank">desktop app</a> to finish setup browser extension! run ssb-server for ssb sync :-)';
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
    let pages = Math.ceil(count / LIMIT);
    document.getElementById('total-page').innerHTML = Sanitizer.escapeHTML`${pages}`;
  }

  document.getElementById('notes').innerHTML = '';
  var notesHTML = message.notes.forEach(function(i){
    // render one item
    document.getElementById('notes').insertAdjacentHTML('beforeend', Sanitizer.escapeHTML`
    <div class="note-sep"></div>
    <div class="note">
      <div class="note-created-at">
        ${i.created_at}
        rowid ${i.rowid}
        <span class="note-tags" id="note-tags-rowid-${i.rowid}">
        </span>
        <button id="btn-delete-rowid-${i.rowid}" title="delete" style="color: red; float:right;">
        Delete
        </button>
      </div>

      <div class="note-title">
        ${i.title}
      </div>

      <div class="note-url"><a target="_blank" href="${i.url}">${i.url}</a></div>

      <div class="note-desc">
        ${i.description}
      </div>

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
            <button id="note-tags-rowid-${i.rowid}-tag-${tag}">
             ${tag}
            </button>
            `);
        // tag search
        document.getElementById('note-tags-rowid-' + i.rowid + '-tag-' + tag).onclick = function(){
          document.getElementById('search-text').value = tag;
          offset = 0;
          cmdSearch();
          document.getElementById('page-idx-input').value = 1;
        }
      });
    }

  });
}

function onDisconnected() {
  // console.log("Disconnected: " + chrome.runtime.lastError.message);
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
      document.getElementById('search-text').value ='';
      document.getElementById('search-text').focus();
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

  chrome.tabs.query({'active': true, 'lastFocusedWindow': true}, function (tabs) {
    var title = tabs[0].title;
    var url = tabs[0].url;
    document.getElementById('title').value =  title ;
    document.getElementById('url').value =  url ;
  });

});


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

