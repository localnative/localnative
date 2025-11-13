# Local Native Flutter

Cross-platform Flutter application for Local Native note management.

## Prerequisites

- Flutter SDK (>= 3.0.0)
- Rust toolchain
- flutter_rust_bridge_codegen CLI tool

## Setup

### 1. Install flutter_rust_bridge_codegen

```bash
cargo install flutter_rust_bridge_codegen
```

### 2. Generate Dart bindings

```bash
flutter_rust_bridge_codegen
```

### 3. Install Flutter dependencies

```bash
flutter pub get
```

### 4. Build Rust library

#### Android
```bash
cargo build --target aarch64-linux-android --release
cargo build --target armv7-linux-androideabi --release
cargo build --target x86_64-linux-android --release
```

#### iOS
```bash
cargo build --target aarch64-apple-ios --release
cargo build --target x86_64-apple-ios --release
```

#### Desktop (macOS)
```bash
cargo build --release
```

## Running

```bash
# Android
flutter run -d android

# iOS
flutter run -d ios

# macOS
flutter run -d macos

# Linux
flutter run -d linux
```

## Building

```bash
# Android APK
flutter build apk

# iOS IPA
flutter build ios

# macOS app
flutter build macos
```

## Architecture

- **lib/models**: Data models (Note, Response, etc.)
- **lib/services**: Business logic layer (database, sync, settings)
- **lib/providers**: State management using Provider pattern
- **lib/screens**: Main application screens
- **lib/widgets**: Reusable UI components
- **rust/**: Rust FFI bridge to localnative_core

## Features

- CRUD operations for notes
- Full-text search across title, URL, tags, and description
- Tag-based filtering
- Date range filtering with visualization
- Pagination support
- P2P synchronization via RPC
- QR code generation and scanning
- Dark/light theme support
- Cross-platform (Android, iOS, macOS, Linux, Windows)
