/// Helper class for chart data visualization
/// Works with DayCount and TagCount from flutter_rust_bridge

class ChartHelper {
  /// Parse date string from DayCount
  static DateTime parseDate(String dateStr) {
    try {
      return DateTime.parse(dateStr);
    } catch (e) {
      return DateTime.now();
    }
  }

  /// Get date range from list of day counts
  static DateTimeRange? getDateRange(List<dynamic> dayCounts) {
    if (dayCounts.isEmpty) return null;

    DateTime? earliest;
    DateTime? latest;

    for (var dayCount in dayCounts) {
      final date = parseDate(dayCount.k as String);
      if (earliest == null || date.isBefore(earliest)) {
        earliest = date;
      }
      if (latest == null || date.isAfter(latest)) {
        latest = date;
      }
    }

    if (earliest != null && latest != null) {
      return DateTimeRange(start: earliest, end: latest);
    }

    return null;
  }

  /// Find max count for chart scaling
  static int getMaxCount(List<dynamic> counts) {
    if (counts.isEmpty) return 0;
    return counts.map((c) => c.v as int).reduce((a, b) => a > b ? a : b);
  }
}

class DateTimeRange {
  final DateTime start;
  final DateTime end;

  DateTimeRange({required this.start, required this.end});

  Duration get duration => end.difference(start);
}
