// Offline service interface for managing offline document access and sync queue
// Provides abstraction over sync datasource for offline-first functionality

import 'dart:async';
import 'dart:typed_data';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:connectivity_plus/connectivity_plus.dart';
import 'package:miniwiki/data/models/sync_queue_item.dart';
import 'package:miniwiki/data/datasources/pending_sync_datasource.dart';

/// Offline service state
class OfflineServiceState {
  final bool isOnline;
  final int pendingQueueCount;
  final int failedQueueCount;
  final int cachedDocumentsCount;
  final int cacheSizeBytes;
  final ConnectivityResult connectionType;
  final bool isSyncing;
  final String? lastSyncError;
  final DateTime? lastSuccessfulSync;

  const OfflineServiceState({
    required this.isOnline,
    required this.pendingQueueCount,
    required this.failedQueueCount,
    required this.cachedDocumentsCount,
    required this.cacheSizeBytes,
    required this.connectionType,
    required this.isSyncing,
    this.lastSyncError,
    this.lastSuccessfulSync,
  });

  factory OfflineServiceState.initial() => const OfflineServiceState(
        isOnline: true,
        pendingQueueCount: 0,
        failedQueueCount: 0,
        cachedDocumentsCount: 0,
        cacheSizeBytes: 0,
        connectionType: ConnectivityResult.wifi,
        isSyncing: false,
      );

  OfflineServiceState copyWith({
    bool? isOnline,
    int? pendingQueueCount,
    int? failedQueueCount,
    int? cachedDocumentsCount,
    int? cacheSizeBytes,
    ConnectivityResult? connectionType,
    bool? isSyncing,
    String? lastSyncError,
    DateTime? lastSuccessfulSync,
  }) =>
      OfflineServiceState(
        isOnline: isOnline ?? this.isOnline,
        pendingQueueCount: pendingQueueCount ?? this.pendingQueueCount,
        failedQueueCount: failedQueueCount ?? this.failedQueueCount,
        cachedDocumentsCount: cachedDocumentsCount ?? this.cachedDocumentsCount,
        cacheSizeBytes: cacheSizeBytes ?? this.cacheSizeBytes,
        connectionType: connectionType ?? this.connectionType,
        isSyncing: isSyncing ?? this.isSyncing,
        lastSyncError: lastSyncError ?? this.lastSyncError,
        lastSuccessfulSync: lastSuccessfulSync ?? this.lastSuccessfulSync,
      );
}

/// Offline service for managing offline-first functionality
class OfflineService {
  final PendingSyncDatasource _syncDatasource;
  final Connectivity _connectivity;
  final StreamController<OfflineServiceState> _stateController =
      StreamController<OfflineServiceState>.broadcast();
  StreamSubscription<List<ConnectivityResult>>? _connectivitySubscription;

  OfflineServiceState _state = OfflineServiceState.initial();

  OfflineService({
    required PendingSyncDatasource syncDatasource,
    required Connectivity connectivity,
  })  : _syncDatasource = syncDatasource,
        _connectivity = connectivity;

  /// Initialize offline service
  Future<void> initialize() async {
    // Subscribe to connectivity changes
    _connectivitySubscription =
        _connectivity.onConnectivityChanged.listen(_onConnectivityChanged);

    // Get initial connectivity status
    final result = await _connectivity.checkConnectivity();
    final results = result;
    final isOnline = results.any((r) => r != ConnectivityResult.none);
    final connectionType = results.firstWhere(
      (r) => r != ConnectivityResult.none,
      orElse: () => ConnectivityResult.none,
    );

    _state = _state.copyWith(
      isOnline: isOnline,
      connectionType: connectionType,
    );
    _stateController.add(_state);

    if (isOnline) {
      _triggerSyncOnReconnect();
    }
  }

  /// Stream of state changes
  Stream<OfflineServiceState> get stateChanges => _stateController.stream;

  /// Current state
  OfflineServiceState get currentState => _state;

  /// Handle connectivity changes
  void _onConnectivityChanged(List<ConnectivityResult> results) {
    final wasOnline = _state.isOnline;
    final isOnline = results.any((r) => r != ConnectivityResult.none);
    final connectionType = results.firstWhere(
      (r) => r != ConnectivityResult.none,
      orElse: () => ConnectivityResult.none,
    );

    _state = _state.copyWith(
      isOnline: isOnline,
      connectionType: connectionType,
    );
    _stateController.add(_state);

    if (wasOnline != isOnline && isOnline) {
      _triggerSyncOnReconnect();
    }
  }

