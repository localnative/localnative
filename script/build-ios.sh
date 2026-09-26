#!/bin/sh
# Build the Rust core for iOS (device + simulator) as an XCFramework at
# localnative-ios/LocalNativeCore.xcframework — the path the Xcode project
# links against.
set -e

cd "$(dirname "$0")/.."
ROOT=$(pwd)

rustup target add aarch64-apple-ios aarch64-apple-ios-sim

cd "$ROOT/localnative-rs"
cargo build --release -p localnative_core --target aarch64-apple-ios
cargo build --release -p localnative_core --target aarch64-apple-ios-sim

HEADERS="$ROOT/localnative-ios/LocalNativeCoreHeaders"
mkdir -p "$HEADERS"
cp "$ROOT/localnative-rs/localnative_core/src/localnative-core.h" "$HEADERS/localnative_core.h"

rm -rf "$ROOT/localnative-ios/LocalNativeCore.xcframework"
xcodebuild -create-xcframework \
  -library "$ROOT/localnative-rs/target/aarch64-apple-ios/release/liblocalnative_core.a" -headers "$HEADERS" \
  -library "$ROOT/localnative-rs/target/aarch64-apple-ios-sim/release/liblocalnative_core.a" -headers "$HEADERS" \
  -output "$ROOT/localnative-ios/LocalNativeCore.xcframework"

echo "Built $ROOT/localnative-ios/LocalNativeCore.xcframework"
