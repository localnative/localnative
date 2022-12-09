#!/bin/sh

# Build liblocalnative_core.a
cargo install cargo-lipo
rustup target add aarch64-apple-ios x86_64-apple-ios
../../script/build-ios.sh

# Install CocoaPods using Homebrew.
brew install cocoapods

# Install dependencies you manage with CocoaPods.
pod install
