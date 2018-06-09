function statusMessage(text) {
  document.getElementById('status').innerHTML = '';
  document.getElementById('status').insertAdjacentHTML('beforeend', Sanitizer.escapeHTML`<p>${text}</p>`);
}

function onNativeMessage(message) {
  // console.log(message);
  document.getElementById('notes').innerHTML = '';
  var notesHTML = message.notes.map(function(i){
    document.getElementById('notes').insertAdjacentHTML('beforeend', Sanitizer.escapeHTML`
    <div class=note>
      <div class="note-created-at">${i.created_at}</div>
      <div class="note-title">${i.title}</div>
      <div class="note-url"><a target="_blank" href="${i.url}">${i.url}</a></div>
      <div class="note-tags">${i.tags}</div>
    </div>
    <div class="note-sep"></div>
      `)
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
  // focus on tags
  document.getElementById('tags-text').focus();

  // register cmdInsert
  document.getElementById('save-button').addEventListener(
      'click', cmdInsert);
  document.getElementById('save-input').addEventListener('keypress', function (e) {
    var key = e.which || e.keyCode;
    if (key === 13) { // 13 is enter
      cmdInsert();
    }
  });

  // register cmdSearch
  document.getElementById('search-button').addEventListener(
      'click', cmdSearch);
  document.getElementById('search-text').addEventListener('keypress', function (e) {
    var key = e.which || e.keyCode;
    if (key === 13) { // 13 is enter
      cmdSearch();
    }
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

function cmdInsert() {
  message = {
    action: "insert",

    title: document.getElementById('title').value,
    url: document.getElementById('url').value,
    tags: document.getElementById('tags-text').value,
    description: document.getElementById('desc-text').value,
    comments: "",
    annotations: "",

    limit: LIMIT,
    offset: 0
  };
  console.log(message);
  cmd(message);
}

function cmdSearch() {
  message = {
    action: "search",

    query: document.getElementById('search-text').value,
    limit: LIMIT,
    offset: 0
  };
  cmd(message);
}

function cmdSelect() {
  message = {
    action: "select",
    limit: LIMIT,
    offset: 0
  };
  cmd(message);
}

function cmd(message){
  var part = connect();
  port.postMessage(message);
  statusMessage("Sent command: " + JSON.stringify(message) );
}
