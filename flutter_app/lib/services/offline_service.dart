// Offline service interface for managing offline document access and sync queue
// Uses SharedPreferences for Web-compatible local storage

import 'dart:async';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:connectivity_plus/connectivity_plus.dart';
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
    _connectivitySubscription =
        _connectivity.onConnectivityChanged.listen(_onConnectivityChanged);

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
    required String entityType,
    required String entityId,
    required String operation,
    required Map<String, dynamic> data,
  }) async {
    await _syncDatasource.addToQueue(entityType, entityId, operation, data);
    await _updateState();
  }

  /// Get all pending queue items
  List<Map<String, dynamic>> getPendingItems() {
    return _syncDatasource.getPendingItems();
  }

  /// Mark queue item as synced
  Future<void> markQueueItemSynced(String entityType, String entityId) async {
    await _syncDatasource.removeFromQueue(entityType, entityId);
    await _updateState();
  }

  /// Mark queue item as failed
  Future<void> markQueueItemFailed(
      String entityType, String entityId, String error) async {
    print('[OfflineService] Sync failed for $entityType:$entityId - $error');
    await _syncDatasource.removeFromQueue(entityType, entityId);
    _state = _state.copyWith(
      failedQueueCount: _state.failedQueueCount + 1,
    );
    _stateController.add(_state);
  }

  /// Clear all queue items
  Future<void> clearQueue() async {
    await _syncDatasource.clearQueue();
    await _updateState();
  }

  /// Get queue size
  int getQueueSize() {
    return _syncDatasource.getQueueSize();
  }

  /// Cache a document for offline access
  Future<void> cacheDocument({
    required String documentId,
    required Map<String, dynamic> data,
  }) async {
    await _syncDatasource.cacheDocument(documentId, data);
    await _updateState();
  }

  /// Get cached document
  Map<String, dynamic>? getCachedDocument(String documentId) {
    return _syncDatasource.getCachedDocument(documentId);
  }

  /// Remove cached document
  Future<void> removeCachedDocument(String documentId) async {
    await _syncDatasource.removeCachedDocument(documentId);
    await _updateState();
  }

  /// Get all cached document IDs
  List<String> getAllCachedDocumentIds() {
    return _syncDatasource.getCachedDocIds();
  }

  /// Get cache size in bytes (approximate based on serialized JSON sizes)
  int getCacheSize() {
    final docIds = _syncDatasource.getCachedDocIds();
    int totalSize = 0;
    for (final docId in docIds) {
      final content = _syncDatasource.getCachedContent(docId);
      totalSize += content?.length ?? 0;
    }
    return totalSize;
  }

  /// Clear all cached documents
  Future<void> clearCache() async {
    await _syncDatasource.clearDocumentCache();
    await _updateState();
  }

  /// Update service state
  Future<void> _updateState() async {
    final pendingCount = getQueueSize();
    final cachedCount = getAllCachedDocumentIds().length;
    final cacheSize = getCacheSize();

    _state = _state.copyWith(
      pendingQueueCount: pendingCount,
      failedQueueCount: 0,
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
    _state = _state.copyWith(lastSuccessfulSync: time);
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
