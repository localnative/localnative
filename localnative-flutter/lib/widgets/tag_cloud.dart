import 'package:flutter/material.dart';
import 'tag_chip.dart';

/// Tag cloud widget showing all available tags with counts
class TagCloud extends StatelessWidget {
  final List<dynamic> tags; // List of TagCount
  final Function(String) onTagTap;

  const TagCloud({
    super.key,
    required this.tags,
    required this.onTagTap,
  });

  @override
  Widget build(BuildContext context) {
    if (tags.isEmpty) {
      return const SizedBox.shrink();
    }

    return Container(
      padding: const EdgeInsets.all(12),
      margin: const EdgeInsets.symmetric(horizontal: 8),
      decoration: BoxDecoration(
        color: Theme.of(context).colorScheme.surfaceVariant.withOpacity(0.3),
        borderRadius: BorderRadius.circular(8),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Row(
            children: [
              Icon(
                Icons.label_outline,
                size: 16,
                color: Theme.of(context).colorScheme.onSurfaceVariant,
              ),
              const SizedBox(width: 6),
              Text(
                'Tags',
                style: Theme.of(context).textTheme.labelMedium?.copyWith(
                  color: Theme.of(context).colorScheme.onSurfaceVariant,
                ),
              ),
            ],
          ),
          const SizedBox(height: 8),
          Wrap(
            spacing: 8,
            runSpacing: 6,
            children: tags.take(20).map((tagCount) {
              final tagName = tagCount.k as String;
              final count = tagCount.v as int;
              return GestureDetector(
                onTap: () => onTagTap(tagName),
                child: Tooltip(
                  message: '$tagName ($count)',
                  child: TagChip(
                    tag: '$tagName ($count)',
                    onTap: () => onTagTap(tagName),
                  ),
                ),
              );
            }).toList(),
          ),
        ],
      ),
    );
  }
}
