#!/bin/bash
cd ../localnative-rs/
pwd
cargo build --bin localnative_iced --release
cargo build --bin localnative-web-ext-host --release
cd target/release
pwd
mkdir localnative_linux_bin
cp localnative_iced localnative_linux_bin/
cp localnative-web-ext-host localnative_linux_bin/
cp ../../LICENSE localnative_linux_bin/
cp ../../README.md localnative_linux_bin/
cp -R ../../locales localnative_linux_bin/
tar -zcvf localnative_linux_bin.tar.gz localnative_linux_bin/
