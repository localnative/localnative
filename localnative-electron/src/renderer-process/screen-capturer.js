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
const {desktopCapturer, screen, shell, remote} = require('electron')
const cmd = require('./cmd')
const _ = require('underscore')

const screenshot = document.getElementById('screenshot-btn')
const screenshotText = document.getElementById('screenshot-text')

let takeScreenshot = _.debounce(takeScreenshotImp, 5000)

screenshot.addEventListener('click', (event) => {
  screenshotText.textContent = 'Screenshot in 5 seconds ...'
  _.delay(()=>{
    remote.BrowserWindow.getFocusedWindow().minimize();
    takeScreenshot();
  }, 1000)
})

function takeScreenshotImp(){
  const thumbSize = determineScreenShotSize()
  let options = { types: ['screen'], thumbnailSize: thumbSize }

  desktopCapturer.getSources(options, (error, sources) => {
    if (error) return console.log(error)

    sources.forEach((source) => {
      let dataUrl = source.thumbnail.toDataURL({scaleFactor:1})
      cmd.cmdInsertImage(dataUrl);
      screenshotText.textContent = 'Screenshot taken!'
    })
  })
}

function determineScreenShotSize () {
  const screenSize = screen.getPrimaryDisplay().workAreaSize
  const maxDimension = Math.max(screenSize.width, screenSize.height)
  return {
    width: maxDimension * window.devicePixelRatio,
    height: maxDimension * window.devicePixelRatio
  }
}
