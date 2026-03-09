class HistoryItem {
  final String fileName;
  final int size;
  final String status;
  final String timestamp;
  final bool isIncoming;
  final int totalFiles;
  final String? savedPath;
  final double? timeTakenSecs;

  HistoryItem({
    required this.fileName,
    required this.size,
    required this.status,
    required this.timestamp,
    required this.isIncoming,
    this.totalFiles = 1,
    this.savedPath,
    this.timeTakenSecs,
  });

  factory HistoryItem.fromJson(Map<String, dynamic> json) {
    return HistoryItem(
      fileName: json['file_name'] ?? '',
      size: (json['size'] as num?)?.toInt() ?? 0,
      status: json['status'] ?? '',
      timestamp: json['timestamp'] ?? '',
      isIncoming: json['is_incoming'] ?? false,
      totalFiles: (json['total_files'] as num?)?.toInt() ?? 1,
      savedPath: json['saved_path'],
      timeTakenSecs: (json['time_taken_secs'] as num?)?.toDouble(),
    );
  }

  Map<String, dynamic> toJson() {
    return {
      'file_name': fileName,
      'size': size,
      'status': status,
      'timestamp': timestamp,
      'is_incoming': isIncoming,
      'total_files': totalFiles,
      'saved_path': savedPath,
      'time_taken_secs': timeTakenSecs,
    };
  }
}
