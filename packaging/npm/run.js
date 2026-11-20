"use strict";
const { spawnSync } = require("child_process");
const path = require("path");

module.exports = function run(entry) {
  const name = path.basename(entry).replace(/\.(js|cmd|exe)$/i, "");
  const extension = process.platform === "win32" ? ".exe" : "";
  const target = `${process.platform}-${process.arch}`;
  const binary = path.join(__dirname, "vendor", target, `${name}${extension}`);
  const result = spawnSync(binary, process.argv.slice(2), {
    stdio: "inherit",
  });
  if (result.error) {
    console.error(
      `localnative: no prebuilt ${name} binary for ${target} (${result.error.message})`,
    );
    process.exit(1);
  }
  if (result.signal) {
    process.kill(process.pid, result.signal);
  }
  process.exit(result.status === null ? 1 : result.status);
};
