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
let lc;
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

  document.getElementById('prev-btn').innerHTML = Sanitizer.escapeHTML`${lc("prev")}`;
  document.getElementById('next-btn').innerHTML = Sanitizer.escapeHTML`${lc("next")}`;
  document.getElementById('page').innerHTML = Sanitizer.escapeHTML`${lc("page")}`;
  document.getElementById('page-of').innerHTML = Sanitizer.escapeHTML`${lc("of")}`;

  document.getElementById('ssb-sync-btn').innerHTML = Sanitizer.escapeHTML`${lc("ssb sync")}`;
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
})

