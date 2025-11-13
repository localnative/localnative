#!/bin/bash
set -e

# Build Rust library for all platforms
# Run this script before building the Flutter app

echo "Building Rust library for Flutter..."

cd rust

# Android targets
echo "Building for Android..."
cargo build --target aarch64-linux-android --release
cargo build --target armv7-linux-androideabi --release
cargo build --target x86_64-linux-android --release
cargo build --target i686-linux-android --release

# Copy Android libraries
echo "Copying Android libraries..."
mkdir -p ../android/app/src/main/jniLibs/arm64-v8a
mkdir -p ../android/app/src/main/jniLibs/armeabi-v7a
mkdir -p ../android/app/src/main/jniLibs/x86_64
mkdir -p ../android/app/src/main/jniLibs/x86

cp target/aarch64-linux-android/release/liblocalnative_flutter.so ../android/app/src/main/jniLibs/arm64-v8a/
cp target/armv7-linux-androideabi/release/liblocalnative_flutter.so ../android/app/src/main/jniLibs/armeabi-v7a/
cp target/x86_64-linux-android/release/liblocalnative_flutter.so ../android/app/src/main/jniLibs/x86_64/
cp target/i686-linux-android/release/liblocalnative_flutter.so ../android/app/src/main/jniLibs/x86/

# iOS targets
echo "Building for iOS..."
cargo build --target aarch64-apple-ios --release
cargo build --target x86_64-apple-ios --release

# Create universal library for iOS
echo "Creating universal iOS library..."
mkdir -p ../ios/Runner/Frameworks
lipo -create \
  target/aarch64-apple-ios/release/liblocalnative_flutter.a \
  target/x86_64-apple-ios/release/liblocalnative_flutter.a \
  -output ../ios/Runner/Frameworks/liblocalnative_flutter.a

# macOS target (for desktop)
echo "Building for macOS..."
cargo build --release

echo "Rust build complete!"
