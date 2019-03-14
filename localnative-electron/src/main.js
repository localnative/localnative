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
const version = "0.3.8"
// Modules to control application life and create native browser window
const {app, BrowserWindow, ipcMain, dialog} = require('electron')
const debug = /--debug/.test(process.argv[2])

const path = require('path')
const glob = require('glob')
const files = glob.sync(path.join(__dirname, 'main-process/**/*.js'))
files.forEach((file) => { require(file) })

ipcMain.on('open-file-dialog', (event) => {
  const win = BrowserWindow.fromWebContents(event.sender)
  dialog.showOpenDialog(win, {
    title: 'Choose another LocalNative sqlite3 file to sync with',
    properties: ['openFile'],
    filters: [
      { name: 'sqlite3 Files', extensions: ['sqlite3'] },
    ]
  }, (files) => {
    if (files) {
      event.sender.send('selected-directory', files)
    }
  })
})

// Keep a global reference of the window object, if you don't, the window will
// be closed automatically when the JavaScript object is garbage collected.
let mainWindow

function createWindow () {
  // Create the browser window.
  mainWindow = new BrowserWindow({width: 800, height: 600})

  // set title
  let title = "Local Native v" + version +
    " - Node.js " + process.versions.node +
    " Chromium " + process.versions.chrome +
    " Electron " + process.versions.electron
  mainWindow.setTitle(title)
  mainWindow.on('page-title-updated', function(e) {
    e.preventDefault()
  });

  // and load the index.html of the app.
  mainWindow.loadFile('src/index.html')

  // Launch fullscreen with DevTools open, usage: npm run debug
  if (debug) {
    mainWindow.webContents.openDevTools()
    mainWindow.maximize()
    require('devtron').install()
  }

  // Emitted when the window is closed.
  mainWindow.on('closed', function () {
    // Dereference the window object, usually you would store windows
    // in an array if your app supports multi windows, this is the time
    // when you should delete the corresponding element.
    mainWindow = null
  })
}

// This method will be called when Electron has finished
// initialization and is ready to create browser windows.
// Some APIs can only be used after this event occurs.
app.on('ready', createWindow)

// Quit when all windows are closed.
app.on('window-all-closed', function () {
  // On macOS it is common for applications and their menu bar
  // to stay active until the user quits explicitly with Cmd + Q
  if (process.platform !== 'darwin') {
    app.quit()
  }
})

app.on('activate', function () {
  // On macOS it's common to re-create a window in the app when the
  // dock icon is clicked and there are no other windows open.
  if (mainWindow === null) {
    createWindow()
  }
})

// In this file you can include the rest of your app's specific main process
// code. You can also put them in separate files and require them here.
