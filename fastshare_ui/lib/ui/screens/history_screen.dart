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
              _buildHistoryList(fastShareStore.history),
              _buildHistoryList(
                fastShareStore.history.where((i) => i.isIncoming).toList(),
              ),
              _buildHistoryList(
                fastShareStore.history.where((i) => !i.isIncoming).toList(),
              ),
            ],
          );
        },
      ),
    );
  }

  Widget _buildHistoryList(List<HistoryItem> items) {
    if (items.isEmpty) {
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
    return ListView.builder(
      physics: const BouncingScrollPhysics(),
      padding: const EdgeInsets.all(16),
      itemCount: items.length,
      itemBuilder: (context, index) {
        final item = items[index];
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
      },
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
}
