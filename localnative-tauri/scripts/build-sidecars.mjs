import { copyFileSync, mkdirSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { execFileSync } from "node:child_process";

const projectRoot = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const rustWorkspace = resolve(projectRoot, "../localnative-rs");
const targetTriple = execFileSync("rustc", ["--print", "host-tuple"], {
  encoding: "utf8",
}).trim();
const extension = process.platform === "win32" ? ".exe" : "";

execFileSync(
  "cargo",
  [
    "build",
    "--manifest-path",
    resolve(rustWorkspace, "Cargo.toml"),
    "--release",
    "--bin",
    "localnative",
    "--bin",
    "localnative-web-ext-host",
  ],
  { stdio: "inherit" },
);

const destination = resolve(projectRoot, "src-tauri/binaries");
mkdirSync(destination, { recursive: true });
for (const name of ["localnative", "localnative-web-ext-host"]) {
  copyFileSync(
    resolve(rustWorkspace, `target/release/${name}${extension}`),
    resolve(destination, `${name}-${targetTriple}${extension}`),
  );
}
