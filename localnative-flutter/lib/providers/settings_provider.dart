import 'package:flutter/material.dart';
import '../services/settings_service.dart';

/// Provider for managing app settings
class SettingsProvider with ChangeNotifier {
  final SettingsService _settingsService = SettingsService();

  // State
  ThemeMode _themeMode = ThemeMode.system;
  String _language = 'en';
  int _paginationLimit = 10;
  bool _confirmDelete = true;

  // Getters
  ThemeMode get themeMode => _themeMode;
  String get language => _language;
  int get paginationLimit => _paginationLimit;
  bool get confirmDelete => _confirmDelete;

  /// Initialize settings from storage
  Future<void> init() async {
    await _settingsService.init();
    _themeMode = _settingsService.getThemeMode();
    _language = _settingsService.getLanguage();
    _paginationLimit = _settingsService.getPaginationLimit();
    _confirmDelete = _settingsService.getConfirmDelete();
    notifyListeners();
  }

  /// Set theme mode
  Future<void> setThemeMode(ThemeMode mode) async {
    _themeMode = mode;
    await _settingsService.setThemeMode(mode);
    notifyListeners();
  }

  /// Set language
  Future<void> setLanguage(String languageCode) async {
    _language = languageCode;
    await _settingsService.setLanguage(languageCode);
    notifyListeners();
  }

  /// Set pagination limit
  Future<void> setPaginationLimit(int limit) async {
    _paginationLimit = limit;
    await _settingsService.setPaginationLimit(limit);
    notifyListeners();
  }

  /// Set confirm delete
  Future<void> setConfirmDelete(bool confirm) async {
    _confirmDelete = confirm;
    await _settingsService.setConfirmDelete(confirm);
    notifyListeners();
  }

  /// Toggle theme between light and dark
  Future<void> toggleTheme() async {
    final newMode = _themeMode == ThemeMode.light
        ? ThemeMode.dark
        : ThemeMode.light;
    await setThemeMode(newMode);
  }
}
