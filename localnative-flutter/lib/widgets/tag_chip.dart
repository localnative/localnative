import 'package:flutter/material.dart';

/// Tag chip widget for displaying and filtering by tags
class TagChip extends StatelessWidget {
  final String tag;
  final VoidCallback? onTap;
  final bool small;

  const TagChip({
    super.key,
    required this.tag,
    this.onTap,
    this.small = false,
  });

  @override
  Widget build(BuildContext context) {
    if (onTap != null) {
      return ActionChip(
        label: Text(tag),
        labelStyle: TextStyle(fontSize: small ? 12 : 14),
        padding: small ? const EdgeInsets.symmetric(horizontal: 4) : null,
        visualDensity: small ? VisualDensity.compact : null,
        onPressed: onTap,
      );
    }

    return Chip(
      label: Text(tag),
      labelStyle: TextStyle(fontSize: small ? 12 : 14),
      padding: small ? const EdgeInsets.symmetric(horizontal: 4) : null,
      visualDensity: small ? VisualDensity.compact : null,
    );
  }
}
