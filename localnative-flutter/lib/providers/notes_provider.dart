import 'package:flutter/foundation.dart';
import '../services/database_service.dart';

/// Provider for managing notes state
///
/// Handles CRUD operations, search, pagination, and filtering.
class NotesProvider with ChangeNotifier {
  final DatabaseService _db = DatabaseService();

  // State
  List<dynamic> _notes = [];
  int _totalCount = 0;
  int _currentOffset = 0;
  int _limit = 10;
  String _searchQuery = '';
  List<dynamic> _dayCounts = [];
  List<dynamic> _tagCounts = [];
  bool _isLoading = false;
  String? _errorMessage;

  // Date filter
  String? _filterFromDate;
  String? _filterToDate;

  // Getters
  List<dynamic> get notes => _notes;
  int get totalCount => _totalCount;
  int get currentOffset => _currentOffset;
  int get limit => _limit;
  String get searchQuery => _searchQuery;
  List<dynamic> get dayCounts => _dayCounts;
  List<dynamic> get tagCounts => _tagCounts;
  bool get isLoading => _isLoading;
  String? get errorMessage => _errorMessage;
  String? get filterFromDate => _filterFromDate;
  String? get filterToDate => _filterToDate;

  int get currentPage => (_currentOffset / _limit).floor() + 1;
  int get totalPages => (_totalCount / _limit).ceil();
  bool get hasNextPage => _currentOffset + _limit < _totalCount;
  bool get hasPreviousPage => _currentOffset > 0;

  /// Load notes with current filters
  Future<void> loadNotes() async {
    _isLoading = true;
    _errorMessage = null;
    notifyListeners();

    try {
      dynamic response;

      if (_filterFromDate != null && _filterToDate != null) {
        // Date filtered
        response = await _db.filterNotes(
          query: _searchQuery,
          from: _filterFromDate!,
          to: _filterToDate!,
          limit: _limit,
          offset: _currentOffset,
        );
      } else if (_searchQuery.isNotEmpty) {
        // Search query
        response = await _db.searchNotes(
          query: _searchQuery,
          limit: _limit,
          offset: _currentOffset,
        );
      } else {
        // Default: select all
        response = await _db.selectNotes(
          limit: _limit,
          offset: _currentOffset,
        );
      }

      _notes = response.notes;
      _totalCount = response.count;
      _dayCounts = response.days;
      _tagCounts = response.tags;
    } catch (e) {
      _errorMessage = e.toString();
    } finally {
      _isLoading = false;
      notifyListeners();
    }
  }

  /// Set search query and reload
  Future<void> search(String query) async {
    _searchQuery = query;
    _currentOffset = 0;
    await loadNotes();
  }

  /// Set pagination limit
  Future<void> setLimit(int newLimit) async {
    _limit = newLimit;
    _currentOffset = 0;
    await loadNotes();
  }

  /// Go to next page
  Future<void> nextPage() async {
    if (hasNextPage) {
      _currentOffset += _limit;
      await loadNotes();
    }
  }

  /// Go to previous page
  Future<void> previousPage() async {
    if (hasPreviousPage) {
      _currentOffset = (_currentOffset - _limit).clamp(0, _totalCount);
      await loadNotes();
    }
  }

  /// Go to specific page
  Future<void> goToPage(int page) async {
    final newOffset = (page - 1) * _limit;
    if (newOffset >= 0 && newOffset < _totalCount) {
      _currentOffset = newOffset;
      await loadNotes();
    }
  }

  /// Filter by date range
  Future<void> filterByDateRange(String from, String to) async {
    _filterFromDate = from;
    _filterToDate = to;
    _currentOffset = 0;
    await loadNotes();
  }

  /// Clear date filter
  Future<void> clearDateFilter() async {
    _filterFromDate = null;
    _filterToDate = null;
    _currentOffset = 0;
    await loadNotes();
  }

  /// Filter by tag (sets search query to tag)
  Future<void> filterByTag(String tag) async {
    await search(tag);
  }

  /// Insert a new note
  Future<bool> insertNote({
    required String title,
    required String url,
    required String tags,
    required String description,
    String comments = '',
    String annotations = '',
    bool isPublic = false,
  }) async {
    _isLoading = true;
    _errorMessage = null;
    notifyListeners();

    try {
      final response = await _db.insertNote(
        title: title,
        url: url,
        tags: tags,
        description: description,
        comments: comments,
        annotations: annotations,
        isPublic: isPublic,
        limit: _limit,
        offset: _currentOffset,
      );

      _notes = response.notes;
      _totalCount = response.count;
      _dayCounts = response.days;
      _tagCounts = response.tags;

      return true;
    } catch (e) {
      _errorMessage = e.toString();
      return false;
    } finally {
      _isLoading = false;
      notifyListeners();
    }
  }

  /// Delete a note
  Future<bool> deleteNote(int rowid) async {
    _isLoading = true;
    _errorMessage = null;
    notifyListeners();

    try {
      final response = await _db.deleteNote(
        rowid: rowid,
        limit: _limit,
        offset: _currentOffset,
      );

      _notes = response.notes;
      _totalCount = response.count;
      _dayCounts = response.days;
      _tagCounts = response.tags;

      // Adjust offset if we deleted the last item on a page
      if (_notes.isEmpty && _currentOffset > 0) {
        _currentOffset = (_currentOffset - _limit).clamp(0, _totalCount);
        await loadNotes();
      }

      return true;
    } catch (e) {
      _errorMessage = e.toString();
      return false;
    } finally {
      _isLoading = false;
      notifyListeners();
    }
  }

  /// Refresh notes (reload current view)
  Future<void> refresh() async {
    await loadNotes();
  }
}
