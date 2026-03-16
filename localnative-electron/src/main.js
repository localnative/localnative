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
const version = "0.5.0"
const {app, BrowserWindow, ipcMain, dialog, shell, screen, desktopCapturer} = require('electron')
const debug = /--debug/.test(process.argv[2])
const os = require('os')
const neon = require('localnative-neon')

const path = require('path')
const glob = require('glob')
const files = glob.sync(path.join(__dirname, 'main-process/**/*.js'))
files.forEach((file) => { require(file) })

// --- IPC Handlers ---

// Run localnative-neon commands in the main process (safe from renderer)
ipcMain.handle('neon-run', (event, input) => {
  return neon.run(input)
})

// File dialog using modern Promise-based API
ipcMain.handle('open-file-dialog', async (event) => {
  const win = BrowserWindow.fromWebContents(event.sender)
  const result = await dialog.showOpenDialog(win, {
    title: 'Choose another LocalNative sqlite3 file to sync with',
    properties: ['openFile'],
    filters: [
      { name: 'sqlite3 Files', extensions: ['sqlite3'] },
    ]
  })
  if (!result.canceled && result.filePaths.length > 0) {
    return result.filePaths[0]
  }
  return null
})

// Open URL in external browser
ipcMain.handle('open-external', (event, url) => {
  return shell.openExternal(url)
})

// Open the server window
ipcMain.handle('open-server-window', (event) => {
  const serverWinPath = path.join('file://', __dirname, '/server.html')
  let win = new BrowserWindow({
    title: "Local Native Server",
    width: 600,
    height: 400,
    webPreferences: {
      nodeIntegration: false,
      contextIsolation: true,
      preload: path.join(__dirname, 'preload-server.js')
    }
  })
  win.webContents.on('crashed', () => {
    win.close()
  })
  win.on('close', () => { win = null })
  win.loadURL(serverWinPath)
  win.show()
})

// Desktop capture (moved to main process in Electron 10+)
ipcMain.handle('get-desktop-sources', async (event, options) => {
  const sources = await desktopCapturer.getSources(options)
  // Serialize thumbnails to data URLs since they can't cross contextBridge
  return sources.map(source => ({
    id: source.id,
    name: source.name,
    thumbnailDataUrl: source.thumbnail.toDataURL({ scaleFactor: 1 })
  }))
})

// Get primary display size
ipcMain.handle('get-primary-display-size', () => {
  return screen.getPrimaryDisplay().workAreaSize
})

// Minimize the focused window
ipcMain.handle('minimize-focused-window', () => {
  const win = BrowserWindow.getFocusedWindow()
  if (win) {
    win.minimize()
  }
})

// Get network interfaces for server IP detection
ipcMain.handle('get-network-interfaces', () => {
  return os.networkInterfaces()
})

// Keep a global reference of the window object
let mainWindow

function createWindow () {
  mainWindow = new BrowserWindow({
    webPreferences: {
        nodeIntegration: false,
        contextIsolation: true,
        preload: path.join(__dirname, 'preload.js')
    },
    width: 800, height: 600})

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
  }

  // Emitted when the window is closed.
  mainWindow.on('closed', function () {
    mainWindow = null
  })
}

app.on('ready', createWindow)

app.on('window-all-closed', function () {
  if (process.platform !== 'darwin') {
    app.quit()
  }
})

app.on('activate', function () {
  if (mainWindow === null) {
    createWindow()
  }
})
