import 'dart:async';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:connectivity_plus/connectivity_plus.dart';
import 'package:miniwiki/core/network/api_client.dart';
import 'package:miniwiki/services/crdt_service.dart';
import 'package:miniwiki/data/datasources/pending_sync_datasource.dart';

/// Sync status enumeration
enum SyncStatus {
  pending,
  syncing,
  completed,
  failed,
}

/// Sync event types
enum SyncEventType {
  started,
  success,
  error,
  completed,
  online,
  offline,
  queueProcessed,
}

/// Sync event
class SyncEvent {
  final SyncEventType type;
  final String? documentId;
  final String? message;
  final DateTime timestamp;

  const SyncEvent({
    required this.type,
    required this.timestamp,
    this.documentId,
    this.message,
  });
}

/// Sync result
class SyncResult {
  final bool success;
  final String? errorMessage;
  final int documentsSynced;

  const SyncResult({
    required this.success,
    this.errorMessage,
    this.documentsSynced = 0,
  });
}

/// Sync summary
class SyncSummary {
  final bool success;
  final int syncedCount;
  final int failedCount;
  final DateTime timestamp;

  const SyncSummary({
    required this.success,
    required this.syncedCount,
    required this.failedCount,
    required this.timestamp,
  });

  SyncSummary copyWith({
    bool? success,
    int? syncedCount,
    int? failedCount,
    DateTime? timestamp,
  }) =>
      SyncSummary(
        success: success ?? this.success,
        syncedCount: syncedCount ?? this.syncedCount,
        failedCount: failedCount ?? this.failedCount,
        timestamp: timestamp ?? this.timestamp,
      );
}

/// Sync service for handling offline-first synchronization
class SyncService {
  final CrdtService _crdtService;
  final PendingSyncDatasource _syncDatasource;
  final ApiClient _apiClient;

  final StreamController<SyncEvent> _syncEventsController =
      StreamController<SyncEvent>.broadcast();
  final Map<String, Timer> _syncTimers = {};

  /// Stream of sync events
  Stream<SyncEvent> get syncEvents => _syncEventsController.stream;

  /// Current connectivity status
  List<ConnectivityResult>? _connectivityStatus;

  /// Connectivity subscription for cleanup
  StreamSubscription<List<ConnectivityResult>>? _connectivitySubscription;

  /// Whether automatic sync is enabled
  bool _autoSyncEnabled = true;

  /// Sync interval in seconds
  int _syncIntervalSeconds = 30;

  /// Sync queue worker timer
  Timer? _queueWorkerTimer;

  /// Whether the queue worker is running
  bool _queueWorkerRunning = false;

  /// Failed sync count (current session)
  int _failedCount = 0;

  /// Total failed count (historical)
  int _totalFailedCount = 0;

  /// Sync service constructor
  SyncService(this._crdtService, this._syncDatasource, this._apiClient);

  /// Initialize sync service
  Future<void> initialize() async {
    _connectivitySubscription =
        Connectivity().onConnectivityChanged.listen(_onConnectivityChanged);
    _connectivityStatus = await Connectivity().checkConnectivity();
    _startQueueWorker();

    if (_autoSyncEnabled && _isOnline) {
      _startAutoSync();
    }
  }

  /// Check if online
  bool get _isOnline {
    return _connectivityStatus?.any((r) => r != ConnectivityResult.none) ??
        false;
  }

  /// Handle connectivity changes
  void _onConnectivityChanged(List<ConnectivityResult> results) {
    _connectivityStatus = results;
    final isOnline = _isOnline;
    _syncEventsController.add(SyncEvent(
      type: isOnline ? SyncEventType.online : SyncEventType.offline,
      timestamp: DateTime.now(),
    ));

    if (isOnline && _autoSyncEnabled) {
      _startAutoSync();
    }
  }

