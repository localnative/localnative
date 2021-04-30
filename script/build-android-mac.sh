BIN_PATH=$HOME/Library/Android/sdk/ndk-bundle/toolchains/llvm/prebuilt/darwin-x86_64/bin
cd `dirname $0`/../localnative-rs/localnative_core

echo 'crate-type = ["cdylib"]' >> Cargo.toml

# cargo clean

PATH=$BIN_PATH:$PATH cargo build --target armv7-linux-androideabi --release
PATH=$BIN_PATH:$PATH cargo build --target i686-linux-android --release
PATH=$BIN_PATH:$PATH cargo build --target aarch64-linux-android --release
PATH=$BIN_PATH:$PATH cargo build --target x86_64-linux-android --release

git reset --hard
