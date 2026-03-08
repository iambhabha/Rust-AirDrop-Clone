class TransferProgress {
  final String fileName;
  final double progress;
  final int totalBytes;
  final int receivedBytes;
  final String? fileId;
  final String? fromAddr;

  TransferProgress({
    required this.fileName,
    required this.progress,
    required this.totalBytes,
    required this.receivedBytes,
    this.fileId,
    this.fromAddr,
  });

  factory TransferProgress.fromJson(Map<String, dynamic> json) {
    return TransferProgress(
      fileName: json['file_name'] ?? '',
      progress: (json['progress'] as num?)?.toDouble() ?? 0.0,
      totalBytes: json['total_bytes'] ?? 0,
      receivedBytes: json['received_bytes'] ?? 0,
      fileId: json['file_id'],
      fromAddr: json['from_addr'],
    );
  }
}

class PendingIncoming {
  final String fileId;
  final String fromAddr;
  final String fileName;
  final int totalFiles;
  final double totalSize;

  PendingIncoming({
    required this.fileId,
    required this.fromAddr,
    required this.fileName,
    required this.totalFiles,
    required this.totalSize,
  });

  factory PendingIncoming.fromJson(Map<String, dynamic> json) {
    return PendingIncoming(
      fileId: json['file_id'] ?? '',
      fromAddr: json['from_addr'] ?? '',
      fileName: json['file_name'] ?? '',
      totalFiles: (json['total_files'] as num?)?.toInt() ?? 1,
      totalSize: (json['total_size'] as num?)?.toDouble() ?? 0.0,
    );
  }
}
