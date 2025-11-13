import 'package:flutter/material.dart';
import 'package:provider/provider.dart';

import '../providers/settings_provider.dart';
import '../providers/notes_provider.dart';

/// Settings screen for app configuration
class SettingsScreen extends StatelessWidget {
  const SettingsScreen({super.key});

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: const Text('Settings'),
      ),
      body: Consumer<SettingsProvider>(
        builder: (context, settings, _) {
          return ListView(
            children: [
              // Appearance section
              _buildSectionHeader(context, 'Appearance'),
              ListTile(
                leading: const Icon(Icons.brightness_6),
                title: const Text('Theme'),
                subtitle: Text(_getThemeModeText(settings.themeMode)),
                trailing: SegmentedButton<ThemeMode>(
                  segments: const [
                    ButtonSegment(
                      value: ThemeMode.light,
                      icon: Icon(Icons.light_mode, size: 16),
                    ),
                    ButtonSegment(
                      value: ThemeMode.dark,
                      icon: Icon(Icons.dark_mode, size: 16),
                    ),
                    ButtonSegment(
                      value: ThemeMode.system,
                      icon: Icon(Icons.auto_mode, size: 16),
                    ),
                  ],
                  selected: {settings.themeMode},
                  onSelectionChanged: (Set<ThemeMode> newSelection) {
                    settings.setThemeMode(newSelection.first);
                  },
                ),
              ),
              const Divider(),

              // Pagination section
              _buildSectionHeader(context, 'Display'),
              ListTile(
                leading: const Icon(Icons.format_list_numbered),
                title: const Text('Items per page'),
                subtitle: Text('${settings.paginationLimit} items'),
                trailing: DropdownButton<int>(
                  value: settings.paginationLimit,
                  items: [5, 10, 20, 50, 100].map((int value) {
                    return DropdownMenuItem<int>(
                      value: value,
                      child: Text('$value'),
                    );
                  }).toList(),
                  onChanged: (int? newValue) async {
                    if (newValue != null) {
                      await settings.setPaginationLimit(newValue);
                      // Update notes provider with new limit
                      if (context.mounted) {
                        await context.read<NotesProvider>().setLimit(newValue);
                      }
                    }
                  },
                ),
              ),
              const Divider(),

              // Behavior section
              _buildSectionHeader(context, 'Behavior'),
              SwitchListTile(
                secondary: const Icon(Icons.delete_outline),
                title: const Text('Confirm before deleting'),
                subtitle: const Text('Show confirmation dialog when deleting notes'),
                value: settings.confirmDelete,
                onChanged: (bool value) {
                  settings.setConfirmDelete(value);
                },
              ),
              const Divider(),

              // Language section
              _buildSectionHeader(context, 'Language'),
              ListTile(
                leading: const Icon(Icons.language),
                title: const Text('Language'),
                subtitle: Text(_getLanguageText(settings.language)),
                trailing: DropdownButton<String>(
                  value: settings.language,
                  items: const [
                    DropdownMenuItem(value: 'en', child: Text('English')),
                    DropdownMenuItem(value: 'es', child: Text('Español')),
                    DropdownMenuItem(value: 'fr', child: Text('Français')),
                    DropdownMenuItem(value: 'de', child: Text('Deutsch')),
                    DropdownMenuItem(value: 'zh', child: Text('中文')),
                  ],
                  onChanged: (String? newValue) {
                    if (newValue != null) {
                      settings.setLanguage(newValue);
                    }
                  },
                ),
              ),
              const Divider(),

              // About section
              _buildSectionHeader(context, 'About'),
              const ListTile(
                leading: Icon(Icons.info_outline),
                title: Text('Version'),
                subtitle: Text('1.0.0+1'),
              ),
              const ListTile(
                leading: Icon(Icons.code),
                title: Text('Local Native Flutter'),
                subtitle: Text('Cross-platform note management application'),
              ),
            ],
          );
        },
      ),
    );
  }

  Widget _buildSectionHeader(BuildContext context, String title) {
    return Padding(
      padding: const EdgeInsets.fromLTRB(16, 16, 16, 8),
      child: Text(
        title.toUpperCase(),
        style: Theme.of(context).textTheme.labelSmall?.copyWith(
          color: Theme.of(context).colorScheme.primary,
          fontWeight: FontWeight.bold,
        ),
      ),
    );
  }

  String _getThemeModeText(ThemeMode mode) {
    switch (mode) {
      case ThemeMode.light:
        return 'Light';
      case ThemeMode.dark:
        return 'Dark';
      case ThemeMode.system:
        return 'System default';
    }
  }

  String _getLanguageText(String code) {
    switch (code) {
      case 'en':
        return 'English';
      case 'es':
        return 'Español';
      case 'fr':
        return 'Français';
      case 'de':
        return 'Deutsch';
      case 'zh':
        return '中文';
      default:
        return 'English';
    }
  }
}
