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
const {LIMIT, cmdDelete, cmdSearch, getOffset, setOffset, setCount} = require('./cmd')
const {refreshChart} = require('./chart')
const _ = require('underscore')

function onNativeMessage(message) {
  let msg = _.omit(message, 'days', 'notes', 'tags');
  let resp = "<< " +  JSON.stringify(msg, null, 2);
  refreshTags(message);
  document.getElementById('response-text').innerHTML = Sanitizer.escapeHTML`${resp}`;
  // abort if no notes
  if (!message.notes) return;

  // show page count
  if (Number(message.count) >=0 ) {
    let count = message.count;
    setCount(count);
    let pages = Math.ceil(count / LIMIT);
    document.getElementById('total-page').innerHTML = Sanitizer.escapeHTML`${pages}`;
  }
  refreshNotes(message.notes);

  if (message.days // filter result has no days field
    && getOffset() == 0 // only first page refresh chart
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
      <li>${t.k}  ${t.v}</li>
      `);
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
          setOffset(0);
          cmdSearch();
          document.getElementById('page-idx-input').value = 1;
        }
      });
    }

  });
}
