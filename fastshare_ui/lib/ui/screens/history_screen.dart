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
      appBar: AppBar(
        title: const Text('Transfer History'),
        bottom: TabBar(
          controller: _tabController,
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
            return const Center(child: CircularProgressIndicator());
          }

          return TabBarView(
            controller: _tabController,
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
      return const Center(
        child: Text(
          'No transfers found',
          style: TextStyle(color: Colors.white54),
        ),
      );
    }
    return ListView.builder(
      padding: const EdgeInsets.all(16),
      itemCount: items.length,
      itemBuilder: (context, index) {
        final item = items[index];
        return Card(
          color: const Color(0xFF1E293B),
          margin: const EdgeInsets.only(bottom: 12),
          child: ListTile(
            leading: Icon(
              item.isIncoming ? Icons.download_rounded : Icons.upload_rounded,
              color: item.isIncoming ? Colors.tealAccent : Colors.blueAccent,
            ),
            title: Text(item.fileName.fileName),
            subtitle: Text('${item.status} • ${item.timestamp}'),
          ),
        );
      },
    );
  }
}
