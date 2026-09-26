#!/bin/sh
# Build the desktop binaries (Iced GUI + browser-extension host) on Linux.
set -e

cd "$(dirname "$0")/.."
ROOT=$(pwd)

cd "$ROOT/localnative-rs"
cargo build --bin localnative_iced --release
cargo build --bin localnative-web-ext-host --release

cd target/release
OUT=localnative_linux_bin
rm -rf "$OUT" "$OUT.tar.gz"
mkdir "$OUT"
cp localnative_iced "$OUT/"
cp localnative-web-ext-host "$OUT/"
cp "$ROOT/LICENSE" "$OUT/"
cp "$ROOT/README.md" "$OUT/"
cp -R "$ROOT/localnative-rs/locales" "$OUT/"
tar -zcvf "$OUT.tar.gz" "$OUT"
echo "Wrote $PWD/$OUT.tar.gz"
