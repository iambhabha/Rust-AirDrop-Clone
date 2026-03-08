import 'package:flutter/material.dart';
import 'package:flutter_mobx/flutter_mobx.dart';
import 'home_screen.dart';

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
      backgroundColor: Colors.black,
      appBar: AppBar(
        title: const Text(
          'Transfer History',
          style: TextStyle(fontWeight: FontWeight.bold),
        ),
        backgroundColor: Colors.transparent,
        elevation: 0,
        bottom: TabBar(
          controller: _tabController,
          indicatorColor: const Color(0xFF9000FF),
          indicatorWeight: 3,
          labelStyle: const TextStyle(fontWeight: FontWeight.bold),
          tabs: const [
            Tab(text: 'All'),
            Tab(text: 'Received'),
            Tab(text: 'Sent'),
          ],
        ),
        actions: [
          IconButton(
            icon: const Icon(Icons.refresh),
            onPressed: () => fastShareStore.loadHistory(),
          ),
        ],
      ),
      body: Observer(
        builder: (_) {
          if (fastShareStore.isHistoryLoading) {
            return const Center(
              child: CircularProgressIndicator(color: Color(0xFF9000FF)),
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

  Widget _buildHistoryList(List<dynamic> items) {
    if (items.isEmpty) {
      return Center(
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            Icon(Icons.history, size: 64, color: Colors.white.withOpacity(0.1)),
            const SizedBox(height: 16),
            const Text(
              'No transfers found',
              style: TextStyle(color: Colors.white54, fontSize: 16),
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
          duration: Duration(milliseconds: 400 + (index.clamp(0, 10) * 50)),
          tween: Tween(begin: 0.0, end: 1.0),
          curve: Curves.easeOutCubic,
          builder: (context, value, child) {
            return Transform.translate(
              offset: Offset(0, 20 * (1 - value)),
              child: Opacity(opacity: value, child: child),
            );
          },
          child: Card(
            color: const Color(0xFF1E1E1E),
            elevation: 0,
            shape: RoundedRectangleBorder(
              borderRadius: BorderRadius.circular(16),
              side: BorderSide(color: Colors.white.withOpacity(0.05)),
            ),
            margin: const EdgeInsets.only(bottom: 12),
            child: ListTile(
              contentPadding: const EdgeInsets.symmetric(
                horizontal: 16,
                vertical: 8,
              ),
              leading: Container(
                padding: const EdgeInsets.all(10),
                decoration: BoxDecoration(
                  color:
                      (item.isIncoming ? Colors.tealAccent : Colors.blueAccent)
                          .withOpacity(0.1),
                  shape: BoxShape.circle,
                ),
                child: Icon(
                  item.isIncoming
                      ? Icons.download_rounded
                      : Icons.upload_rounded,
                  color: item.isIncoming
                      ? Colors.tealAccent
                      : Colors.blueAccent,
                  size: 20,
                ),
              ),
              title: Text(
                item.fileName.fileName,
                style: const TextStyle(
                  color: Colors.white,
                  fontWeight: FontWeight.w500,
                ),
              ),
              subtitle: Padding(
                padding: const EdgeInsets.only(top: 4),
                child: Text(
                  '${item.status} • ${item.timestamp}',
                  style: const TextStyle(color: Colors.white54, fontSize: 12),
                ),
              ),
              trailing: const Icon(
                Icons.chevron_right,
                color: Colors.white24,
                size: 20,
              ),
            ),
          ),
        );
      },
    );
  }
}
