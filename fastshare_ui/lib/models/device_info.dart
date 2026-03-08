class DeviceInfo {
  final String deviceName;
  final String ipAddress;
  final bool isOnline;
  final DateTime? lastSeen;

  DeviceInfo({
    required this.deviceName,
    required this.ipAddress,
    this.isOnline = false,
    this.lastSeen,
  });

  factory DeviceInfo.fromJson(Map<String, dynamic> json) {
    return DeviceInfo(
      deviceName: json['device_name'] ?? 'Unknown',
      ipAddress: json['ip_address'] ?? '0.0.0.0',
      isOnline: json['is_online'] ?? false,
      lastSeen: json['last_seen'] != null
          ? DateTime.parse(json['last_seen'])
          : null,
    );
  }

  Map<String, dynamic> toJson() {
    return {
      'device_name': deviceName,
      'ip_address': ipAddress,
      'is_online': isOnline,
      'last_seen': lastSeen?.toIso8601String(),
    };
  }

  String get initial => deviceName.length > 1
      ? deviceName.substring(0, 2).toUpperCase()
      : deviceName.toUpperCase();
}
