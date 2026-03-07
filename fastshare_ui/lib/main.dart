import 'dart:convert';
import 'dart:async';
import 'package:file_picker/file_picker.dart';
import 'package:flutter/material.dart';
import 'package:fastshare_ui/src/rust/api/simple.dart';
import 'package:fastshare_ui/src/rust/frb_generated.dart';

Future<void> main() async {
  await RustLib.init();
  runApp(const MyApp());
}

class MyApp extends StatelessWidget {
  const MyApp({super.key});

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      debugShowCheckedModeBanner: false,
      title: 'FastShare',
      theme: ThemeData.dark().copyWith(
        useMaterial3: true,
        scaffoldBackgroundColor: const Color(0xFF0F172A), // Slate 900
        colorScheme: const ColorScheme.dark(
          primary: Color(0xFF14B8A6), // Teal 500
          secondary: Color(0xFF38BDF8), // Sky 400
          surface: Color(0xFF1E293B), // Slate 800
          surfaceContainerHighest: Color(0xFF334155), // Slate 700
        ),
        textTheme: const TextTheme(
          displayLarge: TextStyle(
            fontWeight: FontWeight.w800,
            fontSize: 32,
            letterSpacing: -1.0,
          ),
          titleLarge: TextStyle(
            fontWeight: FontWeight.w600,
            fontSize: 20,
            letterSpacing: -0.5,
          ),
          bodyLarge: TextStyle(fontSize: 16, color: Colors.white70),
        ),
        elevatedButtonTheme: ElevatedButtonThemeData(
          style: ElevatedButton.styleFrom(
            elevation: 0,
            padding: const EdgeInsets.symmetric(horizontal: 24, vertical: 16),
            shape: RoundedRectangleBorder(
              borderRadius: BorderRadius.circular(12),
            ),
          ),
        ),
        outlinedButtonTheme: OutlinedButtonThemeData(
          style: OutlinedButton.styleFrom(
            padding: const EdgeInsets.symmetric(horizontal: 24, vertical: 16),
            shape: RoundedRectangleBorder(
              borderRadius: BorderRadius.circular(12),
            ),
          ),
        ),
      ),
      home: const FastShareHome(),
    );
  }
}

class FastShareHome extends StatefulWidget {
  const FastShareHome({super.key});

  @override
  State<FastShareHome> createState() => _FastShareHomeState();
}

class _FastShareHomeState extends State<FastShareHome> {
  String status = "Backend Idle";
  bool isEngineRunning = false;
  String? selectedFilePath;
  final TextEditingController _ipController = TextEditingController();
  List<dynamic> nearbyDevices = [];
  Timer? _discoveryTimer;

  @override
  void dispose() {
    _discoveryTimer?.cancel();
    _ipController.dispose();
    super.dispose();
  }

  void _startBackend() async {
    setState(() {
      status = "Starting Backend...";
      isEngineRunning = true;
    });
    // Call Rust function
    final result = await startFastshare();
    setState(() {
      status = result;
    });

    // Start polling for devices
    _discoveryTimer = Timer.periodic(const Duration(seconds: 2), (timer) async {
      final devicesJson = await getNearbyDevices();
      try {
        final List<dynamic> devices = jsonDecode(devicesJson);
        setState(() {
          nearbyDevices = devices;
        });
      } catch (e) {
        debugPrint("Error parsing devices: $e");
      }
    });
  }

  void _pickFile() async {
    FilePickerResult? result = await FilePicker.platform.pickFiles();
    if (result != null) {
      String path = result.files.single.path!;
      setState(() {
        selectedFilePath = path;
      });

      if (_ipController.text.isEmpty) {
        _showSnackBar(
          'Please enter a target IP address or select a device first!',
          isError: true,
        );
        return;
      }

      setState(() {
        status = "Sending file to ${_ipController.text}...";
      });

      // Send the file over QUIC
      final response = await sendFileToIp(
        filePath: path,
        targetIp: _ipController.text,
      );

      setState(() {
        status = response;
      });

      _showSnackBar(response);
    }
  }

