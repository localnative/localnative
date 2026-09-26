#!/bin/sh
# Xcode Cloud post-clone: Pods, then the Rust core XCFramework the project links.

# Install CocoaPods using Homebrew.
brew install cocoapods

# Install dependencies you manage with CocoaPods.
pod install

# Rust toolchain (Xcode Cloud images may not have it).
if ! command -v rustup >/dev/null 2>&1; then
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable
  # shellcheck disable=SC1091
  . "$HOME/.cargo/env"
fi

# Build localnative_core for device + simulator.
../script/build-ios.sh
