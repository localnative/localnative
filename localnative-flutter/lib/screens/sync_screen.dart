import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import 'package:mobile_scanner/mobile_scanner.dart';

import '../providers/sync_provider.dart';
import '../providers/notes_provider.dart';
import '../widgets/qr_display.dart';

/// Screen for P2P synchronization
class SyncScreen extends StatefulWidget {
  const SyncScreen({super.key});

  @override
  State<SyncScreen> createState() => _SyncScreenState();
}

class _SyncScreenState extends State<SyncScreen> {
  final TextEditingController _addressController = TextEditingController();
  bool _showScanner = false;

  @override
  void dispose() {
    _addressController.dispose();
    super.dispose();
  }

  void _startServer() {
    final syncProvider = context.read<SyncProvider>();
    syncProvider.startServer();
  }

  void _stopServer() {
    final syncProvider = context.read<SyncProvider>();
    syncProvider.stopServer();
  }

  void _syncWithServer(String address) async {
    final syncProvider = context.read<SyncProvider>();
    final success = await syncProvider.syncWithServer(address);

    if (success && mounted) {
      // Refresh notes after successful sync
      context.read<NotesProvider>().refresh();
    }
  }

  void _showQRScanner() {
    setState(() {
      _showScanner = true;
    });
  }

  void _onQRScanned(String code) {
    setState(() {
      _showScanner = false;
    });
    _addressController.text = code;
    _syncWithServer(code);
  }

