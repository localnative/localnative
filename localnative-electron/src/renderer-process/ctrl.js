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
var exports = module.exports = {};
exports.onNativeMessage = onNativeMessage;
const appState =  require('./app-state');
const cmd = require('./cmd')
const {LIMIT, cmdDelete, cmdSearch} = require('./cmd')
const {refreshChart} = require('./chart')
const _ = require('underscore')
const ZXing = require('@zxing/library')
const codeWriter = new ZXing.BrowserQRCodeSvgWriter();

function onNativeMessage(message) {
  let msg = _.omit(message, 'days', 'notes', 'tags');
  let resp = "<< " +  JSON.stringify(msg, null, 2);
  refreshTags(message);
  document.getElementById('response-text').innerHTML = Sanitizer.escapeHTML`${resp}`;
  // abort if no notes
  if (!message.notes) return;

  // show page count
  if (Number(message.count) >=0 ) {
    document.getElementById('pagination-text').innerHTML = appState.makePaginationText();
  }
  refreshNotes(message.notes);

  if (message.days // filter result has no days field
    && appState.getOffset() == 0 // only first page refresh chart
  ){
    refreshChart(message.days);
  }
}


function refreshTags(message){
  if(message.tags){
    let tags = _.sortBy(message.tags, 'v').reverse();
    let dom = document.getElementById('tags');
    dom.innerHTML = '';
    tags.forEach(function(t){
      // render one item
      dom.insertAdjacentHTML('beforeend', Sanitizer.escapeHTML`
      <span><button id="tag-${t.k}">${t.k}</button>${t.v>1?t.v:''}</span>
      `);

      document.getElementById('tag-'+t.k).onclick = function() {
        document.getElementById('search-text').value = t.k;

        appState.clearOffset();
        appState.clearRange();
        cmd.cmdSearch();
        lnDayChart.filterAll();
        lnMonthChart.filterAll();
      }

    });
  }
}

function refreshNotes(notes){
  document.getElementById('notes').innerHTML = '';
  notes.forEach(function(i){
    // render one item
    document.getElementById('notes').insertAdjacentHTML('beforeend', Sanitizer.escapeHTML`
    <div class="note-sep"></div>
    <div class="note">
      <div class="note-created-at">
        ${i.created_at.substring(0,19)} UTC
        uuid ${i.uuid4.substring(0,5)}..
        rowid ${i.rowid}
        <button id="btn-qrcode-rowid-${i.rowid}" title="QR" style="color: gray;">
        QR
        </button>
        <span class="note-tags" id="note-tags-rowid-${i.rowid}">
        </span>
        <button id="btn-delete-rowid-${i.rowid}" title="delete" style="color: red; float:right;">
        Delete
        </button>
      </div>
      <div id="qr-code-${i.rowid}"></div>

      <div class="note-title">
        ${i.title}
      </div>

      <div class="note-url" style="overflow-x:auto"><a target="_blank" href="${i.url}">${i.url}</a></div>

      <div class="note-desc">
        ${i.description}
      </div>
      <img src="${i.annotations}" style="width:400px">
    </div>
      `);

    // qrcode toggle button
    document.getElementById('btn-qrcode-rowid-' + i.rowid).onclick = function(){
      let qr = document.getElementById(`qr-code-${i.rowid}`);
      if (qr.innerHTML.length > 0){
        qr.innerHTML = '';
      } else {
        codeWriter.writeToDom(`#qr-code-${i.rowid}`, i.url, 300, 300);
      }
    };

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
          appState.clearOffset();
          appState.clearRange();
          cmd.cmdSearch();
          lnDayChart.filterAll();
          lnMonthChart.filterAll();
        }
      });
    }

  });
}