  /// Trigger sync when coming back online
  void _triggerSyncOnReconnect() {
    // Hook for sync service to listen and trigger sync
  }

  /// Add item to offline queue
  Future<void> addToQueue({
    required String documentId,
    required String operation,
    required Map<String, dynamic> data,
  }) async {
    final item = SyncQueueItem()
      ..entityType = 'document'
      ..entityId = documentId
      ..operation = operation
      ..data = data
      ..createdAt = DateTime.now()
      ..retryCount = 0;

    await _syncDatasource.addToQueue(item);
    await _updateState();
  }

  /// Get next pending queue item
  Future<SyncQueueItem?> getNextPendingItem() async =>
      await _syncDatasource.getNextPendingItem();

  /// Mark queue item as synced
  Future<void> markQueueItemSynced(String itemId) async {
    await _syncDatasource.markAsSynced(itemId);
    await _updateState();
  }

  /// Mark queue item as failed
  Future<void> markQueueItemFailed(String itemId, String error) async {
    await _syncDatasource.markAsFailed(itemId, error);
    await _updateState();
  }

  /// Remove queue item
  Future<void> removeQueueItem(String itemId) async {
    await _syncDatasource.removeFromQueue(itemId);
    await _updateState();
  }

  /// Clear all completed/failed queue items
  Future<void> clearQueue({bool onlyCompleted = false}) async {
    await _syncDatasource.clearQueue(onlyCompleted: onlyCompleted);
    await _updateState();
  }

  /// Get queue count
  Future<int> getQueueCount() async => await _syncDatasource.getQueueCount();

  /// Get pending queue count
  Future<int> getPendingQueueCount() async =>
      await _syncDatasource.getPendingCount();

  /// Get failed queue count
  Future<int> getFailedQueueCount() async =>
      await _syncDatasource.getFailedCount();

  /// Cache a document for offline access
  Future<void> cacheDocument({
    required String documentId,
    required Uint8List content,
  }) async {
    await _syncDatasource.cacheDocument(documentId, content);
    await _updateState();
  }

  /// Get cached document
  Future<Uint8List?> getCachedDocument(String documentId) async =>
      await _syncDatasource.getCachedDocument(documentId);

  /// Remove cached document
  Future<void> removeCachedDocument(String documentId) async {
    await _syncDatasource.removeCachedDocument(documentId);
    await _updateState();
  }

  /// Get all cached document IDs
  Future<List<String>> getAllCachedDocumentIds() async =>
      await _syncDatasource.getAllCachedDocumentIds();

  /// Get cache size in bytes
  Future<int> getCacheSize() async => await _syncDatasource.getCacheSize();

  /// Clear all cached documents
  Future<void> clearCache() async {
    await _syncDatasource.clearCache();
    await _updateState();
  }

  /// Update service state
  Future<void> _updateState() async {
    final pendingCount = await getPendingQueueCount();
    final failedCount = await getFailedQueueCount();
    final cachedCount = (await getAllCachedDocumentIds()).length;
    final cacheSize = await getCacheSize();

    _state = _state.copyWith(
      pendingQueueCount: pendingCount,
      failedQueueCount: failedCount,
      cachedDocumentsCount: cachedCount,
      cacheSizeBytes: cacheSize,
    );
    _stateController.add(_state);
  }

  /// Set syncing status
  void setSyncing(bool isSyncing) {
    _state = _state.copyWith(isSyncing: isSyncing);
    _stateController.add(_state);
  }

  /// Set last sync error
  void setLastSyncError(String? error) {
    _state = _state.copyWith(lastSyncError: error);
    _stateController.add(_state);
  }

  /// Set last successful sync time
  void setLastSuccessfulSync(DateTime time) {
    _state = _state.copyWith(
      lastSuccessfulSync: time,
    );
    _stateController.add(_state);
  }

  /// Check if online
  bool get isOnline => _state.isOnline;

  /// Get connection type
  ConnectivityResult get connectionType => _state.connectionType;

  /// Dispose of the service
  void dispose() {
    _connectivitySubscription?.cancel();
    _stateController.close();
  }
}

/// Provider for OfflineService
final offlineServiceProvider = Provider<OfflineService>((ref) {
  final syncDatasource = ref.watch(pendingSyncDatasourceProvider);
  final connectivity = Connectivity();
  return OfflineService(
    syncDatasource: syncDatasource,
    connectivity: connectivity,
  );
});
