import 'package:shared_preferences/shared_preferences.dart';
import 'package:flutter/material.dart';

/// Service for managing app settings using SharedPreferences
class SettingsService {
  static const String _themeKey = 'theme_mode';
  static const String _languageKey = 'language';
  static const String _paginationLimitKey = 'pagination_limit';
  static const String _confirmDeleteKey = 'confirm_delete';

  // Singleton pattern
  static final SettingsService _instance = SettingsService._internal();
  factory SettingsService() => _instance;
  SettingsService._internal();

  SharedPreferences? _prefs;

  /// Initialize the service
  Future<void> init() async {
    _prefs = await SharedPreferences.getInstance();
  }

  /// Get theme mode
  ThemeMode getThemeMode() {
    final themeName = _prefs?.getString(_themeKey) ?? 'system';
    switch (themeName) {
      case 'light':
        return ThemeMode.light;
      case 'dark':
        return ThemeMode.dark;
      default:
        return ThemeMode.system;
    }
  }

  /// Set theme mode
  Future<void> setThemeMode(ThemeMode mode) async {
    String themeName;
    switch (mode) {
      case ThemeMode.light:
        themeName = 'light';
        break;
      case ThemeMode.dark:
        themeName = 'dark';
        break;
      default:
        themeName = 'system';
    }
    await _prefs?.setString(_themeKey, themeName);
  }

  /// Get language code
  String getLanguage() {
    return _prefs?.getString(_languageKey) ?? 'en';
  }

  /// Set language code
  Future<void> setLanguage(String languageCode) async {
    await _prefs?.setString(_languageKey, languageCode);
  }

  /// Get pagination limit
  int getPaginationLimit() {
    return _prefs?.getInt(_paginationLimitKey) ?? 10;
  }

  /// Set pagination limit
  Future<void> setPaginationLimit(int limit) async {
    await _prefs?.setInt(_paginationLimitKey, limit);
  }

  /// Get confirm delete setting
  bool getConfirmDelete() {
    return _prefs?.getBool(_confirmDeleteKey) ?? true;
  }

  /// Set confirm delete setting
  Future<void> setConfirmDelete(bool confirm) async {
    await _prefs?.setBool(_confirmDeleteKey, confirm);
  }
}
