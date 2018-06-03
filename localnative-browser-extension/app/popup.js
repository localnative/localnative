function statusMessage(text) {
  document.getElementById('status').innerHTML = "<p>" + text + "</p>";
}

function onNativeMessage(message) {
  // console.log(message);
  var notesHTML = message.notes.map(function(i){
    return '<div class=note>'
      + '<div class="note-created-at">' + i.created_at + "</div>"
      + '<div class="note-title">' + i.title + "</div>"
      + '<div class="note-url">' + '<a target="_blank" href="' + i.url  + '">' +i.url+'</a></div>'
      + '<div class="note-tags">' + i.tags + "</div>"
      + '</div>';
  }).join('<div class="note-sep"></div>');
  console.log(notesHTML);
  document.getElementById('notes').innerHTML =  notesHTML;
}

function onDisconnected() {
  console.log("Disconnected: " + chrome.runtime.lastError.message);
}

function connect() {
  var hostName = "app.localnative";
  statusMessage("Connecting to native messaging host <b>" + hostName + "</b>")
  port = chrome.runtime.connectNative(hostName);
  port.onMessage.addListener(onNativeMessage);
  port.onDisconnect.addListener(onDisconnected);
  return port;
}

document.addEventListener('DOMContentLoaded', function () {
  document.getElementById('save-button').addEventListener(
      'click', cmdInsert);
  document.getElementById('search-button').addEventListener(
      'click', cmdSearch);
  cmdSelect();

  chrome.tabs.query({'active': true, 'lastFocusedWindow': true}, function (tabs) {
    var title = tabs[0].title;
    var url = tabs[0].url;
    document.getElementById('title').value =  title ;
    document.getElementById('url').value =  url ;
  });

});

function cmdInsert() {
  message = {
    action: "insert",

    title: document.getElementById('title').value,
    url: document.getElementById('url').value,
    tags: document.getElementById('tags-text').value,
    description: document.getElementById('desc-text').value,
    comments: "",
    annotations: "",

    limit: 5,
    offset: 0
  };
  console.log(message);
  cmd(message);
}

function cmdSearch() {
  message = {
    action: "search",

    query: document.getElementById('search-text').value,
    limit: 5,
    offset: 0
  };
  cmd(message);
}

function cmdSelect() {
  message = {
    action: "select",
    limit: 5,
    offset: 0
  };
  cmd(message);
}

function cmd(message){
  var part = connect();
  port.postMessage(message);
  statusMessage("Sent message: <b>" + JSON.stringify(message) + "</b>");
}
