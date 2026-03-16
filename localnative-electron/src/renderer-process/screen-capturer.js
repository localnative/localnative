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

(function() {
  const screenshot = document.getElementById('screenshot-btn');
  const screenshotText = document.getElementById('screenshot-text');

  let takeScreenshot = _.debounce(takeScreenshotImp, 5000);

  screenshot.addEventListener('click', function(event) {
    screenshotText.textContent = 'Screenshot in 5 seconds ...';
    _.delay(function() {
      window.localNativeAPI.minimizeFocusedWindow();
      takeScreenshot();
    }, 1000);
  });

  async function takeScreenshotImp() {
    try {
      const screenSize = await window.localNativeAPI.getPrimaryDisplaySize();
      const maxDimension = Math.max(screenSize.width, screenSize.height);
      const thumbSize = {
        width: maxDimension * window.devicePixelRatio,
        height: maxDimension * window.devicePixelRatio
      };
      let options = { types: ['screen'], thumbnailSize: thumbSize };

      const sources = await window.localNativeAPI.getDesktopSources(options);
      sources.forEach(function(source) {
        window.cmd.cmdInsertImage(source.thumbnailDataUrl);
        screenshotText.textContent = 'Screenshot taken!';
      });
    } catch (error) {
      console.error('Screenshot error:', error);
      screenshotText.textContent = 'Screenshot failed.';
    }
  }
})();
