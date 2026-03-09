import 'dart:async';
import 'dart:convert';
import 'dart:io';
import 'package:flutter/foundation.dart';
import 'package:mobx/mobx.dart';
import 'package:path_provider/path_provider.dart';
import 'package:shared_preferences/shared_preferences.dart';
import '../models/device_info.dart';
import '../models/transfer_progress.dart';
import '../models/history_item.dart';
import '../src/rust/api/simple.dart';

part 'fastshare_store.g.dart';

class FastShareStore = _FastShareStore with _$FastShareStore;

abstract class _FastShareStore with Store {
  @observable
  bool isEngineRunning = false;

  @observable
  ObservableList<DeviceInfo> nearbyDevices = ObservableList<DeviceInfo>();

  @observable
  ObservableMap<String, DeviceInfo> savedDevices =
      ObservableMap<String, DeviceInfo>();

  @observable
  ObservableList<TransferProgress> activeIncoming =
      ObservableList<TransferProgress>();

  @observable
  TransferProgress? outgoingProgress;

  @observable
  PendingIncoming? pendingIncoming;

  @observable
  bool isScanning = false;

  @observable
  bool isSending = false;

  @observable
  bool checksumEnabled = false;

  @observable
  bool compressionEnabled = false;

  @observable
  ObservableList<HistoryItem> history = ObservableList<HistoryItem>();

  @observable
  bool isHistoryLoading = false;

  Timer? _discoveryTimer;
  Timer? _pollTimer;

  @action
  Future<void> init() async {
    await _loadInitialData();
    await _startBackend();
    await loadHistory();
  }

  @action
  Future<void> _loadInitialData() async {
    final prefs = await SharedPreferences.getInstance();
    final String? devicesJson = prefs.getString('saved_devices');
    if (devicesJson != null) {
      final Map<String, dynamic> deco = jsonDecode(devicesJson);
      savedDevices.addAll(
        deco.map((key, value) => MapEntry(key, DeviceInfo.fromJson(value))),
      );
    }
    checksumEnabled = getChecksumEnabled();
    compressionEnabled = getCompressionEnabled();
  }

  @action
  Future<void> _startBackend() async {
    isEngineRunning = true;
    try {
      final downloadPath = Platform.isAndroid
          ? '/storage/emulated/0/Download/Rust Drop'
          : (await getExternalStorageDirectory() ??
                        await getApplicationDocumentsDirectory())
                    .path +
                '/Rust Drop';
      final tempPath = (await getTemporaryDirectory()).path + '/Rust Drop/temp';
      await Directory(downloadPath).create(recursive: true);
      await Directory(tempPath).create(recursive: true);
      await startFastshare(downloadPath: downloadPath, tempPath: tempPath);
    } catch (e) {
      debugPrint("Backend start error: $e");
    }

    _discoveryTimer = Timer.periodic(
      const Duration(seconds: 10),
      (_) => _refreshDevices(),
    );
    _pollTimer = Timer.periodic(const Duration(milliseconds: 100), (_) {
      _updateProgress();
    });
  }

  @action
  Future<void> _refreshDevices() async {
    if (isScanning) return;
    try {
      final devicesJson = await getNearbyDevices();
      final List<dynamic> devices = jsonDecode(devicesJson);
      final List<DeviceInfo> newDevices = devices
          .map((d) => DeviceInfo.fromJson({...d, 'is_online': true}))
          .toList();

      nearbyDevices.clear();
      nearbyDevices.addAll(newDevices);

      for (var d in newDevices) {
        _saveDeviceInfo(d);
      }
    } catch (e, s) {
      debugPrint("Refresh error: $e, $s");
    }
  }

  @action
  Future<void> _saveDeviceInfo(DeviceInfo device) async {
    final prefs = await SharedPreferences.getInstance();
    savedDevices[device.deviceName] = DeviceInfo(
      deviceName: device.deviceName,
      ipAddress: device.ipAddress,
      isOnline: false,
      lastSeen: DateTime.now(),
    );
    await prefs.setString(
      'saved_devices',
      jsonEncode(savedDevices.map((k, v) => MapEntry(k, v.toJson()))),
    );
  }

  @action
  Future<void> _updateProgress() async {
    if (!isEngineRunning) return;

    // Incoming
    final pIn = await getIncomingProgress();
    final List<dynamic> decoIn = jsonDecode(pIn);
    activeIncoming.clear();
    activeIncoming.addAll(decoIn.map((e) => TransferProgress.fromJson(e)));

    // Pending Incoming
    final pPend = await getPendingIncoming();
    if (pPend != "null") {
      pendingIncoming = PendingIncoming.fromJson(jsonDecode(pPend));
    } else {
      pendingIncoming = null;
    }

    // Outgoing
    if (isSending) {
      final pOut = await getOutgoingProgress();
      if (pOut != "null") {
        outgoingProgress = TransferProgress.fromJson(jsonDecode(pOut));
      } else {
        outgoingProgress = null;
      }
    }
  }

  @action
  void setChecksum(bool enabled) {
    setChecksumEnabled(enabled: enabled);
    checksumEnabled = enabled;
  }

  @action
  void setCompression(bool enabled) {
    setCompressionEnabled(enabled: enabled);
    compressionEnabled = enabled;
  }

  @action
  Future<String> sendFiles(List<String> paths, String targetIp) async {
    isSending = true;
    outgoingProgress = null;
    final res = await sendFilesToIp(filePaths: paths, targetIp: targetIp);
    isSending = false;
    if (res.toLowerCase().contains("success")) {
      await loadHistory();
    }
    return res;
  }

  @action
  Future<void> loadHistory() async {
    isHistoryLoading = true;
    try {
      final jsonStr = await getTransferHistory();
      final List<dynamic> list = jsonDecode(jsonStr);
      history.clear();
      history.addAll(
        list.map((e) => HistoryItem.fromJson(e)).toList().reversed,
      );
    } catch (e) {
      debugPrint("Error loading history: $e");
    } finally {
      isHistoryLoading = false;
    }
  }

  @action
  Future<void> handleCancelTransfer(String fileId) async {
    await cancelTransfer(fileId: fileId);
    await loadHistory();
  }

  @action
  Future<void> handlePauseTransfer(String fileId) async {
    await pauseTransfer(fileId: fileId);
  }

  void dispose() {
    _discoveryTimer?.cancel();
    _pollTimer?.cancel();
  }
}
