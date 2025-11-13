# Local Native Flutter - Architecture

This document explains the architecture and design decisions for the Flutter implementation of Local Native.

## Overview

The Flutter app follows a layered architecture:
```
┌─────────────────────────────────────────┐
│           UI Layer (Widgets)            │
│  Screens, Widgets, Material Design      │
└──────────────┬──────────────────────────┘
               │
┌──────────────▼──────────────────────────┐
│      State Management (Provider)        │
│  NotesProvider, SyncProvider, Settings  │
└──────────────┬──────────────────────────┘
               │
┌──────────────▼──────────────────────────┐
│        Services Layer (Dart)            │
│  DatabaseService, SettingsService       │
└──────────────┬──────────────────────────┘
               │
┌──────────────▼──────────────────────────┐
│   Rust Bridge (flutter_rust_bridge)    │
│  Type-safe FFI bindings (auto-gen)     │
└──────────────┬──────────────────────────┘
               │
┌──────────────▼──────────────────────────┐
│      Rust Wrapper (localnative_flutter) │
│  Wraps localnative_core with bridge    │
└──────────────┬──────────────────────────┘
               │
┌──────────────▼──────────────────────────┐
│     Core Logic (localnative_core)       │
│  Database, RPC, SQLite operations       │
└─────────────────────────────────────────┘
```

## Key Components

### 1. UI Layer (`lib/screens`, `lib/widgets`)

**Responsibility**: Render UI and handle user interactions

**Key Screens**:
- `HomeScreen`: Main navigation container with bottom bar
- `NotesListScreen`: Display notes with search, pagination, filtering
- `SyncScreen`: P2P sync interface (server/client modes)
- `SettingsScreen`: App configuration

**Key Widgets**:
- `NoteCard`: Individual note display with actions
- `TagCloud`: Tag visualization and filtering
- `SearchBar`: Debounced search input
- `PaginationBar`: Page navigation
- `QRDisplay`: QR code generation/display
- `AddNoteDialog`: Note creation form

**Design System**: Material Design 3 with light/dark theme support

### 2. State Management (`lib/providers`)

**Pattern**: Provider pattern (official Flutter recommendation)

**Providers**:

#### NotesProvider
- Manages note list state
- Handles CRUD operations
- Implements search, filter, pagination logic
- Exposes loading/error states

```dart
class NotesProvider with ChangeNotifier {
  List<dynamic> _notes;
  int _totalCount;
  String _searchQuery;
  // ... pagination, filtering state

  Future<void> loadNotes() { }
  Future<void> search(String query) { }
  Future<bool> insertNote(...) { }
  Future<bool> deleteNote(int rowid) { }
}
```

#### SyncProvider
- Manages sync state (idle, server, client modes)
- Controls RPC server lifecycle
- Handles sync operations

```dart
class SyncProvider with ChangeNotifier {
  SyncMode _mode; // idle, server, client
  SyncStatus _status; // idle, starting, running, syncing, success, error

  Future<bool> startServer({String? address}) { }
  Future<bool> syncWithServer(String address) { }
}
```

#### SettingsProvider
- Manages app settings (theme, language, pagination)
- Persists to SharedPreferences

### 3. Services Layer (`lib/services`)

**Responsibility**: Business logic and external integrations

#### DatabaseService
Wraps Rust bridge functions with Dart-friendly interface:
```dart
class DatabaseService {
  Future<NotesResponse> selectNotes({int limit, int offset});
  Future<NotesResponse> searchNotes({String query, ...});
  Future<NotesResponse> insertNote({...});
  Future<NotesResponse> deleteNote({int rowid, ...});
  Future<NotesResponse> filterNotes({String from, String to, ...});
  Future<String> startServer({String addr});
  Future<String> syncWithServer({String addr});
}
```

#### SettingsService
Manages persistent settings using SharedPreferences:
```dart
class SettingsService {
  ThemeMode getThemeMode();
  Future<void> setThemeMode(ThemeMode mode);
  int getPaginationLimit();
  Future<void> setPaginationLimit(int limit);
}
```

### 4. Rust Bridge Layer (`rust/src/lib.rs`)

**Technology**: flutter_rust_bridge v2.0

**Purpose**: Type-safe FFI between Dart and Rust

**Architecture**:
```
Dart (lib/bridge_generated.dart)
     ↕ [Type-safe FFI calls]
Rust Bridge (rust/src/lib.rs)
     ↕ [JSON-based C FFI]
Core (localnative_core)
```

**Key Functions** (annotated with `#[frb]`):
```rust
#[frb(sync)]
pub fn select_notes(limit: i64, offset: i64) -> Result<NotesResponse, String>

#[frb(sync)]
pub fn search_notes(query: String, limit: i64, offset: i64) -> Result<NotesResponse, String>

#[frb(sync)]
pub fn insert_note(...) -> Result<NotesResponse, String>

#[frb(sync)]
pub fn start_server(addr: String) -> Result<String, String>
```

**Data Flow**:
1. Dart calls type-safe bridge function
2. Bridge serializes to JSON
3. Calls C FFI `localnative_run(json)`
4. Core processes and returns JSON
5. Bridge deserializes to Rust types
6. Returns to Dart as type-safe objects

### 5. Core Logic (`localnative_core`)

**Language**: Rust

**Database**: SQLite via SQLx (async)

**Key Features**:
- CRUD operations on notes
- Full-text search
- Tag management and aggregation
- Date-based filtering
- UUID-based sync protocol
- P2P RPC server (tarpc)

## Data Models