  void _showSnackBar(String message, {bool isError = false}) {
    if (!mounted) return;
    ScaffoldMessenger.of(context).showSnackBar(
      SnackBar(
        content: Text(
          message,
          style: const TextStyle(fontWeight: FontWeight.w500),
        ),
        backgroundColor: isError
            ? Colors.redAccent
            : Theme.of(context).colorScheme.primary,
        behavior: SnackBarBehavior.floating,
        shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(10)),
        margin: const EdgeInsets.all(16),
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    final colorScheme = Theme.of(context).colorScheme;
    final textTheme = Theme.of(context).textTheme;

    return Scaffold(
      body: SafeArea(
        child: SingleChildScrollView(
          physics: const BouncingScrollPhysics(),
          child: Padding(
            padding: const EdgeInsets.symmetric(
              horizontal: 24.0,
              vertical: 32.0,
            ),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.stretch,
              children: [
                // Header
                Center(
                  child: Column(
                    children: [
                      Container(
                        padding: const EdgeInsets.all(16),
                        decoration: BoxDecoration(
                          color: colorScheme.primary.withOpacity(0.1),
                          shape: BoxShape.circle,
                        ),
                        child: Icon(
                          Icons.bolt,
                          size: 48,
                          color: colorScheme.primary,
                        ),
                      ),
                      const SizedBox(height: 16),
                      Text(
                        'FastShare',
                        style: textTheme.displayLarge?.copyWith(
                          color: colorScheme.primary,
                        ),
                      ),
                      const SizedBox(height: 8),
                      Text(
                        'Ultra-High-Performance P2P Transfer',
                        style: textTheme.bodyLarge,
                        textAlign: TextAlign.center,
                      ),
                    ],
                  ),
                ),

                const SizedBox(height: 40),

                // Engine Status Card
                _buildCard(
                  child: Column(
                    children: [
                      Row(
                        children: [
                          Container(
                            width: 12,
                            height: 12,
                            decoration: BoxDecoration(
                              shape: BoxShape.circle,
                              color: isEngineRunning
                                  ? colorScheme.primary
                                  : Colors.grey,
                              boxShadow: isEngineRunning
                                  ? [
                                      BoxShadow(
                                        color: colorScheme.primary.withOpacity(
                                          0.5,
                                        ),
                                        blurRadius: 10,
                                        spreadRadius: 2,
                                      ),
                                    ]
                                  : null,
                            ),
                          ),
                          const SizedBox(width: 12),
                          Expanded(
                            child: Text(
                              'Engine Status: $status',
                              style: const TextStyle(
                                fontWeight: FontWeight.w600,
                                fontSize: 15,
                              ),
                            ),
                          ),
                        ],
                      ),
                      const SizedBox(height: 16),
                      SizedBox(
                        width: double.infinity,
                        child: ElevatedButton.icon(
                          onPressed: isEngineRunning ? null : _startBackend,
                          icon: const Icon(Icons.rocket_launch),
                          label: const Text(
                            'Start Discovery Engine',
                            style: TextStyle(fontWeight: FontWeight.bold),
                          ),
                          style: ElevatedButton.styleFrom(
                            backgroundColor: colorScheme.primary,
                            foregroundColor: colorScheme.surface,
                            disabledBackgroundColor:
                                colorScheme.surfaceContainerHighest,
                          ),
                        ),
                      ),
                    ],
                  ),
                ),

                const SizedBox(height: 24),

                // Manual IP Entry
                _buildCard(
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      Text('Target Device', style: textTheme.titleLarge),
                      const SizedBox(height: 16),
                      TextField(
                        controller: _ipController,
                        style: const TextStyle(fontSize: 16),
                        decoration: InputDecoration(
                          hintText: "Enter IP (e.g. 192.168.1.10)",
                          filled: true,
                          fillColor: colorScheme.surfaceContainerHighest
                              .withOpacity(0.3),
                          border: OutlineInputBorder(
                            borderRadius: BorderRadius.circular(12),
                            borderSide: BorderSide.none,
                          ),
                          prefixIcon: Icon(
                            Icons.wifi,
                            color: colorScheme.primary,
                          ),
                          suffixIcon: IconButton(
                            icon: Icon(
                              Icons.qr_code_scanner,
                              color: colorScheme.primary,
                            ),
                            onPressed: () =>
                                _showSnackBar('QR Scanner coming soon!'),
                          ),
                        ),
                      ),
                    ],
                  ),
                ),

                const SizedBox(height: 24),

                // Nearby Devices (Conditional)
                if (nearbyDevices.isNotEmpty) ...[
                  _buildCard(
                    child: Column(
                      crossAxisAlignment: CrossAxisAlignment.start,
                      children: [
                        Row(
                          children: [
                            Text('Nearby Devices', style: textTheme.titleLarge),
                            const Spacer(),
                            SizedBox(
                              width: 16,
                              height: 16,
                              child: CircularProgressIndicator(
                                strokeWidth: 2,
                                color: colorScheme.primary,
                              ),
                            ),
                          ],
                        ),
                        const SizedBox(height: 16),
                        ListView.separated(
                          shrinkWrap: true,
                          physics: const NeverScrollableScrollPhysics(),
                          itemCount: nearbyDevices.length,
                          separatorBuilder: (_, __) =>
                              const Divider(height: 16, color: Colors.white10),
                          itemBuilder: (context, index) {
                            final device = nearbyDevices[index];
                            return ListTile(
                              contentPadding: EdgeInsets.zero,
                              leading: Container(
                                padding: const EdgeInsets.all(10),
                                decoration: BoxDecoration(
                                  color: colorScheme.surfaceContainerHighest
                                      .withOpacity(0.5),
                                  borderRadius: BorderRadius.circular(10),
                                ),
                                child: Icon(
                                  Icons.computer,
                                  color: colorScheme.secondary,
                                ),
                              ),
                              title: Text(
                                device['device_name'] ?? 'Unknown',
                                style: const TextStyle(
                                  fontWeight: FontWeight.w600,
                                ),
                              ),
                              subtitle: Text(device['ip_address'] ?? ''),
                              trailing: TextButton(
                                onPressed: () {
                                  _ipController.text = device['ip_address'];
                                  _showSnackBar(
                                    'Target set to ${device['device_name']}',
                                  );
                                },
                                style: TextButton.styleFrom(
                                  backgroundColor: colorScheme.secondary
                                      .withOpacity(0.1),
                                  foregroundColor: colorScheme.secondary,
                                ),
                                child: const Text(
                                  "Select",
                                  style: TextStyle(fontWeight: FontWeight.bold),
                                ),
                              ),
                            );
                          },
                        ),
                      ],
                    ),
                  ),
                  const SizedBox(height: 24),
                ],

                // Action Buttons (Using Wrap to fix RenderFlex Overflow)
                Wrap(
                  alignment: WrapAlignment.center,
                  spacing: 16,
                  runSpacing: 16,
                  children: [
                    ElevatedButton.icon(
                      onPressed: _pickFile,
                      icon: const Icon(Icons.send_rounded),
                      label: const Text(
                        'Send File',
                        style: TextStyle(
                          fontSize: 16,
                          fontWeight: FontWeight.bold,
                        ),
                      ),
                      style: ElevatedButton.styleFrom(
                        backgroundColor: colorScheme.primary,
                        foregroundColor: colorScheme.surface,
                        padding: const EdgeInsets.symmetric(
                          horizontal: 32,
                          vertical: 16,
                        ),
                      ),
                    ),
                    OutlinedButton.icon(
                      onPressed: () {
                        _showSnackBar('Auto-Receive is active');
                      },
                      icon: const Icon(Icons.download_rounded),
                      label: const Text(
                        'Receive File',
                        style: TextStyle(
                          fontSize: 16,
                          fontWeight: FontWeight.bold,
                        ),
                      ),
                      style: OutlinedButton.styleFrom(
                        foregroundColor: colorScheme.primary,
                        side: BorderSide(color: colorScheme.primary, width: 2),
                        padding: const EdgeInsets.symmetric(
                          horizontal: 32,
                          vertical: 16,
                        ),
                      ),
                    ),
                  ],
                ),

                if (selectedFilePath != null) ...[
                  const SizedBox(height: 24),
                  Center(
                    child: Container(
                      padding: const EdgeInsets.symmetric(
                        horizontal: 16,
                        vertical: 8,
                      ),
                      decoration: BoxDecoration(
                        color: colorScheme.primary.withOpacity(0.1),
                        borderRadius: BorderRadius.circular(20),
                      ),
                      child: Text(
                        'Selected: ${selectedFilePath!.split(RegExp(r'[/\\]')).last}',
                        style: TextStyle(
                          color: colorScheme.primary,
                          fontSize: 13,
                          fontWeight: FontWeight.w500,
                        ),
                        textAlign: TextAlign.center,
                        maxLines: 1,
                        overflow: TextOverflow.ellipsis,
                      ),
                    ),
                  ),
                ],

                const SizedBox(height: 24),
              ],
            ),
          ),
        ),
      ),
    );
  }

  Widget _buildCard({required Widget child}) {
    return Container(
      padding: const EdgeInsets.all(20),
      decoration: BoxDecoration(
        color: Theme.of(context).colorScheme.surface,
        borderRadius: BorderRadius.circular(20),
        border: Border.all(color: Colors.white.withOpacity(0.05)),
        boxShadow: [
          BoxShadow(
            color: Colors.black.withOpacity(0.15),
            blurRadius: 10,
            offset: const Offset(0, 4),
          ),
        ],
      ),
      child: child,
    );
  }
}
