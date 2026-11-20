"use strict";
const fs = require("fs");
const path = require("path");

if (process.platform === "win32") {
  process.exit(0);
}

const target = `${process.platform}-${process.arch}`;
for (const name of ["localnative", "localnative-web-ext-host"]) {
  const relative = path.join("..", "vendor", target, name);
  const binary = path.join(__dirname, "bin", relative);
  if (!fs.existsSync(binary)) {
    continue;
  }
  try {
    fs.chmodSync(binary, 0o755);
    const launcher = path.join(__dirname, "bin", name);
    fs.unlinkSync(launcher);
    fs.symlinkSync(relative, launcher);
  } catch {
    // Keep the Node launcher when the filesystem refuses symlinks.
  }
}
