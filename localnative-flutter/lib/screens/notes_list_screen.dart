import 'package:flutter/material.dart';
import 'package:provider/provider.dart';

import '../providers/notes_provider.dart';
import '../providers/settings_provider.dart';
import '../widgets/note_card.dart';
import '../widgets/pagination_bar.dart';
import '../widgets/search_bar.dart' as custom;
import '../widgets/tag_cloud.dart';
import '../widgets/add_note_dialog.dart';

/// Main screen for displaying and managing notes
class NotesListScreen extends StatefulWidget {
  const NotesListScreen({super.key});

  @override
  State<NotesListScreen> createState() => _NotesListScreenState();
}

class _NotesListScreenState extends State<NotesListScreen> {
  @override
  void initState() {
    super.initState();
    // Load notes on first render
    WidgetsBinding.instance.addPostFrameCallback((_) {
      final notesProvider = context.read<NotesProvider>();
      final settingsProvider = context.read<SettingsProvider>();
      notesProvider.setLimit(settingsProvider.paginationLimit);
      notesProvider.loadNotes();
    });
  }

  void _showAddNoteDialog() {
    showDialog(
      context: context,
      builder: (context) => const AddNoteDialog(),
    );
  }

  @override
  Widget build(BuildContext context) {
    return Column(
      children: [
        // Search bar
        Padding(
          padding: const EdgeInsets.all(8.0),
          child: custom.SearchBar(
            onSearch: (query) {
              context.read<NotesProvider>().search(query);
            },
          ),
        ),

        // Tag cloud
        Consumer<NotesProvider>(
          builder: (context, notesProvider, _) {
            if (notesProvider.tagCounts.isNotEmpty) {
              return TagCloud(
                tags: notesProvider.tagCounts,
                onTagTap: (tag) {
                  notesProvider.filterByTag(tag);
                },
              );
            }
            return const SizedBox.shrink();
          },
        ),

        // Notes list
        Expanded(
          child: Consumer<NotesProvider>(
            builder: (context, notesProvider, _) {
              if (notesProvider.isLoading) {
                return const Center(
                  child: CircularProgressIndicator(),
                );
              }

              if (notesProvider.errorMessage != null) {
                return Center(
                  child: Column(
                    mainAxisAlignment: MainAxisAlignment.center,
                    children: [
                      const Icon(Icons.error_outline, size: 48, color: Colors.red),
                      const SizedBox(height: 16),
                      Text(
                        'Error: ${notesProvider.errorMessage}',
                        style: const TextStyle(color: Colors.red),
                        textAlign: TextAlign.center,
                      ),
                      const SizedBox(height: 16),
                      ElevatedButton(
                        onPressed: () => notesProvider.refresh(),
                        child: const Text('Retry'),
                      ),
                    ],
                  ),
                );
              }

              if (notesProvider.notes.isEmpty) {
                return Center(
                  child: Column(
                    mainAxisAlignment: MainAxisAlignment.center,
                    children: [
                      const Icon(Icons.note_add_outlined, size: 64, color: Colors.grey),
                      const SizedBox(height: 16),
                      Text(
                        notesProvider.searchQuery.isEmpty
                            ? 'No notes yet'
                            : 'No notes found for "${notesProvider.searchQuery}"',
                        style: Theme.of(context).textTheme.titleMedium?.copyWith(
                          color: Colors.grey,
                        ),
                      ),
                      const SizedBox(height: 16),
                      ElevatedButton.icon(
                        onPressed: _showAddNoteDialog,
                        icon: const Icon(Icons.add),
                        label: const Text('Add Note'),
                      ),
                    ],
                  ),
                );
              }

              return RefreshIndicator(
                onRefresh: () => notesProvider.refresh(),
                child: ListView.builder(
                  itemCount: notesProvider.notes.length,
                  itemBuilder: (context, index) {
                    final note = notesProvider.notes[index];
                    return NoteCard(
                      note: note,
                      onDelete: (rowid) async {
                        final settings = context.read<SettingsProvider>();
                        if (settings.confirmDelete) {
                          final confirmed = await _confirmDelete(context);
                          if (confirmed == true) {
                            await notesProvider.deleteNote(rowid);
                          }
                        } else {
                          await notesProvider.deleteNote(rowid);
                        }
                      },
                    );
                  },
                ),
              );
            },
          ),
        ),

        // Pagination
        Consumer<NotesProvider>(
          builder: (context, notesProvider, _) {
            if (notesProvider.totalCount > 0) {
              return PaginationBar(
                currentPage: notesProvider.currentPage,
                totalPages: notesProvider.totalPages,
                onPrevious: notesProvider.hasPreviousPage
                    ? () => notesProvider.previousPage()
                    : null,
                onNext: notesProvider.hasNextPage
                    ? () => notesProvider.nextPage()
                    : null,
              );
            }
            return const SizedBox.shrink();
          },
        ),
      ],
    );
  }

  Future<bool?> _confirmDelete(BuildContext context) {
    return showDialog<bool>(
      context: context,
      builder: (context) => AlertDialog(
        title: const Text('Delete Note'),
        content: const Text('Are you sure you want to delete this note?'),
        actions: [
          TextButton(
            onPressed: () => Navigator.of(context).pop(false),
            child: const Text('Cancel'),
          ),
          TextButton(
            onPressed: () => Navigator.of(context).pop(true),
            style: TextButton.styleFrom(foregroundColor: Colors.red),
            child: const Text('Delete'),
          ),
        ],
      ),
    );
  }
}
