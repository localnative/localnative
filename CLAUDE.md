# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Local Native is a cross-platform tool for saving and syncing notes in a local SQLite database without going through any centralized service. It supports multiple platforms including desktop (Rust GUI with Iced, Electron, Tauri), mobile (Android, iOS), and browser extensions.

## Architecture

The project is structured as a multi-platform application with a shared Rust core:

### Core Components
- **localnative_core**: Central database operations and RPC functionality using SQLx with SQLite
- **localnative_iced**: Native GUI application using the Iced framework
- **localnative_cli**: Command-line interface

### Database Layer
- Uses SQLx for async database operations with SQLite
- Database migrations handled in `localnative_core/src/db.rs` migrations module
- Database operations include CRUD for notes, search, filtering, and sync functionality
- Notes contain: title, URL, tags, description, comments, annotations (binary data), timestamps, and public/private flags

### RPC/Sync System
- tarpc-based RPC system for peer-to-peer synchronization
- Bi-directional sync between clients using UUID-based conflict resolution
- Server can be started on any client for others to sync with

### Multi-Platform Clients
- **Electron**: Desktop app with Node.js bridge via localnative-neon
- **Tauri**: Modern desktop app with Svelte frontend
- **Android**: Native Android app with JNI bindings
- **iOS**: Native iOS app with Swift interface
- **Browser Extension**: WebExtension with host binary for local database access

## Common Commands

### Rust (Core Development)
```bash
cd localnative-rs

# Build all workspace members
cargo build

# Run GUI application
cargo run -p localnative_iced

# Run CLI
cargo run -p localnative_cli

# Run tests
cargo test

# Check code
cargo check
```

### Electron App
```bash
cd localnative-electron

# Install dependencies
npm install

# Start development
npm run dev

# Build native modules
npm run build
```

### Tauri App
```bash
cd localnative-tauri

# Install dependencies
npm install

# Development mode
npm run dev

# Build
npm run build

# Lint
npm run lint

# Format code
npm run format
```

### Android
```bash
cd localnative-android

# Build debug APK
./gradlew assembleDebug

# Install on device
./gradlew installDebug
```

## Key Development Patterns

### Database Operations
- All database operations are async and use the SQLx connection pool
- Commands are serialized as JSON and processed through the main `process_cmd` function
- Database schema migrations are version-controlled and applied automatically

### Cross-Platform FFI
- C-compatible FFI interface in `localnative_core/src/lib.rs`
- JSON-based message passing between platforms and the Rust core
- Platform-specific bindings (JNI for Android, Swift for iOS, Node.js for Electron)

### Sync Protocol
- Each note has a UUID4 for global identification
- Sync compares UUID lists to determine what needs to be transferred
- Version compatibility checking prevents incompatible sync operations

### Search and Filtering
- Full-text search across title, URL, tags, and description fields
- Date range filtering with visualization
- Tag-based filtering and aggregation

## Development Tips

- The main database logic is in `localnative_core/src/db.rs`
- RPC/sync functionality is in `localnative_core/src/rpc.rs`
- GUI state management is in `localnative_iced/src/lib.rs`
- Cross-platform builds use the `script/` directory for build automation
- The project uses SQLx migrations for database schema changes - run `cargo sqlx migrate run` if needed
- For new database features, add corresponding command variants to the `Cmd` enum in `db/models.rs`