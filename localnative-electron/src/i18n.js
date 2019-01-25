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

  document.getElementById('prev-btn').innerHTML = lc("prev");
  document.getElementById('next-btn').innerHTML = lc("next");
  document.getElementById('page').innerHTML = Sanitizer.escapeHTML`${lc("page")}`;
  document.getElementById('page-of').innerHTML = Sanitizer.escapeHTML`${lc("of")}`;

  document.getElementById('ssb-sync-btn').innerHTML = lc("ssb sync");
  document.getElementById('sync-via-attach-btn').innerHTML = lc("sync via attach file");


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
