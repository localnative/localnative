let version = '0.3.4'

var platform = "-gnu-linux"
switch(process.platform){
  case "darwin":
    platform = "-mac";
    break;
  case "win32":
    platform = ".ext";
}

let addon = require('../native');
let fs = require('fs');

// create dir
const home = require('os').homedir();
const dir = home +'/LocalNative';
const dirBin = dir + '/bin';
const dirConfig = dir + '/config';
if (!fs.existsSync(dir)){
  fs.mkdirSync(dir, {recursive: true});
}

if (!fs.existsSync(dirBin)){
  fs.mkdirSync(dirBin, {recursive: true});
}


// copy binary files
let webExtFileName = 'localnative-web-ext-host' + '-' + version + platform;
let webExtSource = __dirname + '/' + webExtFileName;
let webExtTarget = dirBin + '/' + webExtFileName;
if (!fs.existsSync(webExtTarget)){
  fs.copyFileSync(webExtSource, webExtTarget);
}

let NodePkgFileName = 'localnative-nodejs' + '-' + version;
let NodePkgSource = __dirname + '/' + NodePkgFileName;
let NodePkgTarget = dirBin + '/' + NodePkgFileName;
if (!fs.existsSync(NodePkgTarget)){
  fs.copyFileSync(NodePkgSource, NodePkgTarget);
}

// create manifest
let jsonMozilla = {
  name: "app.localnative",
  description: "Local Native Host",
  path: webExtTarget,
  type: "stdio",
  allowed_extensions: [
    "localnative@example.org"
  ]
}

let jsonChrome = {
  "name": "app.localnative",
  "description": "Local Native Host",
  "path": webExtTarget,
  "type": "stdio",
  "allowed_origins": [
    "chrome-extension://oclkmkeameccmgnajgogjlhdjeaconnb/"
  ]
}

var pathMozilla = home + '/.mozilla/native-messaging-hosts/app.localnative.json'
var pathChrome = home + '/.config/chromium/NativeMessagingHosts/app.localnative.json'
if (platform == '-mac') {
  pathMozilla = home + '/Library/Application Support/Mozilla/NativeMessagingHosts/app.localnative.json'
  pathChrome = home + '/Library/Application Support/Google/Chrome/NativeMessagingHosts/app.localnative.json'
}
if (platform == '.exe'){
  if (!fs.existsSync(dirConfig)){
    fs.mkdirSync(dirConfig, {recursive: true});
  }
  pathMozilla = dirConfig + '/app.localnative.firefox.json'
  pathChrome = dirConfig + '/app.localnative.chrome.json'
}
fs.writeFileSync(pathMozilla, JSON.stringify(jsonMozilla, null, 2))
fs.writeFileSync(pathChrome, JSON.stringify(jsonChrome, null, 2))

module.exports = {
  dir: __dirname,
  run: addon.run
};

