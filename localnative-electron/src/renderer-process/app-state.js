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

window.appState = (function() {
  const LIMIT = 10;
  let range = null;
  let offset = 0;
  let count = 0;
  let query = "";

  return {
    getLIMIT: function() { return LIMIT; },

    makePaginationText: function() {
      let start = count > 0 ? offset + 1 : 0;
      let end  = offset + LIMIT > count ? count : offset + LIMIT;
      return `${start}-${end} / ${count}`;
    },

    getQuery: function() { return query; },
    setQuery: function(q) { query = q; },
    setCount: function(c) { count = c; },

    incOffset: function() {
      if(offset + LIMIT < count) {
        offset += LIMIT;
      }
      return offset;
    },

    decOffset: function() {
      if(offset - LIMIT >= 0) {
        offset -= LIMIT;
      }
      return offset;
    },

    getOffset: function() { return offset; },
    clearOffset: function() { offset = 0; },
    getRange: function() { return range; },
    setRange: function(r) { range = r; },
    clearRange: function() { range = null; }
  };
})();
