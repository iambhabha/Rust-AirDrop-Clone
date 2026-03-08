import 'dart:convert';
import 'dart:async';
import 'package:file_picker/file_picker.dart';
import 'package:flutter/cupertino.dart';
import 'package:flutter/material.dart' hide SearchBar;
import 'package:flutter_mobx/flutter_mobx.dart';
import 'package:flutter_screenutil/flutter_screenutil.dart';

import '../../src/rust/api/simple.dart';
import '../../models/transfer_progress.dart';
import '../../utils/extensions.dart';
import '../../stores/fastshare_store.dart';
import '../components/device_grid.dart';
import '../components/received_stack.dart';
import '../components/settings_sheet.dart';
import '../components/search_bar.dart';
import 'history_screen.dart';

final fastShareStore = FastShareStore();

class FastShareHome extends StatefulWidget {
  const FastShareHome({super.key});
  @override
  State<FastShareHome> createState() => _FastShareHomeState();
}

class _FastShareHomeState extends State<FastShareHome>
    with WidgetsBindingObserver {
  final TextEditingController _ipController = TextEditingController();
  bool _showingIncomingDialog = false;
  Timer? _pollTimer;

  @override
  void initState() {
    super.initState();
    WidgetsBinding.instance.addObserver(this);
    fastShareStore.init();

    _pollTimer = Timer.periodic(const Duration(milliseconds: 500), (_) {
      _checkPendingIncoming();
    });
  }

  @override
  void dispose() {
    WidgetsBinding.instance.removeObserver(this);
    _pollTimer?.cancel();
    _ipController.dispose();
    super.dispose();
  }

  Future<void> _checkPendingIncoming() async {
    if (_showingIncomingDialog) return;
    final s = await getPendingIncoming();
    if (s == "null" || s.isEmpty) return;
    final pending = PendingIncoming.fromJson(jsonDecode(s));
    _showingIncomingDialog = true;
    if (!mounted) return;
    showDialog(
      context: context,
      barrierDismissible: false,
      builder: (ctx) => AlertDialog(
        title: Text('Receive ${pending.totalFiles} items?'),
        content: Text('From: ${pending.fromAddr}\nFile: ${pending.fileName}'),
        actions: [
          TextButton(
            onPressed: () {
              respondIncoming(fileId: pending.fileId, accept: false);
              Navigator.pop(ctx);
            },
            child: const Text('Decline'),
          ),
          TextButton(
            onPressed: () {
              respondIncoming(fileId: pending.fileId, accept: true);
              Navigator.pop(ctx);
            },
            child: const Text('Accept'),
          ),
        ],
      ),
    ).then((_) => _showingIncomingDialog = false);
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      backgroundColor: Colors.black,
      appBar: AppBar(
        leading: IconButton(
          icon: const Icon(Icons.settings_outlined, color: Colors.white60),
          onPressed: _showSettings,
        ),
        title: Text(
          'Blip',
          style: TextStyle(fontWeight: FontWeight.bold, fontSize: 18.sp),
        ),
        centerTitle: true,
        backgroundColor: Colors.transparent,
        actions: [
          IconButton(
            icon: const Icon(Icons.history, color: Colors.white60),
            onPressed: () => context.push(const TransferHistoryScreen()),
          ),
        ],
      ),
      body: SafeArea(
        child: Column(
          children: [
            SearchBar(
              controller: _ipController,
              onQrResult: (ip) {
                _ipController.text = ip;
                context.showSnackBar('Target set to $ip');
              },
            ),
            Padding(
              padding: EdgeInsets.symmetric(horizontal: 16.w, vertical: 16.h),
              child: Observer(
                builder: (_) {
                  final displayDevices = [...fastShareStore.nearbyDevices];
                  final nearbyIps = fastShareStore.nearbyDevices
                      .map((d) => d.ipAddress)
                      .toSet();
                  fastShareStore.savedDevices.forEach((k, v) {
                    if (!nearbyIps.contains(v.ipAddress)) displayDevices.add(v);
                  });

                  return DeviceGrid(
                    devices: displayDevices,
                    onDeviceTap: (d) async {
                      if (d.isOnline) {
                        _ipController.text = d.ipAddress;
                        final result = await FilePicker.platform.pickFiles(
                          allowMultiple: true,
                        );
                        if (result != null) {
                          final paths = result.paths
                              .whereType<String>()
                              .toList();
                          final res = await fastShareStore.sendFiles(
                            paths,
                            d.ipAddress,
                          );
                          if (mounted)
                            context.showSnackBar(
                              res,
                              isError: res.contains('Error'),
                            );
                        }
                      } else {
                        context.showSnackBar(
                          'Device is offline',
                          isError: true,
                        );
                      }
                    },
                  );
                },
              ),
            ),
            const Spacer(),
            Padding(
              padding: EdgeInsets.all(16.w),
              child: Observer(
                builder: (_) => ReceivedStack(
                  activeIncoming: fastShareStore.activeIncoming,
                  outgoingProgress: fastShareStore.outgoingProgress,
                ),
              ),
            ),
          ],
        ),
      ),
    );
  }

  void _showSettings() {
    showCupertinoModalPopup(
      context: context,
      builder: (_) => Observer(
        builder: (_) => SettingsSheet(
          savedDevices: fastShareStore.savedDevices.values.toList(),
          checksumEnabled: fastShareStore.checksumEnabled,
          compressionEnabled: fastShareStore.compressionEnabled,
          onChecksumChanged: (v) => fastShareStore.setChecksum(v),
          onCompressionChanged: (v) => fastShareStore.setCompression(v),
        ),
      ),
    );
  }
}
