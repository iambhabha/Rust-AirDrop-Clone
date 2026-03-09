import 'dart:async';
import 'package:file_picker/file_picker.dart';
import 'package:flutter/cupertino.dart';
import 'package:flutter/material.dart' hide SearchBar;
import 'package:flutter_mobx/flutter_mobx.dart';
import 'package:flutter_screenutil/flutter_screenutil.dart';
import '../../ui/theme.dart';
import '../../src/rust/api/simple.dart';
import '../../models/transfer_progress.dart';
import '../../utils/extensions.dart';
import '../../stores/fastshare_store.dart';
import '../components/device_grid.dart';
import '../components/received_stack.dart';
import '../components/transfer_sheet.dart';
import '../components/settings_sheet.dart';
import '../components/search_bar.dart';
import '../screens/qr_scanner_screen.dart';
import 'history_screen.dart';
import 'package:pull_down_button/pull_down_button.dart';
import 'package:open_filex/open_filex.dart';

final fastShareStore = FastShareStore();

class FastShareHome extends StatefulWidget {
  const FastShareHome({super.key});
  @override
  State<FastShareHome> createState() => _FastShareHomeState();
}

class _FastShareHomeState extends State<FastShareHome>
    with WidgetsBindingObserver, SingleTickerProviderStateMixin {
  final TextEditingController _ipController = TextEditingController();
  bool _showingIncomingDialog = false;
  Timer? _pollTimer;
  late AnimationController _fadeController;
  late Animation<double> _fadeAnimation;

  @override
  void initState() {
    super.initState();
    WidgetsBinding.instance.addObserver(this);
    fastShareStore.init();

    _fadeController = AnimationController(
      vsync: this,
      duration: const Duration(milliseconds: 1000),
    );
    _fadeAnimation = CurvedAnimation(
      parent: _fadeController,
      curve: Curves.easeIn,
    );
    _fadeController.forward();

    _pollTimer = Timer.periodic(const Duration(milliseconds: 500), (_) {
      _checkPendingIncoming();
    });
  }

  @override
  void dispose() {
    WidgetsBinding.instance.removeObserver(this);
    _pollTimer?.cancel();
    _ipController.dispose();
    _fadeController.dispose();
    super.dispose();
  }

  Future<void> _checkPendingIncoming() async {
    if (_showingIncomingDialog) return;

    // Check pending from store
    final pending = fastShareStore.pendingIncoming;
    if (pending == null) return;

    _showingIncomingDialog = true;
    if (!mounted) return;

    showModalBottomSheet(
      context: context,
      isScrollControlled: true,
      backgroundColor: Colors.transparent,
      enableDrag: true,
      barrierColor: Colors.black.withOpacity(0.5),
      builder: (ctx) => Observer(
        builder: (_) {
          final pending = fastShareStore.pendingIncoming;

          if (pending == null) {
            // Close sheet as soon as the request is accepted/declined
            WidgetsBinding.instance.addPostFrameCallback((_) {
              if (Navigator.canPop(ctx)) Navigator.pop(ctx);
            });
            return const SizedBox.shrink();
          }

          return TransferSheet(
            pending: pending,
            onAccept: () {
              respondIncoming(fileId: pending.fileId, accept: true);
            },
            onDecline: () {
              respondIncoming(fileId: pending.fileId, accept: false);
              Navigator.pop(ctx);
            },
            onCancel: () {
              fastShareStore.handleCancelTransfer(pending.fileId);
              Navigator.pop(ctx);
            },
            onPause: () {
              fastShareStore.handlePauseTransfer(pending.fileId);
            },
          );
        },
      ),
    ).then((_) {
      _showingIncomingDialog = false;
    });
  }

  void _showProgressSheet(TransferProgress p) {
    showModalBottomSheet(
      context: context,
      isScrollControlled: true,
      backgroundColor: Colors.transparent,
      enableDrag: true,
      barrierColor: Colors.black.withOpacity(0.5),
      builder: (ctx) => Observer(
        builder: (_) {
          TransferProgress? current;
          if (fastShareStore.outgoingProgress?.fileName == p.fileName) {
            current = fastShareStore.outgoingProgress;
          } else {
            try {
              current = fastShareStore.activeIncoming.firstWhere(
                (element) => element.fileName == p.fileName,
              );
            } catch (_) {
              current = p;
            }
          }

          return TransferSheet(
            progress: current,
            onAccept: () {},
            onDecline: () {},
            onCancel: () {
              if (current?.fileId != null) {
                fastShareStore.handleCancelTransfer(current!.fileId!);
              } else {
                fastShareStore.handleCancelTransfer("batch");
              }
              Navigator.pop(ctx);
            },
            onPause: () {
              if (current?.fileId != null) {
                fastShareStore.handlePauseTransfer(current!.fileId!);
              } else {
                fastShareStore.handlePauseTransfer("batch");
              }
            },
          );
        },
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      backgroundColor: AppTheme.background,
      appBar: AppBar(
        title: Text(
          'Rust Drop',
          style: Theme.of(
            context,
          ).textTheme.titleMedium?.copyWith(fontWeight: FontWeight.bold),
        ),
        centerTitle: true,
        backgroundColor: Colors.transparent,
        elevation: 0,
        actions: [
          PullDownButton(
            itemBuilder: (context) => [
              PullDownMenuItem(
                title: 'Scanner',
                icon: CupertinoIcons.qrcode_viewfinder,
                onTap: () async {
                  final ip = await Navigator.push<String>(
                    context,
                    MaterialPageRoute(builder: (_) => const QrScannerScreen()),
                  );
                  if (ip != null) {
                    _ipController.text = ip;
                    if (mounted) context.showSnackBar('Target set to $ip');
                  }
                },
              ),
              PullDownMenuItem(
                title: 'History',
                icon: CupertinoIcons.clock,
                onTap: () {
                  if (mounted) context.push(const TransferHistoryScreen());
                },
              ),
              const PullDownMenuDivider(),
              PullDownMenuItem(
                title: 'Settings',
                icon: CupertinoIcons.settings,
                onTap: () {
                  _showSettings();
                },
              ),
              PullDownMenuItem(
                title: 'Profile',
                icon: CupertinoIcons.person_crop_circle,
                onTap: () {
                  if (mounted) context.showSnackBar('Profile coming soon');
                },
              ),
              const PullDownMenuDivider(),
              PullDownMenuItem(
                title: 'Delete',
                isDestructive: true,
                icon: CupertinoIcons.delete,
                onTap: () {
                  if (mounted) context.showSnackBar('Delete clicked');
                },
              ),
            ],
            buttonBuilder: (context, showMenu) => CupertinoButton(
              onPressed: showMenu,
              padding: EdgeInsets.zero,
              child: const Icon(
                CupertinoIcons.ellipsis_circle,
                color: Colors.white70,
                size: 28,
              ),
            ),
          ),
          const SizedBox(width: 8),
        ],
      ),
      body: FadeTransition(
        opacity: _fadeAnimation,
        child: SafeArea(
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              SearchBar(
                controller: _ipController,
                onQrResult: (ip) {
                  _ipController.text = ip;
                  context.showSnackBar('Target set to $ip');
                },
              ),
              Expanded(
                child: SingleChildScrollView(
                  physics: const BouncingScrollPhysics(),
                  padding: EdgeInsets.symmetric(
                    horizontal: 16.w,
                    vertical: 16.h,
                  ),
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      Observer(
                        builder: (_) {
                          final displayDevices = [
                            ...fastShareStore.nearbyDevices,
                          ];
                          final nearbyIps = fastShareStore.nearbyDevices
                              .map((d) => d.ipAddress)
                              .toSet();
                          fastShareStore.savedDevices.forEach((k, v) {
                            if (!nearbyIps.contains(v.ipAddress))
                              displayDevices.add(v);
                          });

                          return DeviceGrid(
                            devices: displayDevices,
                            onDeviceTap: (d) async {
                              if (d.isOnline) {
                                _ipController.text = d.ipAddress;
                                final result = await FilePicker.platform
                                    .pickFiles(allowMultiple: true);
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
                      SizedBox(height: 32.h),
                      Text(
                        'Received',
                        style: Theme.of(context).textTheme.displayMedium,
                      ),
                      SizedBox(height: 24.h),
                      Observer(
                        builder: (_) => ReceivedStack(
                          activeIncoming: fastShareStore.activeIncoming,
                          outgoingProgress: fastShareStore.outgoingProgress,
                          history: fastShareStore.history
                              .where((i) => i.isIncoming)
                              .toList(),
                          onProgressTap: (p) {
                            if ((p.progress >= 1.0 ||
                                    p.status?.toLowerCase() == "received" ||
                                    p.status?.toLowerCase() == "completed") &&
                                p.savedPath != null) {
                              OpenFilex.open(p.savedPath!);
                            } else {
                              _showProgressSheet(p);
                            }
                          },
                          onHistoryTap: (item) {
                            if (item.savedPath != null) {
                              OpenFilex.open(item.savedPath!);
                            } else {
                              final p = TransferProgress(
                                fileName: item.fileName,
                                progress: 1.0,
                                totalBytes: item.size,
                                receivedBytes: item.size,
                                status: item.status,
                                fromAddr: item.isIncoming
                                    ? "External Device"
                                    : "Me",
                                totalFiles: item.totalFiles,
                                savedPath: item.savedPath,
                              );
                              _showProgressSheet(p);
                            }
                          },
                        ),
                      ),
                      SizedBox(height: 40.h),
                    ],
                  ),
                ),
              ),
            ],
          ),
        ),
      ),
    );
  }

  void _showSettings() {
    showModalBottomSheet(
      context: context,
      isScrollControlled: true,
      backgroundColor: Colors.transparent,
      enableDrag: true,
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
