cd "$(dirname "${BASH_SOURCE[0]}")/.."

set -e
export ANDROID_NDK_ROOT=$HOME/Library/Android/sdk/ndk/29.0.14033849
cd `dirname $0`/../localnative-rs/localnative_core

cargo ndk -t armeabi-v7a build --release
cp ../target/aarch64-linux-android/release/liblocalnative_core.so ../../localnative-android/app/src/main/jniLibs/armeabi-v7a/liblocalnative_core.so

cargo ndk -t arm64-v8a build --release
cp ../target/aarch64-linux-android/release/liblocalnative_core.so ../../localnative-android/app/src/main/jniLibs/arm64-v8a/liblocalnative_core.so

cargo ndk -t x86 build --release
cp ../target/i686-linux-android/release/liblocalnative_core.so ../../localnative-android/app/src/main/jniLibs/x86/liblocalnative_core.so

cargo ndk -t x86_64 build --release
cp ../target/x86_64-linux-android/release/liblocalnative_core.so ../../localnative-android/app/src/main/jniLibs/x86_64/liblocalnative_core.so