  @override
  Widget build(BuildContext context) {
    return Consumer<SyncProvider>(
      builder: (context, syncProvider, _) {
        if (_showScanner) {
          return _buildQRScanner();
        }

        return SingleChildScrollView(
          padding: const EdgeInsets.all(16.0),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.stretch,
            children: [
              // Server mode section
              Card(
                child: Padding(
                  padding: const EdgeInsets.all(16.0),
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      Row(
                        children: [
                          const Icon(Icons.router, size: 32),
                          const SizedBox(width: 12),
                          Text(
                            'Server Mode',
                            style: Theme.of(context).textTheme.headlineSmall,
                          ),
                        ],
                      ),
                      const SizedBox(height: 16),
                      Text(
                        'Start a server on this device for others to sync with.',
                        style: Theme.of(context).textTheme.bodyMedium,
                      ),
                      const SizedBox(height: 16),

                      if (syncProvider.isServerRunning) ...[
                        QRDisplay(data: syncProvider.serverAddress),
                        const SizedBox(height: 16),
                        Text(
                          'Server Address: ${syncProvider.serverAddress}',
                          style: Theme.of(context).textTheme.bodyLarge?.copyWith(
                            fontWeight: FontWeight.bold,
                          ),
                        ),
                        if (syncProvider.message != null) ...[
                          const SizedBox(height: 8),
                          Text(
                            syncProvider.message!,
                            style: const TextStyle(color: Colors.green),
                          ),
                        ],
                        const SizedBox(height: 16),
                        ElevatedButton.icon(
                          onPressed: _stopServer,
                          icon: const Icon(Icons.stop),
                          label: const Text('Stop Server'),
                          style: ElevatedButton.styleFrom(
                            backgroundColor: Colors.red,
                            foregroundColor: Colors.white,
                          ),
                        ),
                      ] else ...[
                        TextField(
                          controller: TextEditingController(
                            text: syncProvider.serverAddress,
                          ),
                          decoration: const InputDecoration(
                            labelText: 'Server Address',
                            hintText: '0.0.0.0:2345',
                            border: OutlineInputBorder(),
                          ),
                          onChanged: (value) {
                            syncProvider.setServerAddress(value);
                          },
                        ),
                        const SizedBox(height: 16),
                        ElevatedButton.icon(
                          onPressed: _startServer,
                          icon: const Icon(Icons.play_arrow),
                          label: const Text('Start Server'),
                        ),
                      ],
                    ],
                  ),
                ),
              ),

              const SizedBox(height: 24),

              // Client mode section
              Card(
                child: Padding(
                  padding: const EdgeInsets.all(16.0),
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      Row(
                        children: [
                          const Icon(Icons.cloud_sync, size: 32),
                          const SizedBox(width: 12),
                          Text(
                            'Client Mode',
                            style: Theme.of(context).textTheme.headlineSmall,
                          ),
                        ],
                      ),
                      const SizedBox(height: 16),
                      Text(
                        'Connect to another device to sync notes.',
                        style: Theme.of(context).textTheme.bodyMedium,
                      ),
                      const SizedBox(height: 16),

                      TextField(
                        controller: _addressController,
                        decoration: const InputDecoration(
                          labelText: 'Server Address',
                          hintText: '192.168.1.100:2345',
                          border: OutlineInputBorder(),
                        ),
                      ),
                      const SizedBox(height: 16),

                      Row(
                        children: [
                          Expanded(
                            child: ElevatedButton.icon(
                              onPressed: () {
                                final address = _addressController.text.trim();
                                if (address.isNotEmpty) {
                                  _syncWithServer(address);
                                }
                              },
                              icon: const Icon(Icons.sync),
                              label: const Text('Sync Now'),
                            ),
                          ),
                          const SizedBox(width: 12),
                          ElevatedButton.icon(
                            onPressed: _showQRScanner,
                            icon: const Icon(Icons.qr_code_scanner),
                            label: const Text('Scan QR'),
                          ),
                        ],
                      ),

                      if (syncProvider.status == SyncStatus.syncing) ...[
                        const SizedBox(height: 16),
                        const LinearProgressIndicator(),
                        const SizedBox(height: 8),
                        Text(syncProvider.message ?? 'Syncing...'),
                      ],

                      if (syncProvider.status == SyncStatus.success) ...[
                        const SizedBox(height: 16),
                        Container(
                          padding: const EdgeInsets.all(12),
                          decoration: BoxDecoration(
                            color: Colors.green.shade50,
                            borderRadius: BorderRadius.circular(8),
                            border: Border.all(color: Colors.green),
                          ),
                          child: Row(
                            children: [
                              const Icon(Icons.check_circle, color: Colors.green),
                              const SizedBox(width: 12),
                              Expanded(
                                child: Text(
                                  syncProvider.message ?? 'Sync completed',
                                  style: const TextStyle(color: Colors.green),
                                ),
                              ),
                            ],
                          ),
                        ),
                      ],
                    ],
                  ),
                ),
              ),

              // Error display
              if (syncProvider.errorMessage != null) ...[
                const SizedBox(height: 16),
                Container(
                  padding: const EdgeInsets.all(12),
                  decoration: BoxDecoration(
                    color: Colors.red.shade50,
                    borderRadius: BorderRadius.circular(8),
                    border: Border.all(color: Colors.red),
                  ),
                  child: Row(
                    children: [
                      const Icon(Icons.error, color: Colors.red),
                      const SizedBox(width: 12),
                      Expanded(
                        child: Text(
                          syncProvider.errorMessage!,
                          style: const TextStyle(color: Colors.red),
                        ),
                      ),
                      IconButton(
                        icon: const Icon(Icons.close, size: 20),
                        onPressed: () => syncProvider.clearError(),
                      ),
                    ],
                  ),
                ),
              ],
            ],
          ),
        );
      },
    );
  }

  Widget _buildQRScanner() {
    return Stack(
      children: [
        MobileScanner(
          onDetect: (capture) {
            final List<Barcode> barcodes = capture.barcodes;
            for (final barcode in barcodes) {
              if (barcode.rawValue != null) {
                _onQRScanned(barcode.rawValue!);
                break;
              }
            }
          },
        ),
        Positioned(
          top: 16,
          left: 16,
          child: IconButton(
            icon: const Icon(Icons.close, color: Colors.white, size: 32),
            onPressed: () {
              setState(() {
                _showScanner = false;
              });
            },
          ),
        ),
      ],
    );
  }
}
