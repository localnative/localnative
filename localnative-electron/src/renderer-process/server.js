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

document.addEventListener('DOMContentLoaded', async function () {
  var addr = await getIp() + ":2345";
  document.getElementById("addr").innerHTML = addr;
  var codeWriter = new ZXing.BrowserQRCodeSvgWriter();
  codeWriter.writeToDom("#server-qr-code", addr, 300, 300);
  setTimeout(function() {
    cmdServer();
  }, 3000);
});

async function getIp() {
  try {
    var ifaces = await window.localNativeAPI.getNetworkInterfaces();
    var ip = "0.0.0.0";
    Object.keys(ifaces).forEach(function (ifname) {
      ifaces[ifname].forEach(function (iface) {
        if ('IPv4' !== iface.family || iface.internal !== false) {
          return;
        }
        ip = iface.address;
      });
    });
    return ip;
  } catch (err) {
    console.error('Failed to get network interfaces:', err);
    return "0.0.0.0";
  }
}

async function cmdServer() {
  var message = {
    action: "server",
    addr: "127.0.0.1:2345"
  };
  var input = JSON.stringify(message, null, 2);
  try {
    var result = await window.localNativeAPI.neonRun(input);
    console.log('Server started:', result);
  } catch (err) {
    console.error('Server start error:', err);
  }
}
