let version = '0.3.5'

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

// manifest content
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

// assign path value, default platform
var pathMozilla = home + '/.mozilla/native-messaging-hosts'
var pathChromium = home + '/.config/chromium/NativeMessagingHosts'
var pathChrome = home + '/.config/google-chrome/NativeMessagingHosts'
if (platform == '-mac') {
  pathMozilla = home + '/Library/Application Support/Mozilla/NativeMessagingHosts'
  pathChromium = home + '/Library/Application Support/Chromium/NativeMessagingHosts'
  pathChrome = home + '/Library/Application Support/Google/Chrome/NativeMessagingHosts'
}
if (platform == '.exe'){
  if (!fs.existsSync(dirConfig)){
    try{
      fs.mkdirSync(dirConfig, {recursive: true});
    }catch(error){
      console.error(error);
    }
  }
  pathMozilla = dirConfig + '/mozilla'
  pathChrome = dirConfig + '/chrome'
}

// create directory
if (!fs.existsSync(pathMozilla)){
  try{
    fs.mkdirSync(pathMozilla, {recursive: true});
  }catch(error){
    console.error(error);
  }
}
// windows seems not have chromium
if (platform != '.exe' && !fs.existsSync(pathChromium)){
  try{
    fs.mkdirSync(pathChromium, {recursive: true});
  }catch(error){
    console.error(error);
  }
}
if (!fs.existsSync(pathChrome)){
  try{
    fs.mkdirSync(pathChrome, {recursive: true});
  }catch(error){
    console.error(error);
  }
}

// create manifest file
if (fs.existsSync(pathMozilla)){
  try{
    fs.writeFileSync(pathMozilla + '/app.localnative.json', JSON.stringify(jsonMozilla, null, 2))
  }catch(error){
    console.error(error);
  }
}
if (fs.existsSync(pathChromium)){
  try{
    fs.writeFileSync(pathChromium + '/app.localnative.json', JSON.stringify(jsonChrome, null, 2))
  }catch(error){
    console.error(error);
  }
}
if (fs.existsSync(pathChrome)){
  try{
    fs.writeFileSync(pathChrome + '/app.localnative.json', JSON.stringify(jsonChrome, null, 2))
  }catch(error){
    console.error(error);
  }
}

module.exports = {
  dir: __dirname,
  run: addon.run
};

