import 'package:flutter/material.dart';
import 'package:qr_flutter/qr_flutter.dart';

/// Widget for displaying QR codes
class QRDisplay extends StatelessWidget {
  final String data;
  final double size;

  const QRDisplay({
    super.key,
    required this.data,
    this.size = 200,
  });

  @override
  Widget build(BuildContext context) {
    return Center(
      child: Container(
        padding: const EdgeInsets.all(16),
        decoration: BoxDecoration(
          color: Colors.white,
          borderRadius: BorderRadius.circular(12),
        ),
        child: QrImageView(
          data: data,
          version: QrVersions.auto,
          size: size,
          gapless: false,
          errorStateBuilder: (context, error) {
            return Center(
              child: Text(
                'Error generating QR code',
                style: const TextStyle(color: Colors.red),
              ),
            );
          },
        ),
      ),
    );
  }
}