### Note
```dart
class Note {
  int rowid;           // Auto-increment ID
  String uuid4;        // UUID for sync
  String title;        // Note title
  String url;          // Associated URL
  String tags;         // Comma-separated tags
  String description;  // Main content
  String comments;     // Additional notes
  String annotations;  // Binary data (images)
  String createdAt;    // Timestamp
  bool isPublic;       // Visibility flag
}
```

### NotesResponse
```dart
class NotesResponse {
  int count;                 // Total count
  List<Note> notes;          // Page of notes
  List<DayCount> days;       // Date histogram
  List<TagCount> tags;       // Tag frequencies
}
```

## Build Process

### 1. Code Generation
```bash
flutter_rust_bridge_codegen
```
Generates:
- `lib/bridge_generated.dart` - Dart FFI bindings
- `ios/Runner/bridge_generated.h` - C header for iOS

### 2. Rust Compilation

#### Android (Cross-compilation)
```bash
cargo build --target aarch64-linux-android --release
# → liblocalnative_flutter.so for arm64-v8a
```

Libraries copied to `android/app/src/main/jniLibs/<abi>/`

#### iOS (Cross-compilation + Universal Binary)
```bash
cargo build --target aarch64-apple-ios --release
cargo build --target x86_64-apple-ios --release
lipo -create ... -output liblocalnative_flutter.a
```

Universal library placed in `ios/Runner/Frameworks/`

### 3. Flutter Build
```bash
flutter build apk    # Android
flutter build ios    # iOS
flutter build macos  # macOS
```

## Platform Integration

### Android
**JNI Loading**: `MainActivity.kt` loads `liblocalnative_flutter.so`
```kotlin
init {
    System.loadLibrary("localnative_flutter")
}
```

**Permissions**: `AndroidManifest.xml`
- INTERNET (for sync)
- CAMERA (for QR scanning)

### iOS
**Static Linking**: Xcode links `liblocalnative_flutter.a`

**Bridging Header**: `Runner-Bridging-Header.h`
```c
#include "bridge_generated.h"
```

**Permissions**: `Info.plist`
- NSCameraUsageDescription
- NSLocalNetworkUsageDescription

## State Flow Example: Adding a Note

```
User taps "Add Note"
       ↓
AddNoteDialog shown (widget)
       ↓
User fills form, taps "Add"
       ↓
NotesProvider.insertNote() called
       ↓
DatabaseService.insertNote() called
       ↓
Rust bridge: insert_note() called
       ↓
JSON command created: {"action": "insert", ...}
       ↓
C FFI: localnative_run(json)
       ↓
Core processes INSERT into SQLite
       ↓
Returns JSON with updated notes
       ↓
Bridge deserializes to NotesResponse
       ↓
NotesProvider updates state
       ↓
NotesProvider.notifyListeners()
       ↓
UI rebuilds with new note
```

## Sync Protocol

### P2P Synchronization

**Server Mode**:
1. User taps "Start Server"
2. SyncProvider.startServer() called
3. Rust starts tarpc server on specified port
4. QR code generated with server address
5. Server listens for connections

**Client Mode**:
1. User scans QR code or enters address
2. SyncProvider.syncWithServer(address) called
3. Rust client connects to server
4. Exchange UUID lists
5. Bidirectional note transfer
6. UUID-based deduplication

## Performance Considerations

### Pagination
- Default page size: 10 notes
- Configurable in settings (5, 10, 20, 50, 100)
- Offset-based pagination

### Search Debouncing
- 500ms debounce on search input
- Prevents excessive database queries
- Implemented in `SearchBar` widget

### Async Operations
- All database operations are async
- Loading states shown during operations
- Error handling with user-friendly messages

### FFI Overhead
- Bridge calls are synchronous (blocking)
- Database operations are async in Rust core
- FFI marshaling is minimal (JSON serialization)

## Security

### Database
- Local SQLite database
- No cloud storage
- User controls data location

### Network
- P2P sync only (no central server)
- Local network recommended
- User must manually enter/scan server address

### Permissions
- Camera: Only for QR scanning
- Network: Only for P2P sync
- No location, contacts, or other sensitive permissions

## Testing

### Unit Tests
- Test providers with mock services
- Test models and helper functions
- Test widget behavior

### Integration Tests
- Test full user flows
- Test FFI bridge integration
- Test sync protocol

### Platform Tests
- Test on physical Android devices
- Test on iOS simulators and devices
- Test on macOS desktop

## Future Enhancements

### Potential Improvements
1. **Offline-first**: Queue sync operations when offline
2. **Conflict Resolution**: Handle concurrent edits gracefully
3. **Image Optimization**: Compress annotations before storage
4. **Export**: Export notes to various formats (JSON, Markdown, PDF)
5. **Import**: Import from other note apps
6. **Encryption**: End-to-end encryption for sync
7. **Search**: More advanced search with filters, sorting
8. **Tags**: Hierarchical tags, tag suggestions
9. **Backup**: Automated backup to cloud storage (optional)
10. **Collaboration**: Multi-user editing with conflict resolution

### Code Quality
- Add comprehensive test coverage
- Implement CI/CD pipeline
- Add code documentation
- Performance profiling and optimization
- Accessibility improvements (screen readers, high contrast)

## Contributing

When contributing, please:
1. Follow Dart style guide for Flutter code
2. Follow Rust style guide for bridge code
3. Update documentation for architectural changes
4. Add tests for new features
5. Ensure cross-platform compatibility
6. Test on both Android and iOS before submitting

## License

Same license as the main Local Native project.