  /// Start auto-sync timer
  void _startAutoSync() {
    _syncTimers['autoSync']?.cancel();
    _syncTimers['autoSync'] = Timer.periodic(
      Duration(seconds: _syncIntervalSeconds),
      (_) => _processQueue(),
    );
  }

  /// Start queue worker
  void _startQueueWorker() {
    if (_queueWorkerRunning) return;
    _queueWorkerRunning = true;
    _processQueue();
  }

  /// Process sync queue
  Future<void> _processQueue() async {
    if (!_isOnline) return;

    final items = _syncDatasource.getPendingItems();
    if (items.isEmpty) return;

    _syncEventsController.add(SyncEvent(
      type: SyncEventType.started,
      timestamp: DateTime.now(),
      message: 'Processing ${items.length} items',
    ));

    int successCount = 0;
    int failedCount = 0;

    for (final item in items) {
      final entityType = item['entityType'] as String;
      final entityId = item['entityId'] as String;
      final operation = item['operation'] as String;
      final data = item['data'] as Map<String, dynamic>?;

      try {
        bool success = false;

        switch (entityType) {
          case 'document':
            success = await _syncDocument(entityId, operation, data ?? {});
            break;
          default:
            print('[SyncService] Unknown entity type: $entityType');
        }

        if (success) {
          _syncDatasource.removeFromQueue(entityType, entityId);
          successCount++;
        } else {
          throw Exception('Sync returned false');
        }
      } catch (e) {
        print('[SyncService] Failed to sync $entityType:$entityId - $e');
        _syncDatasource.removeFromQueue(entityType, entityId);
        failedCount++;
        _failedCount++;
        _totalFailedCount++;
      }
    }

    _syncEventsController.add(SyncEvent(
      type: SyncEventType.completed,
      timestamp: DateTime.now(),
      message: 'Success: $successCount, Failed: $failedCount',
    ));
  }

  /// Sync a document item
  Future<bool> _syncDocument(
    String documentId,
    String operation,
    Map<String, dynamic> data,
  ) async {
    try {
      switch (operation) {
        case 'create':
          final spaceId = data['spaceId'] as String?;
          if (spaceId == null) {
            print(
                '[SyncService] Cannot create document: missing spaceId in data: $data');
            return false;
          }
          await _apiClient.post('/spaces/$spaceId/documents', data: data);
          return true;

        case 'update':
          await _apiClient.patch('/documents/$documentId', data: data);
          return true;

        case 'delete':
          await _apiClient.delete('/documents/$documentId');
          return true;

        default:
          print('[SyncService] Unknown operation: $operation');
          return false;
      }
    } catch (e) {
      print('[SyncService] Document sync failed: $e');
      rethrow;
    }
  }

  /// Queue document for sync
  Future<void> queueDocumentForSync(
      String documentId, String operation, Map<String, dynamic> data) async {
    await _syncDatasource.addToQueue('document', documentId, operation, data);
  }

  /// Get pending sync count
  int getPendingCount() {
    return _syncDatasource.getQueueSize();
  }

  /// Get failed sync count
  int getFailedCount() {
    return _failedCount;
  }

  /// Get total failed count (historical)
  int getTotalFailedCount() {
    return _totalFailedCount;
  }

  /// Enable/disable auto-sync
  void setAutoSync(bool enabled) {
    _autoSyncEnabled = enabled;
    if (enabled && _isOnline) {
      _startAutoSync();
    } else {
      _syncTimers['autoSync']?.cancel();
    }
  }

  /// Dispose
  void dispose() {
    _connectivitySubscription?.cancel();
    _syncTimers.values.forEach((t) => t.cancel());
    _queueWorkerTimer?.cancel();
    _syncEventsController.close();
  }
}

/// Provider for SyncService
final syncServiceProvider = Provider<SyncService>((ref) {
  final crdtService = ref.watch(crdtServiceProvider);
  final syncDatasource = ref.watch(pendingSyncDatasourceProvider);
  return SyncService(crdtService, syncDatasource, ApiClient.defaultInstance());
});
