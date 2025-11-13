# Local Native Flutter - Setup Guide

Complete setup guide for building and running the Local Native Flutter application.

## Prerequisites

### 1. Flutter SDK
- Flutter 3.0.0 or higher
- Installation: https://docs.flutter.dev/get-started/install

### 2. Rust Toolchain
- Rust 1.70.0 or higher
- Installation: https://rustup.rs/

### 3. Platform-Specific Requirements

#### Android
- Android Studio or Android SDK Command-line Tools
- Android NDK (for cross-compilation)
- Set environment variables:
  ```bash
  export ANDROID_HOME=$HOME/Library/Android/sdk
  export ANDROID_NDK_HOME=$ANDROID_HOME/ndk/<version>
  ```

#### iOS (macOS only)
- Xcode 14.0 or higher
- Xcode Command Line Tools: `xcode-select --install`
- CocoaPods: `sudo gem install cocoapods`

#### macOS Desktop
- Xcode Command Line Tools

## Initial Setup

### 1. Clone and Navigate
```bash
cd localnative-flutter
```

### 2. Install Development Tools
```bash
make setup
```

This command will:
- Install required Rust targets for cross-compilation
- Install flutter_rust_bridge_codegen
- Download Flutter dependencies

### 3. Configure Android NDK (Android only)

Create `~/.cargo/config.toml`:
```toml
[target.aarch64-linux-android]
ar = "<NDK_PATH>/toolchains/llvm/prebuilt/darwin-x86_64/bin/llvm-ar"
linker = "<NDK_PATH>/toolchains/llvm/prebuilt/darwin-x86_64/bin/aarch64-linux-android30-clang"

[target.armv7-linux-androideabi]
ar = "<NDK_PATH>/toolchains/llvm/prebuilt/darwin-x86_64/bin/llvm-ar"
linker = "<NDK_PATH>/toolchains/llvm/prebuilt/darwin-x86_64/bin/armv7a-linux-androideabi30-clang"

[target.i686-linux-android]
ar = "<NDK_PATH>/toolchains/llvm/prebuilt/darwin-x86_64/bin/llvm-ar"
linker = "<NDK_PATH>/toolchains/llvm/prebuilt/darwin-x86_64/bin/i686-linux-android30-clang"

[target.x86_64-linux-android]
ar = "<NDK_PATH>/toolchains/llvm/prebuilt/darwin-x86_64/bin/llvm-ar"
linker = "<NDK_PATH>/toolchains/llvm/prebuilt/darwin-x86_64/bin/x86_64-linux-android30-clang"
```

Replace `<NDK_PATH>` with your actual NDK path (e.g., `$ANDROID_HOME/ndk/25.2.9519653`).

On Linux, replace `darwin-x86_64` with `linux-x86_64`.

## Building

### Generate Bridge Code
Before the first build, generate Dart/Rust bridge bindings:
```bash
make bridge
```

This creates:
- `lib/bridge_generated.dart` - Dart FFI bindings
- `ios/Runner/bridge_generated.h` - C header for iOS

### Build Rust Library
Compile the Rust core for all platforms:
```bash
make build-rust
```

This will:
- Compile Rust code for Android (arm64, armv7, x86_64, x86)
- Compile Rust code for iOS (arm64, x86_64)
- Create universal library for iOS
- Copy libraries to platform directories

### Build Flutter App

#### Android
```bash
flutter build apk
# or for app bundle
flutter build appbundle
```

#### iOS
```bash
cd ios && pod install && cd ..
flutter build ios
```

#### macOS
```bash
flutter build macos
```

## Running

### Android
```bash
make run-android
# or
flutter run -d android
```

### iOS
```bash
make run-ios
# or
flutter run -d ios
```

### macOS
```bash
make run-macos
# or
flutter run -d macos
```

### Development Mode (Hot Reload)
For rapid development with hot reload:
```bash
make dev
```

Note: This skips Rust rebuilds. If you modify Rust code, run `make build-rust` first.

## Troubleshooting

### Bridge Generation Fails
**Problem**: `flutter_rust_bridge_codegen` not found

**Solution**:
```bash
cargo install flutter_rust_bridge_codegen
```

### Android Build Fails - "NDK not found"
**Problem**: Android NDK not configured

**Solution**:
1. Install NDK via Android Studio SDK Manager
2. Set `ANDROID_NDK_HOME` environment variable
3. Configure `~/.cargo/config.toml` (see above)

### iOS Build Fails - "Library not found"
**Problem**: Rust library not built for iOS

**Solution**:
```bash
make build-rust
cd ios && pod install && cd ..
```

### "localnative_core" not found
**Problem**: Rust workspace path incorrect

**Solution**: Verify `rust/Cargo.toml` has correct path to `localnative_core`:
```toml
localnative_core = { path = "../../localnative-rs/localnative_core" }
```

### Hot Reload Not Working for Rust Changes
**Explanation**: Flutter hot reload only works for Dart code. Rust changes require full rebuild.

**Solution**: After modifying Rust code:
```bash
make build-rust
# Then restart the app
flutter run
```

## Development Workflow

### Making Changes

1. **Dart-only changes**: Use hot reload (r in terminal)
2. **Rust changes**:
   ```bash
   # Stop the app
   make build-rust
   flutter run
   ```
3. **Bridge interface changes**:
   ```bash
   make bridge
   make build-rust
   flutter run
   ```

### Clean Build
If experiencing issues:
```bash
make clean
make setup
make bridge
make build-rust
flutter run
```

## Project Structure

```
localnative-flutter/
├── lib/                    # Dart/Flutter code
│   ├── main.dart          # App entry point
│   ├── models/            # Data models
│   ├── providers/         # State management
│   ├── screens/           # UI screens
│   ├── services/          # Business logic
│   └── widgets/           # Reusable widgets
├── rust/                  # Rust bridge layer
│   ├── src/lib.rs         # Bridge implementation
│   └── Cargo.toml         # Rust dependencies
├── android/               # Android platform code
├── ios/                   # iOS platform code
├── scripts/               # Build scripts
└── Makefile               # Build automation
```

## Next Steps

- [ ] Generate bridge code: `make bridge`
- [ ] Build Rust library: `make build-rust`
- [ ] Run on device: `make run-android` or `make run-ios`
- [ ] Start developing!

## Support

For issues related to:
- **Flutter**: https://docs.flutter.dev/
- **Rust**: https://www.rust-lang.org/learn
- **flutter_rust_bridge**: https://github.com/fzyzcjy/flutter_rust_bridge
- **Local Native**: See main project documentation
