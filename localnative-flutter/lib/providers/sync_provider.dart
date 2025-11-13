import 'package:flutter/foundation.dart';
import '../services/database_service.dart';

enum SyncMode {
  idle,
  server,
  client,
}

enum SyncStatus {
  idle,
  starting,
  running,
  syncing,
  success,
  error,
}

/// Provider for managing sync operations
///
/// Handles RPC server/client mode for peer-to-peer synchronization.
class SyncProvider with ChangeNotifier {
  final DatabaseService _db = DatabaseService();

  // State
  SyncMode _mode = SyncMode.idle;
  SyncStatus _status = SyncStatus.idle;
  String _serverAddress = '0.0.0.0:2345';
  String _clientAddress = '';
  String? _message;
  String? _errorMessage;

  // Getters
  SyncMode get mode => _mode;
  SyncStatus get status => _status;
  String get serverAddress => _serverAddress;
  String get clientAddress => _clientAddress;
  String? get message => _message;
  String? get errorMessage => _errorMessage;

  bool get isServerRunning => _mode == SyncMode.server && _status == SyncStatus.running;
  bool get isIdle => _status == SyncStatus.idle;

  /// Start RPC server
  Future<bool> startServer({String? address}) async {
    if (address != null) {
      _serverAddress = address;
    }

    _status = SyncStatus.starting;
    _errorMessage = null;
    _message = 'Starting server on $_serverAddress...';
    notifyListeners();

    try {
      final result = await _db.startServer(addr: _serverAddress);
      _mode = SyncMode.server;
      _status = SyncStatus.running;
      _message = 'Server running on $_serverAddress\n$result';
      notifyListeners();
      return true;
    } catch (e) {
      _status = SyncStatus.error;
      _errorMessage = 'Failed to start server: $e';
      _message = null;
      notifyListeners();
      return false;
    }
  }

  /// Stop RPC server
  Future<bool> stopServer() async {
    if (_mode != SyncMode.server) {
      return false;
    }

    try {
      await _db.stopServer(addr: _serverAddress);
      _mode = SyncMode.idle;
      _status = SyncStatus.idle;
      _message = null;
      notifyListeners();
      return true;
    } catch (e) {
      _errorMessage = 'Failed to stop server: $e';
      notifyListeners();
      return false;
    }
  }

  /// Sync with remote server
  Future<bool> syncWithServer(String address) async {
    _clientAddress = address;
    _mode = SyncMode.client;
    _status = SyncStatus.syncing;
    _errorMessage = null;
    _message = 'Syncing with $address...';
    notifyListeners();

    try {
      final result = await _db.syncWithServer(addr: address);
      _status = SyncStatus.success;
      _message = 'Sync completed successfully\n$result';
      notifyListeners();

      // Return to idle after a delay
      Future.delayed(const Duration(seconds: 2), () {
        _mode = SyncMode.idle;
        _status = SyncStatus.idle;
        _message = null;
        notifyListeners();
      });

      return true;
    } catch (e) {
      _status = SyncStatus.error;
      _errorMessage = 'Sync failed: $e';
      _message = null;
      notifyListeners();
      return false;
    }
  }

  /// Set server address
  void setServerAddress(String address) {
    _serverAddress = address;
    notifyListeners();
  }

  /// Set client address
  void setClientAddress(String address) {
    _clientAddress = address;
    notifyListeners();
  }

  /// Clear error message
  void clearError() {
    _errorMessage = null;
    notifyListeners();
  }

  /// Clear message
  void clearMessage() {
    _message = null;
    notifyListeners();
  }

  /// Reset to idle state
  void reset() {
    _mode = SyncMode.idle;
    _status = SyncStatus.idle;
    _message = null;
    _errorMessage = null;
    notifyListeners();
  }
}
