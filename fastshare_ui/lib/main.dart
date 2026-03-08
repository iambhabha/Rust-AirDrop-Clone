import 'dart:convert';
import 'dart:async';
import 'dart:io';
import 'package:file_picker/file_picker.dart';
import 'package:flutter/material.dart';
import 'package:path_provider/path_provider.dart';
import 'package:mobile_scanner/mobile_scanner.dart';
import 'package:permission_handler/permission_handler.dart';
import 'package:fastshare_ui/src/rust/api/simple.dart';
import 'package:fastshare_ui/src/rust/frb_generated.dart';

final GlobalKey<NavigatorState> navigatorKey = GlobalKey<NavigatorState>();

Future<void> main() async {
  WidgetsFlutterBinding.ensureInitialized();
  await RustLib.init();
  runApp(const MyApp());
}

class MyApp extends StatelessWidget {
  const MyApp({super.key});

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      navigatorKey: navigatorKey,
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

class _FastShareHomeState extends State<FastShareHome>
    with WidgetsBindingObserver {
  String status = "Backend Idle";
  bool isEngineRunning = false;
  List<String> selectedFilePaths = [];
  final TextEditingController _ipController = TextEditingController();
  List<dynamic> nearbyDevices = [];
  Timer? _discoveryTimer;
  Timer? _incomingPollTimer;
  bool isScanning = false;
  bool _showingIncomingDialog = false;
  bool _isSending = false;
  int _sendingFileCount = 0;
  List<dynamic> activeIncoming = [];
  Timer? _incomingProgressTimer;

  @override
  void initState() {
    super.initState();
    WidgetsBinding.instance.addObserver(this);
    // Auto-start backend on launch to look professionally smooth
    WidgetsBinding.instance.addPostFrameCallback((_) {
      _startBackend();
    });
  }

  @override
  void didChangeAppLifecycleState(AppLifecycleState state) {
    super.didChangeAppLifecycleState(state);
    // When app comes to foreground, immediately check for pending incoming
    if (state == AppLifecycleState.resumed && mounted) {
      _checkPendingIncoming();
    }
  }

  @override
  void dispose() {
    WidgetsBinding.instance.removeObserver(this);
    _discoveryTimer?.cancel();
    _incomingPollTimer?.cancel();
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

    // Start background auto-refresh for devices list
    _discoveryTimer?.cancel();
    _discoveryTimer = Timer.periodic(const Duration(seconds: 2), (_) async {
      if (mounted && isEngineRunning && !isScanning) {
        try {
          final devicesJson = await getNearbyDevices();
          final List<dynamic> devices = jsonDecode(devicesJson);
          if (mounted) {
            setState(() {
              nearbyDevices = devices;
            });
          }
        } catch (e) {
          debugPrint("Error auto-refreshing devices: $e");
        }
      }
    });

    // Poll for incoming file transfers to show Accept/Decline popup (200ms for faster response)
    _incomingPollTimer?.cancel();
    _incomingPollTimer = Timer.periodic(
      const Duration(milliseconds: 200),
      (_) => _checkPendingIncoming(),
    );

    // Poll for active incoming transfer progress
    _incomingProgressTimer?.cancel();
    _incomingProgressTimer = Timer.periodic(
      const Duration(milliseconds: 500),
      (_) => _updateIncomingProgress(),
    );

    // Perform initial one-time scan
    _refreshDevices();
  }

  Future<void> _checkPendingIncoming() async {
    if (!isEngineRunning) return;
    try {
      final s = await getPendingIncoming();

      // If we got null/empty, just quiet return
      if (s == "null" || s.isEmpty) {
        if (_showingIncomingDialog) {
          debugPrint(
            '📥 [FastShare] No longer pending; closing dialog if open.',
          );
          if (mounted && navigatorKey.currentContext != null) {
            // We don't force pop here as the dialog might be popped by the user action already
          }
        }
        return;
      }

      final map = jsonDecode(s) as Map<String, dynamic>;
      final fileId = map['file_id'] as String? ?? '';

      // If we are already showing a dialog for THIS specific fileId, skip
      if (_showingIncomingDialog) return;

      debugPrint(
        '📥 [FastShare] ALERT: Incoming file request received! JSON: $s',
      );

      final fromAddr = map['from_addr'] as String? ?? '';
      final fileName = map['file_name'] as String? ?? '';
      final totalFilesRaw = map['total_files'];
      final totalFiles = (totalFilesRaw is num) ? totalFilesRaw.toInt() : 1;

      if (fileId.isEmpty) return;

      _showingIncomingDialog = true;

      final navContext = navigatorKey.currentContext;
      if (navContext == null) {
        debugPrint('⚠️ [FastShare] Cannot show dialog: navContext is null');
        _showingIncomingDialog = false;
        return;
      }

      // Show a Material Banner as a fallback/redundancy in case dialog fails or is hidden
      ScaffoldMessenger.of(navContext).showMaterialBanner(
        MaterialBanner(
          padding: const EdgeInsets.all(20),
          content: Text(
            'Incoming transfer from $fromAddr: $fileName ${totalFiles > 1 ? "(+$totalFiles files)" : ""}',
          ),
          leading: const Icon(Icons.file_download, color: Colors.teal),
          backgroundColor: const Color(0xFF1E293B),
          actions: [
            TextButton(
              onPressed: () async {
                ScaffoldMessenger.of(navContext).hideCurrentMaterialBanner();
                await respondIncoming(fileId: fileId, accept: true);
              },
              child: const Text(
                'ACCEPT',
                style: TextStyle(
                  color: Colors.teal,
                  fontWeight: FontWeight.bold,
                ),
              ),
            ),
            TextButton(
              onPressed: () async {
                ScaffoldMessenger.of(navContext).hideCurrentMaterialBanner();
                await respondIncoming(fileId: fileId, accept: false);
              },
              child: const Text(
                'DECLINE',
                style: TextStyle(color: Colors.redAccent),
              ),
            ),
          ],
        ),
      );

      // Also show the standard dialog for better UX
      showDialog<void>(
        context: navContext,
        barrierDismissible: false,
        builder: (ctx) => AlertDialog(
          backgroundColor: const Color(0xFF1E293B),
          title: Row(
            children: [
              const Icon(Icons.download_for_offline, color: Colors.teal),
              const SizedBox(width: 12),
              Text(
                totalFiles > 1 ? 'Receive $totalFiles Files?' : 'Receive File?',
              ),
            ],
          ),
          content: Column(
            mainAxisSize: MainAxisSize.min,
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              const Text(
                'Source Device:',
                style: TextStyle(color: Colors.white70, fontSize: 13),
              ),
              Text(
                fromAddr,
                style: const TextStyle(
                  fontWeight: FontWeight.bold,
                  fontSize: 16,
                ),
              ),
              const SizedBox(height: 16),
              const Text(
                'Files:',
                style: TextStyle(color: Colors.white70, fontSize: 13),
              ),
              Text(
                fileName,
                style: const TextStyle(fontStyle: FontStyle.italic),
              ),
              if (totalFiles > 1)
                Padding(
                  padding: const EdgeInsets.only(top: 4.0),
                  child: Text(
                    '+ ${totalFiles - 1} more files',
                    style: const TextStyle(
                      color: Colors.tealAccent,
                      fontSize: 12,
                    ),
                  ),
                ),
            ],
          ),
          actions: [
            OutlinedButton(
              onPressed: () async {
                ScaffoldMessenger.of(navContext).hideCurrentMaterialBanner();
                await respondIncoming(fileId: fileId, accept: false);
                if (ctx.mounted) Navigator.of(ctx).pop();
              },
              child: const Text('Decline'),
            ),
            FilledButton(
              onPressed: () async {
                ScaffoldMessenger.of(navContext).hideCurrentMaterialBanner();
                await respondIncoming(fileId: fileId, accept: true);
                if (ctx.mounted) Navigator.of(ctx).pop();
              },
              style: FilledButton.styleFrom(backgroundColor: Colors.teal),
              child: const Text('Accept and Download'),
            ),
          ],
        ),
      ).then((_) {
        _showingIncomingDialog = false;
        ScaffoldMessenger.of(navContext).hideCurrentMaterialBanner();
      });
    } catch (e) {
      debugPrint('Incoming check error: $e');
    }
  }

