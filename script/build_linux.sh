#!/bin/bash
cd localnative_iced/
pwd
cargo build --release
cd ../localnative_cli/
pwd
cargo build --release
cd ..
pwd
mkdir localnative_bin_linux
cp target/release/localnative_iced localnative_bin_linux/
cp target/release/localnative-web-ext-host localnative_bin_linux/
tar -zcvf localnative_bin_linux.tar.gz localnative_bin_linux/


