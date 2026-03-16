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
const { contextBridge, ipcRenderer } = require('electron');

contextBridge.exposeInMainWorld('localNativeAPI', {
  // Run a command through the localnative-neon native module (runs in main process)
  neonRun: (input) => ipcRenderer.invoke('neon-run', input),

  // Open a file dialog to select a sqlite3 file for sync
  openFileDialog: () => ipcRenderer.invoke('open-file-dialog'),

  // Open a URL in the default external browser
  openExternal: (url) => ipcRenderer.invoke('open-external', url),

  // Open the server window
  openServerWindow: () => ipcRenderer.invoke('open-server-window'),

  // Take a screenshot (desktopCapturer moved to main process in Electron 10+)
  getDesktopSources: (options) => ipcRenderer.invoke('get-desktop-sources', options),

  // Get the primary display work area size
  getPrimaryDisplaySize: () => ipcRenderer.invoke('get-primary-display-size'),

  // Minimize the focused window
  minimizeFocusedWindow: () => ipcRenderer.invoke('minimize-focused-window'),
});
