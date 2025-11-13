import 'package:flutter/material.dart';
import 'package:url_launcher/url_launcher.dart';

import '../models/note.dart';
import 'tag_chip.dart';
import 'qr_display.dart';

/// Card widget for displaying a single note
class NoteCard extends StatelessWidget {
  final dynamic note; // Will be Note type from bridge_generated.dart
  final Function(int) onDelete;

  const NoteCard({
    super.key,
    required this.note,
    required this.onDelete,
  });

  @override
  Widget build(BuildContext context) {
    final tags = NoteHelper.parseTags(note.tags as String);
    final formattedDate = NoteHelper.formatCreatedAt(note.createdAt as String);
    final hasUrl = (note.url as String).isNotEmpty;

    return Card(
      margin: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
      child: Padding(
        padding: const EdgeInsets.all(12.0),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            // Title and actions row
            Row(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Expanded(
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      // Title
                      Text(
                        note.title as String,
                        style: Theme.of(context).textTheme.titleMedium?.copyWith(
                          fontWeight: FontWeight.bold,
                        ),
                      ),
                      // URL (if present)
                      if (hasUrl) ...[
                        const SizedBox(height: 4),
                        InkWell(
                          onTap: () => _launchUrl(note.url as String),
                          child: Text(
                            note.url as String,
                            style: TextStyle(
                              color: Theme.of(context).colorScheme.primary,
                              decoration: TextDecoration.underline,
                            ),
                            maxLines: 1,
                            overflow: TextOverflow.ellipsis,
                          ),
                        ),
                      ],
                    ],
                  ),
                ),
                // Actions
                Row(
                  mainAxisSize: MainAxisSize.min,
                  children: [
                    if (hasUrl)
                      IconButton(
                        icon: const Icon(Icons.qr_code),
                        iconSize: 20,
                        padding: EdgeInsets.zero,
                        constraints: const BoxConstraints(),
                        onPressed: () => _showQRCode(context, note.url as String),
                        tooltip: 'Show QR code',
                      ),
                    const SizedBox(width: 8),
                    IconButton(
                      icon: const Icon(Icons.delete_outline),
                      iconSize: 20,
                      color: Colors.red,
                      padding: EdgeInsets.zero,
                      constraints: const BoxConstraints(),
                      onPressed: () => onDelete(note.rowid as int),
                      tooltip: 'Delete note',
                    ),
                  ],
                ),
              ],
            ),

            // Description
            if ((note.description as String).isNotEmpty) ...[
              const SizedBox(height: 8),
              Text(
                note.description as String,
                style: Theme.of(context).textTheme.bodyMedium,
                maxLines: 3,
                overflow: TextOverflow.ellipsis,
              ),
            ],

            // Comments
            if ((note.comments as String).isNotEmpty) ...[
              const SizedBox(height: 8),
              Container(
                padding: const EdgeInsets.all(8),
                decoration: BoxDecoration(
                  color: Theme.of(context).colorScheme.surfaceVariant,
                  borderRadius: BorderRadius.circular(4),
                ),
                child: Text(
                  note.comments as String,
                  style: Theme.of(context).textTheme.bodySmall,
                  maxLines: 2,
                  overflow: TextOverflow.ellipsis,
                ),
              ),
            ],

            // Tags
            if (tags.isNotEmpty) ...[
              const SizedBox(height: 8),
              Wrap(
                spacing: 6,
                runSpacing: 4,
                children: tags.map((tag) => TagChip(tag: tag, small: true)).toList(),
              ),
            ],

            // Footer: date and metadata
            const SizedBox(height: 8),
            Row(
              children: [
                Icon(Icons.access_time, size: 14, color: Colors.grey[600]),
                const SizedBox(width: 4),
                Text(
                  formattedDate,
                  style: Theme.of(context).textTheme.bodySmall?.copyWith(
                    color: Colors.grey[600],
                  ),
                ),
                const Spacer(),
                if (note.isPublic as bool)
                  Chip(
                    label: const Text('Public'),
                    padding: EdgeInsets.zero,
                    visualDensity: VisualDensity.compact,
                    labelStyle: const TextStyle(fontSize: 10),
                  ),
              ],
            ),
          ],
        ),
      ),
    );
  }

  void _launchUrl(String urlString) async {
    final url = Uri.parse(urlString);
    if (await canLaunchUrl(url)) {
      await launchUrl(url, mode: LaunchMode.externalApplication);
    }
  }

  void _showQRCode(BuildContext context, String data) {
    showDialog(
      context: context,
      builder: (context) => AlertDialog(
        title: const Text('QR Code'),
        content: SizedBox(
          width: 280,
          height: 280,
          child: QRDisplay(data: data),
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.of(context).pop(),
            child: const Text('Close'),
          ),
        ],
      ),
    );
  }
}
