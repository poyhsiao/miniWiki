import 'dart:async';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:connectivity_plus/connectivity_plus.dart';
import 'package:miniwiki/services/crdt_service.dart';

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

  /// Sync service constructor
  SyncService(this._crdtService);

  /// Initialize sync service
  Future<void> initialize() async {
    // Subscribe to connectivity changes
    Connectivity().onConnectivityChanged.listen(_onConnectivityChanged);

    // Get initial connectivity status
    _connectivityStatus = await Connectivity().checkConnectivity();

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
      } else {
        // Went offline, stop auto-sync
        _stopAutoSync();
      }
    }
  }

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

  /// Sync all dirty documents
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

  /// Queue a document for sync
  Future<void> queueForSync(String documentId) async {
    final update = await _crdtService.getState(documentId);
    if (update != null) {
      // TODO: Implement sync queue with Isar
      // For now, just mark as synced locally
      _crdtService.markSynced(documentId);
    }
  }

  /// Get pending sync queue count
  Future<int> getPendingSyncCount() async {
    // TODO: Implement with Isar sync queue
    return _crdtService.getDirtyDocumentIds().length;
  }

  /// Clear sync queue
  Future<void> clearSyncQueue() async {
    // TODO: Implement with Isar sync queue
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

  /// Dispose of the service
  void dispose() {
    _stopAutoSync();
    _syncEventsController.close();
  }
}

/// Provider for SyncService
final syncServiceProvider = Provider<SyncService>((ref) {
  final crdtService = ref.watch(crdtServiceProvider);
  return SyncService(crdtService);
});
