#!/bin/bash
set -e

echo "Setting up Rust targets for cross-compilation..."

# Android targets
rustup target add aarch64-linux-android
rustup target add armv7-linux-androideabi
rustup target add x86_64-linux-android
rustup target add i686-linux-android

# iOS targets
rustup target add aarch64-apple-ios
rustup target add x86_64-apple-ios

# Desktop targets
rustup target add aarch64-apple-darwin
rustup target add x86_64-apple-darwin

echo "Rust targets installed successfully!"
echo ""
echo "Next steps:"
echo "1. Install flutter_rust_bridge_codegen: cargo install flutter_rust_bridge_codegen"
echo "2. Generate bridge code: flutter_rust_bridge_codegen"
echo "3. Build Rust library: ./scripts/build-rust.sh"
echo "4. Run Flutter app: flutter run"
