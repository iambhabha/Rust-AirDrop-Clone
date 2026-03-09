class TransferProgress {
  final String fileName;
  final double progress;
  final int totalBytes;
  final int receivedBytes;
  final String? fileId;
  final String? fromAddr;
  final String? speed;
  final String? status;
  final int? totalFiles;

  final int? throughputBps;
  final double? batchProgress;
  final bool? isPaused;
  final String? savedPath;

  TransferProgress({
    required this.fileName,
    required this.progress,
    required this.totalBytes,
    required this.receivedBytes,
    this.fileId,
    this.fromAddr,
    this.speed,
    this.status,
    this.totalFiles,
    this.throughputBps,
    this.batchProgress,
    this.isPaused,
    this.savedPath,
  });

  factory TransferProgress.fromJson(Map<String, dynamic> json) {
    return TransferProgress(
      fileName: json['file_name'] ?? '',
      progress: (json['progress'] as num?)?.toDouble() ?? 0.0,
      totalBytes: (json['total_bytes'] as num?)?.toInt() ?? 0,
      receivedBytes: (json['received_bytes'] as num?)?.toInt() ?? 0,
      fileId: json['file_id'],
      fromAddr: json['from_addr'],
      speed: json['speed'],
      status: json['status'],
      totalFiles: (json['total_files'] as num?)?.toInt(),
      throughputBps: (json['throughput_bps'] as num?)?.toInt(),
      batchProgress: (json['batch_progress'] as num?)?.toDouble(),
      isPaused: json['is_paused'] as bool?,
      savedPath: json['saved_path'],
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
