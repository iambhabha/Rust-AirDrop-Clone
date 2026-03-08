// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'fastshare_store.dart';

// **************************************************************************
// StoreGenerator
// **************************************************************************

// ignore_for_file: non_constant_identifier_names, unnecessary_brace_in_string_interps, unnecessary_lambdas, prefer_expression_function_bodies, lines_longer_than_80_chars, avoid_as, avoid_annotating_with_dynamic, no_leading_underscores_for_local_identifiers

mixin _$FastShareStore on _FastShareStore, Store {
  late final _$isEngineRunningAtom = Atom(
    name: '_FastShareStore.isEngineRunning',
    context: context,
  );

  @override
  bool get isEngineRunning {
    _$isEngineRunningAtom.reportRead();
    return super.isEngineRunning;
  }

  @override
  set isEngineRunning(bool value) {
    _$isEngineRunningAtom.reportWrite(value, super.isEngineRunning, () {
      super.isEngineRunning = value;
    });
  }

  late final _$nearbyDevicesAtom = Atom(
    name: '_FastShareStore.nearbyDevices',
    context: context,
  );

  @override
  ObservableList<DeviceInfo> get nearbyDevices {
    _$nearbyDevicesAtom.reportRead();
    return super.nearbyDevices;
  }

  @override
  set nearbyDevices(ObservableList<DeviceInfo> value) {
    _$nearbyDevicesAtom.reportWrite(value, super.nearbyDevices, () {
      super.nearbyDevices = value;
    });
  }

  late final _$savedDevicesAtom = Atom(
    name: '_FastShareStore.savedDevices',
    context: context,
  );

  @override
  ObservableMap<String, DeviceInfo> get savedDevices {
    _$savedDevicesAtom.reportRead();
    return super.savedDevices;
  }

  @override
  set savedDevices(ObservableMap<String, DeviceInfo> value) {
    _$savedDevicesAtom.reportWrite(value, super.savedDevices, () {
      super.savedDevices = value;
    });
  }

  late final _$activeIncomingAtom = Atom(
    name: '_FastShareStore.activeIncoming',
    context: context,
  );

  @override
  ObservableList<TransferProgress> get activeIncoming {
    _$activeIncomingAtom.reportRead();
    return super.activeIncoming;
  }

  @override
  set activeIncoming(ObservableList<TransferProgress> value) {
    _$activeIncomingAtom.reportWrite(value, super.activeIncoming, () {
      super.activeIncoming = value;
    });
  }

  late final _$outgoingProgressAtom = Atom(
    name: '_FastShareStore.outgoingProgress',
    context: context,
  );

  @override
  Map<String, dynamic>? get outgoingProgress {
    _$outgoingProgressAtom.reportRead();
    return super.outgoingProgress;
  }

  @override
  set outgoingProgress(Map<String, dynamic>? value) {
    _$outgoingProgressAtom.reportWrite(value, super.outgoingProgress, () {
      super.outgoingProgress = value;
    });
  }

  late final _$isScanningAtom = Atom(
    name: '_FastShareStore.isScanning',
    context: context,
  );

  @override
  bool get isScanning {
    _$isScanningAtom.reportRead();
    return super.isScanning;
  }

  @override
  set isScanning(bool value) {
    _$isScanningAtom.reportWrite(value, super.isScanning, () {
      super.isScanning = value;
    });
  }

  late final _$isSendingAtom = Atom(
    name: '_FastShareStore.isSending',
    context: context,
  );

  @override
  bool get isSending {
    _$isSendingAtom.reportRead();
    return super.isSending;
  }

  @override
  set isSending(bool value) {
    _$isSendingAtom.reportWrite(value, super.isSending, () {
      super.isSending = value;
    });
  }

  late final _$checksumEnabledAtom = Atom(
    name: '_FastShareStore.checksumEnabled',
    context: context,
  );

  @override
  bool get checksumEnabled {
    _$checksumEnabledAtom.reportRead();
    return super.checksumEnabled;
  }

  @override
  set checksumEnabled(bool value) {
    _$checksumEnabledAtom.reportWrite(value, super.checksumEnabled, () {
      super.checksumEnabled = value;
    });
  }

  late final _$compressionEnabledAtom = Atom(
    name: '_FastShareStore.compressionEnabled',
    context: context,
  );

  @override
  bool get compressionEnabled {
    _$compressionEnabledAtom.reportRead();
    return super.compressionEnabled;
  }

  @override
  set compressionEnabled(bool value) {
    _$compressionEnabledAtom.reportWrite(value, super.compressionEnabled, () {
      super.compressionEnabled = value;
    });
  }

  late final _$historyAtom = Atom(
    name: '_FastShareStore.history',
    context: context,
  );

  @override
  ObservableList<HistoryItem> get history {
    _$historyAtom.reportRead();
    return super.history;
  }

  @override
  set history(ObservableList<HistoryItem> value) {
    _$historyAtom.reportWrite(value, super.history, () {
      super.history = value;
    });
  }

  late final _$isHistoryLoadingAtom = Atom(
    name: '_FastShareStore.isHistoryLoading',
    context: context,
  );

  @override
  bool get isHistoryLoading {
    _$isHistoryLoadingAtom.reportRead();
    return super.isHistoryLoading;
  }

  @override
  set isHistoryLoading(bool value) {
    _$isHistoryLoadingAtom.reportWrite(value, super.isHistoryLoading, () {
      super.isHistoryLoading = value;
    });
  }

  late final _$initAsyncAction = AsyncAction(
    '_FastShareStore.init',
    context: context,
  );

  @override
  Future<void> init() {
    return _$initAsyncAction.run(() => super.init());
  }

  late final _$_loadInitialDataAsyncAction = AsyncAction(
    '_FastShareStore._loadInitialData',
    context: context,
  );

  @override
  Future<void> _loadInitialData() {
    return _$_loadInitialDataAsyncAction.run(() => super._loadInitialData());
  }

  late final _$_startBackendAsyncAction = AsyncAction(
    '_FastShareStore._startBackend',
    context: context,
  );

  @override
  Future<void> _startBackend() {
    return _$_startBackendAsyncAction.run(() => super._startBackend());
  }

  late final _$_refreshDevicesAsyncAction = AsyncAction(
    '_FastShareStore._refreshDevices',
    context: context,
  );

  @override
  Future<void> _refreshDevices() {
    return _$_refreshDevicesAsyncAction.run(() => super._refreshDevices());
  }

  late final _$_saveDeviceInfoAsyncAction = AsyncAction(
    '_FastShareStore._saveDeviceInfo',
    context: context,
  );

  @override
  Future<void> _saveDeviceInfo(DeviceInfo device) {
    return _$_saveDeviceInfoAsyncAction.run(
      () => super._saveDeviceInfo(device),
    );
  }

  late final _$_updateProgressAsyncAction = AsyncAction(
    '_FastShareStore._updateProgress',
    context: context,
  );

  @override
  Future<void> _updateProgress() {
    return _$_updateProgressAsyncAction.run(() => super._updateProgress());
  }

  late final _$sendFilesAsyncAction = AsyncAction(
    '_FastShareStore.sendFiles',
    context: context,
  );

  @override
  Future<String> sendFiles(List<String> paths, String targetIp) {
    return _$sendFilesAsyncAction.run(() => super.sendFiles(paths, targetIp));
  }

  late final _$loadHistoryAsyncAction = AsyncAction(
    '_FastShareStore.loadHistory',
    context: context,
  );

  @override
  Future<void> loadHistory() {
    return _$loadHistoryAsyncAction.run(() => super.loadHistory());
  }

  late final _$_FastShareStoreActionController = ActionController(
    name: '_FastShareStore',
    context: context,
  );

  @override
  void setChecksum(bool enabled) {
    final _$actionInfo = _$_FastShareStoreActionController.startAction(
      name: '_FastShareStore.setChecksum',
    );
    try {
      return super.setChecksum(enabled);
    } finally {
      _$_FastShareStoreActionController.endAction(_$actionInfo);
    }
  }

  @override
  void setCompression(bool enabled) {
    final _$actionInfo = _$_FastShareStoreActionController.startAction(
      name: '_FastShareStore.setCompression',
    );
    try {
      return super.setCompression(enabled);
    } finally {
      _$_FastShareStoreActionController.endAction(_$actionInfo);
    }
  }

  @override
  String toString() {
    return '''
isEngineRunning: ${isEngineRunning},
nearbyDevices: ${nearbyDevices},
savedDevices: ${savedDevices},
activeIncoming: ${activeIncoming},
outgoingProgress: ${outgoingProgress},
isScanning: ${isScanning},
isSending: ${isSending},
checksumEnabled: ${checksumEnabled},
compressionEnabled: ${compressionEnabled},
history: ${history},
isHistoryLoading: ${isHistoryLoading}
    ''';
  }
}
