import 'dart:async';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:connectivity_plus/connectivity_plus.dart';
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

  /// Sync service constructor
  SyncService(this._crdtService, this._syncDatasource);

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
      try {
        // Process item - in real implementation, this would call the API
        await Future.delayed(const Duration(milliseconds: 100));
        _syncDatasource.removeFromQueue(
          item['entityType'] as String,
          item['entityId'] as String,
        );
        successCount++;
      } catch (_) {
        _syncDatasource.removeFromQueue(
          item['entityType'] as String,
          item['entityId'] as String,
        );
        failedCount++;
      }
    }

    _syncEventsController.add(SyncEvent(
      type: SyncEventType.completed,
      timestamp: DateTime.now(),
      message: 'Success: $successCount, Failed: $failedCount',
    ));
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
    return 0; // Simplified - all items are removed after processing
  }

  /// Get total failed count (historical)
  int getTotalFailedCount() {
    return 0; // Simplified
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
  return SyncService(crdtService, syncDatasource);
});
