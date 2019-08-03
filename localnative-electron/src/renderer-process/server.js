/*
    Local Native
    Copyright (C) 2019  Yi Wang

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

const os = require('os');
const ZXing = require('@zxing/library')
const codeWriter = new ZXing.BrowserQRCodeSvgWriter();
const cmd = require('./cmd');
const {cmdServer, cmdClientStopServer} = require('./cmd');

document.addEventListener('DOMContentLoaded', function () {
  var addr = getIp() + ":2345";
  document.getElementById("addr").innerHTML = addr;
  codeWriter.writeToDom("#server-qr-code", addr, 300, 300);
  setTimeout(() =>{
    cmdServer();
  }, 3000)
})

function getIp(){
  var ifaces = os.networkInterfaces();
  var ip = "0.0.0.0";
  Object.keys(ifaces).forEach(function (ifname) {
    ifaces[ifname].forEach(function (iface) {
      if ('IPv4' !== iface.family || iface.internal !== false) {
        // skip over internal (i.e. 127.0.0.1) and non-ipv4 addresses
        return;
      }
      ip = iface.address
    });
  });
  return ip;
}