  Future<void> _updateIncomingProgress() async {
    if (!isEngineRunning) return;
    try {
      final progressJson = await getIncomingProgress();
      final List<dynamic> progress = jsonDecode(progressJson);
      if (mounted) {
        setState(() {
          activeIncoming = progress;
        });
      }
    } catch (e) {
      debugPrint("Error updating incoming progress: $e");
    }
  }

  Future<void> _refreshDevices() async {
    if (isScanning) return;

    setState(() {
      isScanning = true;
    });

    try {
      await triggerDiscoveryScan();
      // Wait for devices to respond before reading the state
      await Future.delayed(const Duration(milliseconds: 800));
      final devicesJson = await getNearbyDevices();
      final List<dynamic> devices = jsonDecode(devicesJson);
      setState(() {
        nearbyDevices = devices;
      });
    } catch (e) {
      debugPrint("Error parsing devices: $e");
    } finally {
      // Keep "scanning" state for a bit for better UI feedback
      await Future.delayed(const Duration(milliseconds: 700));
      if (mounted) {
        setState(() {
          isScanning = false;
        });
      }
    }
  }

  Future<List<String>> _resolveFilePaths(List<PlatformFile> files) async {
    final paths = <String>[];
    final tempDir = await getTemporaryDirectory();
    var idx = 0;
    for (final f in files) {
      if (f.path != null && f.path!.isNotEmpty) {
        paths.add(f.path!);
      } else if (f.readStream != null) {
        final tempPath =
            '${tempDir.path}/fastshare_${DateTime.now().microsecondsSinceEpoch}_${idx}_${f.name}';
        final sink = File(tempPath).openWrite();
        await f.readStream!.pipe(sink);
        await sink.close();
        paths.add(tempPath);
      } else if (f.bytes != null && f.bytes!.isNotEmpty) {
        final tempPath =
            '${tempDir.path}/fastshare_${DateTime.now().microsecondsSinceEpoch}_${idx}_${f.name}';
        await File(tempPath).writeAsBytes(f.bytes!);
        paths.add(tempPath);
      }
      idx++;
    }
    return paths;
  }

