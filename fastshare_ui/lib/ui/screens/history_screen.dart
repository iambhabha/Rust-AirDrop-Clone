import 'package:flutter/material.dart';
import 'package:flutter_mobx/flutter_mobx.dart';
import 'home_screen.dart';
import '../theme.dart';
import '../../models/history_item.dart';
import '../../models/transfer_progress.dart';
import '../components/transfer_sheet.dart';
import '../../utils/extensions.dart';
import 'package:open_filex/open_filex.dart';

class TransferHistoryScreen extends StatefulWidget {
  const TransferHistoryScreen({super.key});

  @override
  State<TransferHistoryScreen> createState() => _TransferHistoryScreenState();
}

class _TransferHistoryScreenState extends State<TransferHistoryScreen>
    with SingleTickerProviderStateMixin {
  late TabController _tabController;

  @override
  void initState() {
    super.initState();
    _tabController = TabController(length: 3, vsync: this);
  }

  @override
  void dispose() {
    _tabController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      backgroundColor: AppTheme.background,
      appBar: AppBar(
        title: const Text('Transfer History'),
        bottom: TabBar(
          controller: _tabController,
          indicatorColor: AppTheme.primary,
          indicatorWeight: 2,
          dividerColor: AppTheme.border,
          labelColor: AppTheme.primary,
          unselectedLabelColor: AppTheme.mutedForeground,
          tabs: const [
            Tab(text: 'All'),
            Tab(text: 'Received'),
            Tab(text: 'Sent'),
          ],
        ),
        actions: [
          IconButton(
            icon: const Icon(Icons.refresh, color: AppTheme.mutedForeground),
            onPressed: () => fastShareStore.loadHistory(),
          ),
        ],
      ),
      body: Observer(
        builder: (_) {
          if (fastShareStore.isHistoryLoading) {
            return const Center(
              child: CircularProgressIndicator(color: AppTheme.primary),
            );
          }
          return TabBarView(
            controller: _tabController,
            physics: const BouncingScrollPhysics(),
            children: [
              _buildAllTab(),
              Observer(
                builder: (_) => _buildHistoryList(
                  fastShareStore.history.where((i) => i.isIncoming).toList(),
                  activeTransfers: fastShareStore.activeIncoming.toList(),
                ),
              ),
              Observer(
                builder: (_) => _buildHistoryList(
                  fastShareStore.history.where((i) => !i.isIncoming).toList(),
                  outgoing: fastShareStore.outgoingProgress,
                ),
              ),
            ],
          );
        },
      ),
    );
  }

  Widget _buildAllTab() {
    return Observer(
      builder: (_) => _buildHistoryList(
        fastShareStore.history.toList(),
        activeTransfers: fastShareStore.activeIncoming.toList(),
        outgoing: fastShareStore.outgoingProgress,
      ),
    );
  }

  Widget _buildHistoryList(
    List<HistoryItem> items, {
    List<TransferProgress> activeTransfers = const [],
    TransferProgress? outgoing,
  }) {
    final hasActive = activeTransfers.isNotEmpty || outgoing != null;
    if (items.isEmpty && !hasActive) {
      return Center(
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            Icon(Icons.history, size: 48, color: AppTheme.border),
            const SizedBox(height: 16),
            const Text(
              'No transfers found',
              style: TextStyle(color: AppTheme.mutedForeground),
            ),
          ],
        ),
      );
    }
    return ListView(
      physics: const BouncingScrollPhysics(),
      padding: const EdgeInsets.all(16),
      children: [
        // ── Active Transfers Section ──
        if (hasActive) ...[
          Padding(
            padding: const EdgeInsets.only(bottom: 8, left: 4),
            child: Row(
              children: [
                Container(
                  width: 7,
                  height: 7,
                  decoration: const BoxDecoration(
                    color: AppTheme.primary,
                    shape: BoxShape.circle,
                  ),
                ),
                const SizedBox(width: 6),
                const Text(
                  'IN PROGRESS',
                  style: TextStyle(
                    color: AppTheme.primary,
                    fontSize: 11,
                    fontWeight: FontWeight.bold,
                    letterSpacing: 1.2,
                  ),
                ),
              ],
            ),
          ),
          ...activeTransfers.map(
            (p) => _buildActiveTransferTile(context, p, isIncoming: true),
          ),
          if (outgoing != null)
            _buildActiveTransferTile(context, outgoing, isIncoming: false),
          const SizedBox(height: 16),
          Divider(color: AppTheme.border.withOpacity(0.3), height: 1),
          const SizedBox(height: 16),
        ],
        // ── History Items ──
        ...items.asMap().entries.map((entry) {
          final index = entry.key;
          final item = entry.value;
          return TweenAnimationBuilder<double>(
            duration: Duration(milliseconds: 300 + (index.clamp(0, 10) * 50)),
            tween: Tween(begin: 0.0, end: 1.0),
            curve: Curves.easeOut,
            builder: (context, value, child) =>
                Opacity(opacity: value, child: child),
            child: Card(
              margin: const EdgeInsets.only(bottom: 12),
              color: AppTheme.card.withOpacity(0.5),
              child: ListTile(
                onTap: () => _showHistoryDetails(context, item),
                contentPadding: const EdgeInsets.symmetric(
                  horizontal: 16,
                  vertical: 4,
                ),
                leading: Stack(
                  alignment: Alignment.center,
                  children: [
                    SizedBox(
                      width: 38,
                      height: 38,
                      child: CircularProgressIndicator(
                        value: 1.0,
                        strokeWidth: 2,
                        backgroundColor: Colors.transparent,
                        valueColor: AlwaysStoppedAnimation<Color>(
                          (item.isIncoming
                                  ? AppTheme.secondary
                                  : AppTheme.foreground)
                              .withOpacity(0.2),
                        ),
                      ),
                    ),
                    Container(
                      padding: const EdgeInsets.all(8),
                      decoration: BoxDecoration(
                        color:
                            (item.isIncoming
                                    ? AppTheme.secondary
                                    : AppTheme.foreground)
                                .withOpacity(0.1),
                        gradient: item.isIncoming
                            ? AppTheme.secondaryGradient
                            : null,
                        shape: BoxShape.circle,
                      ),
                      child: Icon(
                        item.isIncoming
                            ? Icons.download_rounded
                            : Icons.upload_rounded,
                        color: item.isIncoming
                            ? Colors.white
                            : AppTheme.foreground,
                        size: 18,
                      ),
                    ),
                  ],
                ),
                title: Text(
                  item.fileName.fileName,
                  style: const TextStyle(
                    color: AppTheme.foreground,
                    fontWeight: FontWeight.w500,
                    fontSize: 14,
                  ),
                ),
                subtitle: Text(
                  '${item.status} • ${item.timestamp}',
                  style: const TextStyle(
                    color: AppTheme.mutedForeground,
                    fontSize: 11,
                  ),
                ),
                trailing: const Icon(
                  Icons.arrow_forward_ios,
                  color: AppTheme.border,
                  size: 12,
                ),
              ),
            ),
          );
        }),
      ],
    );
  }

  Widget _buildActiveTransferTile(
    BuildContext context,
    TransferProgress p, {
    required bool isIncoming,
  }) {
    final percentage = ((p.progress) * 100).toInt().clamp(0, 100);
    final color = isIncoming ? AppTheme.secondary : AppTheme.primary;
    return Card(
      margin: const EdgeInsets.only(bottom: 10),
      color: color.withOpacity(0.06),
      shape: RoundedRectangleBorder(
        borderRadius: BorderRadius.circular(16),
        side: BorderSide(color: color.withOpacity(0.2), width: 1),
      ),
      child: InkWell(
        borderRadius: BorderRadius.circular(16),
        onTap: () => _showActiveProgress(context, p),
        child: Padding(
          padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 12),
          child: Row(
            children: [
              // Progress Ring
              SizedBox(
                width: 42,
                height: 42,
                child: Stack(
                  alignment: Alignment.center,
                  children: [
                    CircularProgressIndicator(
                      value: p.progress.clamp(0.0, 1.0),
                      strokeWidth: 3,
                      backgroundColor: color.withOpacity(0.1),
                      valueColor: AlwaysStoppedAnimation<Color>(color),
                    ),
                    Icon(
                      isIncoming
                          ? Icons.download_rounded
                          : Icons.upload_rounded,
                      size: 16,
                      color: color,
                    ),
                  ],
                ),
              ),
              const SizedBox(width: 14),
              // Info
              Expanded(
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Text(
                      p.fileName.fileName,
                      style: const TextStyle(
                        color: AppTheme.foreground,
                        fontWeight: FontWeight.w600,
                        fontSize: 14,
                      ),
                      maxLines: 1,
                      overflow: TextOverflow.ellipsis,
                    ),
                    const SizedBox(height: 3),
                    Text(
                      p.status ?? (isIncoming ? 'Receiving...' : 'Sending...'),
                      style: TextStyle(
                        color: color.withOpacity(0.8),
                        fontSize: 12,
                      ),
                    ),
                  ],
                ),
              ),
              const SizedBox(width: 8),
              // Percentage badge
              Container(
                padding: const EdgeInsets.symmetric(
                  horizontal: 10,
                  vertical: 4,
                ),
                decoration: BoxDecoration(
                  color: color.withOpacity(0.15),
                  borderRadius: BorderRadius.circular(20),
                ),
                child: Text(
                  '$percentage%',
                  style: TextStyle(
                    color: color,
                    fontSize: 13,
                    fontWeight: FontWeight.bold,
                  ),
                ),
              ),
            ],
          ),
        ),
      ),
    );
  }

  void _showHistoryDetails(BuildContext context, HistoryItem item) {
    if (item.savedPath != null) {
      OpenFilex.open(item.savedPath!);
      return;
    }

    final progress = TransferProgress(
      fileName: item.fileName,
      progress: 1.0,
      totalBytes: item.size,
      receivedBytes: item.size,
      status: item.status,
      fromAddr: item.isIncoming ? "External Device" : "Me",
      totalFiles: item.totalFiles,
      savedPath: item.savedPath,
    );

    showModalBottomSheet(
      context: context,
      isScrollControlled: true,
      backgroundColor: Colors.transparent,
      enableDrag: true,
      barrierColor: Colors.black.withOpacity(0.5),
      builder: (ctx) => TransferSheet(
        progress: progress,
        onAccept: () {},
        onDecline: () {},
        onCancel: () => Navigator.pop(ctx),
      ),
    );
  }

  void _showActiveProgress(BuildContext context, TransferProgress initialP) {
    showModalBottomSheet(
      context: context,
      isScrollControlled: true,
      backgroundColor: Colors.transparent,
      enableDrag: true,
      barrierColor: Colors.black.withOpacity(0.5),
      builder: (ctx) => Observer(
        builder: (_) {
          // Live-track the progress from the store
          TransferProgress? current;
          try {
            if (initialP.fileId != null) {
              current = fastShareStore.activeIncoming.firstWhere(
                (p) => p.fileId == initialP.fileId,
              );
            }
          } catch (_) {}
          current ??=
              fastShareStore.outgoingProgress?.fileName == initialP.fileName
              ? fastShareStore.outgoingProgress
              : null;
          current ??= initialP; // Fall back to initial if done

          return TransferSheet(
            progress: current,
            onAccept: () {},
            onDecline: () {},
            onCancel: () {
              if (current?.fileId != null) {
                fastShareStore.handleCancelTransfer(current!.fileId!);
              }
              Navigator.pop(ctx);
            },
            onPause: () {
              if (current?.fileId != null) {
                fastShareStore.handlePauseTransfer(current!.fileId!);
              }
            },
          );
        },
      ),
    );
  }
}
