import 'dart:async';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:connectivity_plus/connectivity_plus.dart';
import 'package:miniwiki/services/crdt_service.dart';
import 'package:miniwiki/data/datasources/pending_sync_datasource.dart';
import 'package:miniwiki/data/models/sync_queue_item.dart';

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
    this.documentId,
    this.message,
    required this.timestamp,
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
  }) {
    return SyncSummary(
      success: success ?? this.success,
      syncedCount: syncedCount ?? this.syncedCount,
      failedCount: failedCount ?? this.failedCount,
      timestamp: timestamp ?? this.timestamp,
    );
  }
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

  /// Whether automatic sync is enabled
  bool _autoSyncEnabled = true;

  /// Sync interval in seconds
  int _syncIntervalSeconds = 30;

  /// Sync queue worker timer
  Timer? _queueWorkerTimer;

  /// Whether the queue worker is running
  bool _queueWorkerRunning = false;

  /// Sync service constructor
  SyncService(this._crdtService, [PendingSyncDatasource? syncDatasource])
      : _syncDatasource = syncDatasource ?? PendingSyncDatasource(
            isar: throw UnimplementedError(
                'PendingSyncDatasource must be provided'));

  /// Initialize sync service
  Future<void> initialize() async {
    // Subscribe to connectivity changes
    Connectivity().onConnectivityChanged.listen(_onConnectivityChanged);

    // Get initial connectivity status
    _connectivityStatus = await Connectivity().checkConnectivity();

    // Start queue worker
    _startQueueWorker();

    // Start auto-sync if enabled and online
    if (_autoSyncEnabled && _isOnline) {
      _startAutoSync();
    }
  }

  /// Check if currently online
  bool get _isOnline {
    if (_connectivityStatus == null) return false;
    return _connectivityStatus!.isNotEmpty &&
        !_connectivityStatus!.contains(ConnectivityResult.none);
  }

  /// Handle connectivity changes
  void _onConnectivityChanged(List<ConnectivityResult> result) {
    final wasOnline = _isOnline;
    _connectivityStatus = result;
    final isOnline = _isOnline;

    if (wasOnline != isOnline) {
      _syncEventsController.add(SyncEvent(
        type: isOnline ? SyncEventType.online : SyncEventType.offline,
        timestamp: DateTime.now(),
      ));

      if (isOnline) {
        // Came back online, trigger sync
        _startAutoSync();
        // Also process the sync queue immediately
        _processSyncQueue();
      } else {
        // Went offline, stop auto-sync
        _stopAutoSync();
      }
    }
  }

  // ============================================
  // SYNC QUEUE WORKER (T145)
  // ============================================

  /// Start the sync queue worker
  void _startQueueWorker() {
    // Stop any existing worker
    _stopQueueWorker();

    // Run queue worker every 10 seconds
    _queueWorkerTimer = Timer.periodic(
      const Duration(seconds: 10),
      (_) => _processSyncQueue(),
    );
  }

  /// Stop the sync queue worker
  void _stopQueueWorker() {
    _queueWorkerTimer?.cancel();
    _queueWorkerTimer = null;
  }

  /// Process pending items in the sync queue
  Future<void> _processSyncQueue() async {
    // Prevent concurrent processing
    if (_queueWorkerRunning) return;
    if (!_isOnline) return;

    _queueWorkerRunning = true;

    try {
      // Get items ready for retry
      final items = await _syncDatasource.getItemsReadyForRetry();

      if (items.isEmpty) {
        _queueWorkerRunning = false;
        return;
      }

      int processed = 0;
      int failed = 0;

      for (final item in items) {
        try {
          final success = await _processQueueItem(item);
          if (success) {
            processed++;
          } else {
            failed++;
          }
        } catch (e) {
          failed++;
          await _syncDatasource.markAsFailed(item.id.toString(), e.toString());
        }
      }

      if (processed > 0 || failed > 0) {
        _syncEventsController.add(SyncEvent(
          type: SyncEventType.queueProcessed,
          message: 'Processed $processed items, $failed failed',
          timestamp: DateTime.now(),
        ));
      }
    } finally {
      _queueWorkerRunning = false;
    }
  }

  /// Process a single queue item
  Future<bool> _processQueueItem(SyncQueueItem item) async {
    try {
      // Process based on entity type
      switch (item.entityType) {
        case 'document':
          return await _syncDocumentFromQueue(item);
        default:
          // Unknown entity type, mark as failed
          return false;
      }
    } catch (e) {
      await _syncDatasource.markAsFailed(item.id.toString(), e.toString());
      return false;
    }
  }

  /// Sync a document from the queue item
  Future<bool> _syncDocumentFromQueue(SyncQueueItem item) async {
    final documentId = item.entityId;

    // Emit started event
    _syncEventsController.add(SyncEvent(
      type: SyncEventType.started,
      documentId: documentId,
      timestamp: DateTime.now(),
    ));

    try {
      // Get the current state from CRDT
      final update = await _crdtService.getState(documentId);

      if (update == null) {
        // No update to sync, remove from queue
        await _syncDatasource.markAsSynced(item.id.toString());
        return true;
      }

      // TODO: Send update to server API
      // This would be implemented when the backend sync endpoint is ready
      // await apiClient.post('/sync/documents/$documentId', data: item.data);

      // For now, simulate successful sync
      // Mark as synced in CRDT
      _crdtService.markSynced(documentId);

      // Mark queue item as synced (removed)
      await _syncDatasource.markAsSynced(item.id.toString());

      // Emit success event
      _syncEventsController.add(SyncEvent(
        type: SyncEventType.success,
        documentId: documentId,
        timestamp: DateTime.now(),
      ));

      return true;
    } catch (e) {
      // Mark as failed with retry
      await _syncDatasource.markAsFailed(item.id.toString(), e.toString());

      // Emit error event
      _syncEventsController.add(SyncEvent(
        type: SyncEventType.error,
        documentId: documentId,
        message: e.toString(),
        timestamp: DateTime.now(),
      ));

      return false;
    }
  }

  /// Add a document to the sync queue
  Future<void> queueDocumentForSync(String documentId) async {
    final update = await _crdtService.getState(documentId);
    if (update != null) {
      final item = SyncQueueItem()
        ..entityType = 'document'
        ..entityId = documentId
        ..operation = 'update'
        ..data = {'update': update}
        ..createdAt = DateTime.now()
        ..retryCount = 0
        ..priority = 1;

      await _syncDatasource.addToQueue(item);

      // Trigger immediate sync if online
      if (_isOnline) {
        _processSyncQueue();
      }
    }
  }

  // ============================================
  // AUTO SYNC
  // ============================================

  /// Start automatic sync for all dirty documents
  void _startAutoSync() {
    if (!_autoSyncEnabled) return;

    // Cancel existing timer
    _stopAutoSync();

    // Start periodic sync
    _syncTimers['auto'] = Timer.periodic(
      Duration(seconds: _syncIntervalSeconds),
      (_) => syncAllDirtyDocuments(),
    );
  }

  /// Stop automatic sync
  void _stopAutoSync() {
    _syncTimers.remove('auto')?.cancel();
  }

  /// Sync all dirty documents (T146 - triggered on connectivity change)
  Future<SyncSummary> syncAllDirtyDocuments() async {
    if (!_isOnline) {
      _syncEventsController.add(SyncEvent(
        type: SyncEventType.error,
        message: 'Cannot sync while offline',
        timestamp: DateTime.now(),
      ));
      return SyncSummary(
        success: false,
        syncedCount: 0,
        failedCount: 0,
        timestamp: DateTime.now(),
      );
    }

    // First, process the sync queue
    await _processSyncQueue();

    // Then sync any remaining dirty documents
    final dirtyIds = _crdtService.getDirtyDocumentIds();
    var syncedCount = 0;
    var failedCount = 0;

    for (final documentId in dirtyIds) {
      try {
        final result = await syncDocument(documentId);
        if (result.success) {
          syncedCount++;
        } else {
          failedCount++;
        }
      } catch (e) {
        failedCount++;
      }
    }

    if (dirtyIds.isNotEmpty) {
      _syncEventsController.add(SyncEvent(
        type: SyncEventType.completed,
        message: 'Synced $syncedCount documents',
        timestamp: DateTime.now(),
      ));
    }

    return SyncSummary(
      success: failedCount == 0,
      syncedCount: syncedCount,
      failedCount: failedCount,
      timestamp: DateTime.now(),
    );
  }

  /// Sync a single document
  Future<SyncResult> syncDocument(String documentId) async {
    _syncEventsController.add(SyncEvent(
      type: SyncEventType.started,
      documentId: documentId,
      timestamp: DateTime.now(),
    ));

    try {
      // Get the current state
      final update = await _crdtService.getState(documentId);

      if (update == null) {
        return const SyncResult(
          success: true,
          documentsSynced: 0,
        );
      }

      // TODO: Send update to server API
      // This would be implemented when the backend sync endpoint is ready
      // await apiClient.post('/sync/documents/$documentId', data: {'update': update});

      // Mark as synced locally
      _crdtService.markSynced(documentId);

      _syncEventsController.add(SyncEvent(
        type: SyncEventType.success,
        documentId: documentId,
        timestamp: DateTime.now(),
      ));

      return const SyncResult(
        success: true,
        documentsSynced: 1,
      );
    } catch (e) {
      _syncEventsController.add(SyncEvent(
        type: SyncEventType.error,
        documentId: documentId,
        message: e.toString(),
        timestamp: DateTime.now(),
      ));

      return SyncResult(
        success: false,
        errorMessage: e.toString(),
        documentsSynced: 0,
      );
    }
  }

  /// Get pending sync queue count
  Future<int> getPendingSyncCount() async {
    return await _syncDatasource.getPendingCount();
  }

  /// Clear sync queue
  Future<void> clearSyncQueue() async {
    await _syncDatasource.clearQueue();
  }

  /// Enable or disable auto-sync
  void setAutoSync(bool enabled) {
    _autoSyncEnabled = enabled;
    if (enabled && _isOnline) {
      _startAutoSync();
    } else {
      _stopAutoSync();
    }
  }

  /// Set sync interval
  void setSyncInterval(int seconds) {
    _syncIntervalSeconds = seconds;
    if (_autoSyncEnabled && _isOnline) {
      _startAutoSync();
    }
  }

  /// Get auto-sync status
  bool get autoSyncEnabled => _autoSyncEnabled;

  /// Get sync interval
  int get syncIntervalSeconds => _syncIntervalSeconds;

  /// Get queue statistics
  Future<QueueStats> getQueueStats() async {
    final pending = await _syncDatasource.getPendingCount();
    final failed = await _syncDatasource.getFailedCount();
    final totalFailed = await _syncDatasource.getTotalFailedCount();
    return QueueStats(
      pendingCount: pending,
      failedCount: failed,
      totalFailedAttempts: totalFailed,
    );
  }

  /// Dispose of the service
  void dispose() {
    _stopAutoSync();
    _stopQueueWorker();
    _syncEventsController.close();
  }
}

/// Queue statistics
class QueueStats {
  final int pendingCount;
  final int failedCount;
  final int totalFailedAttempts;

  const QueueStats({
    required this.pendingCount,
    required this.failedCount,
    required this.totalFailedAttempts,
  });
}

/// Provider for SyncService
final syncServiceProvider = Provider<SyncService>((ref) {
  final crdtService = ref.watch(crdtServiceProvider);
  final syncDatasource = ref.watch(pendingSyncDatasourceProvider);
  return SyncService(crdtService, syncDatasource);
});
