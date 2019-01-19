// This file is required by the index.html file and will
// be executed in the renderer process for that window.
// All of the Node.js APIs are available in this process.
let neon = require('localnative-neon');

function requestMessage(text) {
  document.getElementById('response-text').innerHTML = '<< running or failed :-( run ssb-server like <a href="https://github.com/ssbc/patchwork/releases">Patchwork</a> or <a href="https://github.com/ssbc/patchbay/releases">Patchbay</a> for ssb sync :-)';
  document.getElementById('request-text').innerHTML = Sanitizer.escapeHTML`${text}`;
}

function onNativeMessage(message) {
  let resp = "<< " +  JSON.stringify(message).substring(0, 90) + " ...";
  document.getElementById('response-text').innerHTML = Sanitizer.escapeHTML`${resp}`;
  // abort if no notes
  if (!message.notes) return;

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
          cmdSearch();
        }
      });
    }

  });
}

function i18nRefresh() {
  document.getElementById('label-ssbify').innerHTML = Sanitizer.escapeHTML`${lc('ssbify')}`;
  document.getElementById('label-public').innerHTML = Sanitizer.escapeHTML`${lc('public')}`;
  document.getElementById('label-language').innerHTML = Sanitizer.escapeHTML`${lc('language')}`;
  document.getElementById('title').placeholder = lc('title');
  document.getElementById('url').placeholder = lc('url');
  document.getElementById('tags-text').placeholder = lc("type to add tags, enter to save, comma or space as tag seperator");
  document.getElementById('desc-text').placeholder = lc('description');
  document.getElementById('search-text').placeholder = lc('type to search');
  document.getElementById('search-clear-btn').title = lc("clear search term(s)");
}

document.addEventListener('DOMContentLoaded', function () {
  // i18n
  let lang = localStorage.getItem('lang') || navigator.language;
  document.getElementById('select-language').value = locales[lang]? lang: 'en-US';
  lc = locales[lang] || locales['en-US'];
  i18nRefresh();
  document.getElementById('select-language').onchange = function (e) {
    let lang = e.target.options[e.target.selectedIndex].value;
    lc=locales[lang];
    i18nRefresh();
    localStorage.setItem('lang', lang);
  };

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

  // register ssb-sync
  document.getElementById('ssb-sync-btn').onclick = function(){
      cmdSsbSync();
  };

  document.getElementById('sync-via-attach-btn').onclick = function(){
      cmdSyncViaAttach("/home/e/Documents/localnative.sqlite3");
  };

  // register cmdSearch
  document.getElementById('search-text').addEventListener('keyup', function (e) {
      cmdSearch();
  });

  document.getElementById('search-clear-btn').onclick = function(){
    document.getElementById('search-text').value = '';
    cmdSearch();
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

let LIMIT = 20;

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
    offset: 0,
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

function cmdSsbSync() {
  var message = {
    action: "ssb-sync"
  };
  cmd(message);
}

function cmdSyncViaAttach(uri) {
  var message = {
    action: "sync-via-attach",
    uri: uri
  };
  cmd(message);
}

function cmd(message){
  let input = JSON.stringify(message);
  requestMessage(">> " + input.substring(0,180) + " ...");
  var resp = JSON.parse(neon.run(input));
  onNativeMessage(resp);
}