  void _pickFiles() async {
    FilePickerResult? result = await FilePicker.platform.pickFiles(
      allowMultiple: true,
      withData: true, // Needed when path is null on Android
      withReadStream: true,
    );

    if (result != null) {
      final List<String> paths = await _resolveFilePaths(result.files);
      if (paths.isEmpty) {
        _showSnackBar(
          'No valid files could be loaded. Try a different file source.',
          isError: true,
        );
        return;
      }
      if (paths.length < result.files.length) {
        _showSnackBar(
          '${result.files.length - paths.length} file(s) skipped (could not access path).',
          isError: true,
        );
      }
      setState(() {
        selectedFilePaths = paths;
      });

      if (_ipController.text.isEmpty) {
        _showSnackBar(
          'Please enter a target IP address or select a device first!',
          isError: true,
        );
        return;
      }

      setState(() {
        status = "Sending ${paths.length} file(s) to ${_ipController.text}...";
        _isSending = true;
        _sendingFileCount = paths.length;
      });

      // Send the files over QUIC
      final response = await sendFilesToIp(
        filePaths: paths,
        targetIp: _ipController.text,
      );

      setState(() {
        status = response;
        _isSending = false;
      });

      _showSnackBar(response);
    }
  }

  void _scanQr() async {
    if (!mounted) return;

    final status = await Permission.camera.request();
    if (status.isPermanentlyDenied) {
      _showSnackBar(
        'Camera permission is required for QR scanning. Please enable it in settings.',
        isError: true,
      );
      openAppSettings();
      return;
    }
    if (!status.isGranted) {
      _showSnackBar(
        'Camera permission is required for QR scanning.',
        isError: true,
      );
      return;
    }

    if (!mounted) return;

    final ip = await Navigator.of(context).push<String>(
      MaterialPageRoute(builder: (context) => const _QrScannerScreen()),
    );
    if (ip != null && ip.isNotEmpty && mounted) {
      _ipController.text = ip;
      _showSnackBar('Target set to $ip');
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

                // Transfer progress (when sending)
                if (_isSending) ...[
                  _buildCard(
                    child: Column(
                      crossAxisAlignment: CrossAxisAlignment.start,
                      children: [
                        Row(
                          children: [
                            Icon(Icons.upload_file, color: colorScheme.primary),
                            const SizedBox(width: 12),
                            Expanded(
                              child: Text(
                                'Sending $_sendingFileCount file(s)...',
                                style: textTheme.titleMedium?.copyWith(
                                  fontWeight: FontWeight.w600,
                                ),
                              ),
                            ),
                          ],
                        ),
                        const SizedBox(height: 12),
                        const LinearProgressIndicator(),
                        const SizedBox(height: 8),
                        Text(
                          status,
                          style: TextStyle(
                            fontSize: 13,
                            color: colorScheme.onSurfaceVariant.withOpacity(
                              0.8,
                            ),
                          ),
                        ),
                      ],
                    ),
                  ),
                  const SizedBox(height: 24),
                ],

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
                      Row(
                        children: [
                          Expanded(
                            child: ElevatedButton.icon(
                              onPressed: isEngineRunning ? null : _startBackend,
                              icon: const Icon(Icons.rocket_launch),
                              label: const Text(
                                'Start Engine',
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
                          const SizedBox(width: 12),
                          IconButton.filledTonal(
                            onPressed: isEngineRunning
                                ? _checkPendingIncoming
                                : null,
                            icon: const Icon(Icons.sync),
                            tooltip: 'Poll Incoming',
                          ),
                        ],
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
                      Row(
                        children: [
                          Expanded(
                            child: TextField(
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
                              ),
                            ),
                          ),
                          const SizedBox(width: 12),
                          Container(
                            decoration: BoxDecoration(
                              color: colorScheme.primary.withOpacity(0.1),
                              borderRadius: BorderRadius.circular(12),
                            ),
                            child: IconButton(
                              icon: Icon(
                                Icons.qr_code_scanner_rounded,
                                color: colorScheme.primary,
                                size: 28,
                              ),
                              padding: const EdgeInsets.all(12),
                              onPressed: _scanQr,
                              tooltip: 'Scan QR Code',
                            ),
                          ),
                        ],
                      ),
                    ],
                  ),
                ),

                const SizedBox(height: 24),

                // Nearby Devices (Auto-Scanning)
                if (isEngineRunning) ...[
                  _buildCard(
                    child: Column(
                      crossAxisAlignment: CrossAxisAlignment.start,
                      children: [
                        Row(
                          children: [
                            Text('Nearby Devices', style: textTheme.titleLarge),
                            const Spacer(),
                            if (isScanning)
                              SizedBox(
                                width: 20,
                                height: 20,
                                child: CircularProgressIndicator(
                                  strokeWidth: 2,
                                  color: colorScheme.primary,
                                ),
                              )
                            else
                              Row(
                                mainAxisSize: MainAxisSize.min,
                                children: [
                                  IconButton(
                                    icon: const Icon(
                                      Icons.qr_code_scanner,
                                      size: 20,
                                    ),
                                    color: colorScheme.primary,
                                    padding: EdgeInsets.zero,
                                    constraints: const BoxConstraints(),
                                    tooltip: 'Scan QR Code',
                                    onPressed: _scanQr,
                                  ),
                                  const SizedBox(width: 12),
                                  IconButton(
                                    icon: const Icon(Icons.refresh, size: 20),
                                    color: colorScheme.primary,
                                    padding: EdgeInsets.zero,
                                    constraints: const BoxConstraints(),
                                    tooltip: 'Refresh devices',
                                    onPressed: _refreshDevices,
                                  ),
                                ],
                              ),
                          ],
                        ),
                        const SizedBox(height: 16),
                        if (isScanning && nearbyDevices.isEmpty)
                          Padding(
                            padding: const EdgeInsets.symmetric(vertical: 24.0),
                            child: Center(
                              child: Column(
                                children: [
                                  Icon(
                                    Icons.network_check_rounded,
                                    size: 40,
                                    color: colorScheme.secondary.withOpacity(
                                      0.5,
                                    ),
                                  ),
                                  const SizedBox(height: 12),
                                  Text(
                                    'Scanning for devices...',
                                    style: TextStyle(
                                      color: colorScheme.onSurfaceVariant,
                                      fontStyle: FontStyle.italic,
                                    ),
                                  ),
                                ],
                              ),
                            ),
                          )
                        else if (!isScanning && nearbyDevices.isEmpty)
                          Padding(
                            padding: const EdgeInsets.symmetric(vertical: 24.0),
                            child: Center(
                              child: Column(
                                children: [
                                  Icon(
                                    Icons.devices_other_rounded,
                                    size: 40,
                                    color: colorScheme.onSurfaceVariant
                                        .withOpacity(0.3),
                                  ),
                                  const SizedBox(height: 12),
                                  const Text(
                                    'No devices found',
                                    style: TextStyle(color: Colors.white38),
                                  ),
                                  TextButton(
                                    onPressed: _refreshDevices,
                                    child: const Text('Scan Again'),
                                  ),
                                ],
                              ),
                            ),
                          )
                        else
                          ListView.separated(
                            shrinkWrap: true,
                            physics: const NeverScrollableScrollPhysics(),
                            itemCount: nearbyDevices.length,
                            separatorBuilder: (_, __) => const Divider(
                              height: 16,
                              color: Colors.white10,
                            ),
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
                                    style: TextStyle(
                                      fontWeight: FontWeight.bold,
                                    ),
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

                // Action Buttons (Pick & Send)
                Wrap(
                  alignment: WrapAlignment.center,
                  spacing: 16,
                  runSpacing: 16,
                  children: [
                    SizedBox(
                      height: 56,
                      child: ElevatedButton.icon(
                        onPressed: isEngineRunning ? _pickFiles : null,
                        icon: const Icon(Icons.file_upload_rounded),
                        label: Text(
                          selectedFilePaths.isEmpty
                              ? 'Send Multiple Files'
                              : '${selectedFilePaths.length} Selected',
                        ),
                        style: ElevatedButton.styleFrom(
                          backgroundColor: colorScheme.secondary,
                          foregroundColor: colorScheme.onSecondary,
                          elevation: 4,
                          padding: const EdgeInsets.symmetric(horizontal: 24),
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

                if (selectedFilePaths.isNotEmpty) ...[
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
                        selectedFilePaths.length == 1
                            ? 'Selected: ${selectedFilePaths.first.split(RegExp(r'[/\\]')).last}'
                            : '${selectedFilePaths.length} Files Selected',
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

/// Extracts IP from QR content. Supports plain IP or JSON {"ip":"x.x.x.x"}.
String? _extractIpFromQr(String raw) {
  final trimmed = raw.trim();
  // Plain IPv4
  final ipv4 = RegExp(r'^\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}(:\d+)?$');
  if (ipv4.hasMatch(trimmed)) {
    return trimmed.split(':').first;
  }
  // JSON format
  try {
    final json = jsonDecode(trimmed) as Map<String, dynamic>;
    final ip = json['ip']?.toString();
    if (ip != null && ip.isNotEmpty) return ip;
  } catch (_) {}
  return trimmed.isNotEmpty ? trimmed : null;
}

class _QrScannerScreen extends StatefulWidget {
  const _QrScannerScreen();

  @override
  State<_QrScannerScreen> createState() => _QrScannerScreenState();
}

class _QrScannerScreenState extends State<_QrScannerScreen> {
  bool _hasScanned = false;

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: const Text('Scan QR Code'),
        backgroundColor: Theme.of(context).colorScheme.surface,
      ),
      body: Stack(
        children: [
          MobileScanner(
            onDetect: (capture) {
              if (_hasScanned) return;
              final barcodes = capture.barcodes;
              for (final barcode in barcodes) {
                final raw = barcode.rawValue;
                if (raw != null && raw.isNotEmpty) {
                  _hasScanned = true;
                  final ip = _extractIpFromQr(raw);
                  if (ip != null && mounted) {
                    Navigator.of(context).pop(ip);
                  }
                  return;
                }
              }
            },
          ),
          Center(
            child: Container(
              width: 250,
              height: 250,
              decoration: BoxDecoration(
                border: Border.all(color: Colors.white54, width: 2),
                borderRadius: BorderRadius.circular(12),
              ),
              child: const Center(
                child: Text(
                  'Point camera at QR code',
                  style: TextStyle(color: Colors.white70, fontSize: 14),
                  textAlign: TextAlign.center,
                ),
              ),
            ),
          ),
        ],
      ),
    );
  }
}
