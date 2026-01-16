import 'dart:async';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:connectivity_plus/connectivity_plus.dart';
import 'package:miniwiki/core/network/api_client.dart';
import 'package:miniwiki/data/datasources/pending_sync_datasource.dart';

/// Simple logger for sync service
class SyncLogger {
  final String _tag;

  SyncLogger(this._tag);

  void warn(String message) {
    print('[WARN][$_tag] $message');
  }

  void info(String message) {
    print('[INFO][$_tag] $message');
  }

  void error(String message) {
    print('[ERROR][$_tag] $message');
  }
}

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
  final List<Map<String, dynamic>> skippedEntities;
  final DateTime timestamp;

  const SyncSummary({
    required this.success,
    required this.syncedCount,
    required this.failedCount,
    this.skippedEntities = const [],
    required this.timestamp,
  });

  SyncSummary copyWith({
    bool? success,
    int? syncedCount,
    int? failedCount,
    List<Map<String, dynamic>>? skippedEntities,
    DateTime? timestamp,
  }) =>
      SyncSummary(
        success: success ?? this.success,
        syncedCount: syncedCount ?? this.syncedCount,
        failedCount: failedCount ?? this.failedCount,
        skippedEntities: skippedEntities ?? this.skippedEntities,
        timestamp: timestamp ?? this.timestamp,
      );
}

/// Sync service for handling offline-first synchronization
class SyncService {
  final PendingSyncDatasource _syncDatasource;
  final ApiClient _apiClient;
  final SyncLogger logger = SyncLogger('SyncService');

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

  /// Whether queue processing is in progress (prevents concurrent execution)
  bool _isProcessing = false;

  /// Failed sync count (current session)
  int _failedCount = 0;

  /// Total failed count (historical)
  int _totalFailedCount = 0;

  final Connectivity _connectivity;

  /// Sync service constructor
  SyncService(this._syncDatasource, this._apiClient,
      {Connectivity? connectivity})
      : _connectivity = connectivity ?? Connectivity();

  /// Initialize sync service
  Future<void> initialize() async {
    _connectivitySubscription =
        _connectivity.onConnectivityChanged.listen(_onConnectivityChanged);
    _connectivityStatus = await _connectivity.checkConnectivity();
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
    if (_isProcessing) return;
    _isProcessing = true;

    try {
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
            await _syncDatasource.removeFromQueue(entityType, entityId);
            successCount++;
          } else {
            throw Exception('Sync returned false');
          }
        } catch (e) {
          print('[SyncService] Failed to sync $entityType:$entityId - $e');
          await _syncDatasource.removeFromQueue(entityType, entityId);
          await _syncDatasource.addToFailedQueue(
              entityType, entityId, operation, data ?? {}, e.toString());
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
    } finally {
      _isProcessing = false;
    }
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
  int getPendingSyncCount() {
    return _syncDatasource.getQueueSize();
  }

  /// Get pending sync count (deprecated)
  @Deprecated('Use getPendingSyncCount instead')
  int getPendingCount() {
    return getPendingSyncCount();
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

  /// Set sync interval in seconds
  void setSyncInterval(int seconds) {
    _syncIntervalSeconds = seconds;
    if (_autoSyncEnabled && _isOnline) {
      _startAutoSync();
    }
  }

  /// Sync all dirty documents
  Future<SyncSummary> syncAllDirtyDocuments() async {
    if (!_isOnline) {
      return SyncSummary(
        success: false,
        syncedCount: 0,
        failedCount: 0,
        timestamp: DateTime.now(),
      );
    }

    final pendingItems = _syncDatasource.getPendingItems();
    if (pendingItems.isEmpty) {
      return SyncSummary(
        success: true,
        syncedCount: 0,
        failedCount: 0,
        timestamp: DateTime.now(),
      );
    }

    int syncedCount = 0;
    int failedCount = 0;
    final List<Map<String, dynamic>> skippedEntities = [];

    for (final item in pendingItems) {
      final entityType = item['entityType'] as String?;
      final entityId = item['entityId'] as String?;
      final operation = (item['operation'] ?? item['op']) as String?;
      final data = item['data'] as Map<String, dynamic>? ?? {};

      if (entityType == null || entityId == null || operation == null || operation.isEmpty) {
        logger.warn('Invalid queue item metadata: $item');
        if (entityType != null && entityId != null) {
          await _syncDatasource.removeFromQueue(entityType, entityId);
          await _syncDatasource.addToFailedQueue(
              entityType,
              entityId,
              operation ?? 'unknown',
              data,
              'Missing or invalid operation metadata');
          failedCount++;
          _failedCount++;
          _totalFailedCount++;
        }
        continue;
      }

      try {
        // Only process document entities
        if (entityType == 'document') {
          final success = await _syncDocument(entityId!, operation!, data);
          if (success) {
            await _syncDatasource.removeFromQueue(entityType, entityId!);
            syncedCount++;
          } else {
            await _syncDatasource.removeFromQueue(entityType, entityId!);
            await _syncDatasource.addToFailedQueue(
                entityType, entityId!, operation!, data, 'sync returned false');
            failedCount++;
            _failedCount++;
            _totalFailedCount++;
          }
        } else {
          // Skip non-document entities - log warning, track in summary, move to skipped queue
          logger.warn('Skipping non-document entity: $entityType:$entityId');
          skippedEntities.add({
            'entityType': entityType,
            'entityId': entityId,
          });
          final moved =
              await _syncDatasource.moveToSkippedQueue(entityType, entityId!);
          if (!moved) {
            logger.warn(
                'Failed to move skipped item: $entityType:$entityId (not found in pending)');
          }
        }
      } catch (e) {
        await _syncDatasource.removeFromQueue(entityType, entityId);
        await _syncDatasource.addToFailedQueue(
            entityType, entityId!, operation!, data, e.toString());
        failedCount++;
        _failedCount++;
        _totalFailedCount++;
      }
    }

    return SyncSummary(
      success: failedCount == 0,
      syncedCount: syncedCount,
      failedCount: failedCount,
      skippedEntities: skippedEntities,
      timestamp: DateTime.now(),
    );
  }

  /// Sync a single document and return result
  Future<SyncResult> syncDocument(String documentId) async {
    if (!_isOnline) {
      return SyncResult(
        success: false,
        errorMessage: 'Not online',
        documentsSynced: 0,
      );
    }

    try {
      // Get the document from cache or queue
      final cachedDoc = _syncDatasource.getCachedDocument(documentId);
      if (cachedDoc != null) {
        try {
          // Read operation metadata - support both 'op' and 'operation' for migration
          final operation =
              (cachedDoc['operation'] ?? cachedDoc['op']) as String?;

          if (operation == null || operation.isEmpty) {
            throw Exception('Missing operation metadata in cached document');
          }

          // Create sanitized copy without internal metadata
          final sanitizedData = Map<String, dynamic>.from(cachedDoc)
            ..remove('op')
            ..remove('operation');

          // Branch to correct API method based on operation
          switch (operation) {
            case 'create':
              final spaceId = sanitizedData['spaceId'] as String?;
              if (spaceId == null) {
                throw Exception('Missing spaceId for create operation');
              }
              await _apiClient.post('/spaces/$spaceId/documents',
                  data: sanitizedData);
              break;

            case 'update':
              await _apiClient.patch('/documents/$documentId',
                  data: sanitizedData);
              break;

            case 'delete':
              await _apiClient.delete('/documents/$documentId');
              break;

            default:
              throw Exception('Unsupported operation: $operation');
          }

          // Remove from cache after successful sync
          await _syncDatasource.removeCachedDocument(documentId);

          // Also remove potentially pending item for this document to prevent double-sync
          await _syncDatasource.removeFromQueue('document', documentId);

          return SyncResult(
            success: true,
            documentsSynced: 1,
          );
        } catch (e) {
          // Handle cached-document branch failures
          // Extract operation for failed queue
          final operation =
              (cachedDoc['operation'] ?? cachedDoc['op']) as String? ??
                  'unknown';

          // Add to failed queue
          await _syncDatasource.addToFailedQueue(
            'document',
            documentId,
            operation,
            cachedDoc,
            e.toString(),
          );

          // Remove cached/queue entries to prevent repeated failures
          await _syncDatasource.removeCachedDocument(documentId);
          await _syncDatasource.removeFromQueue('document', documentId);

          // Return failure result
          return SyncResult(
            success: false,
            errorMessage: e.toString(),
            documentsSynced: 0,
          );
        }
      }

      // Check if document is in queue
      final pendingItems = _syncDatasource.getPendingItems();
      final queuedItem = pendingItems.firstWhere(
        (item) =>
            item['entityType'] == 'document' && item['entityId'] == documentId,
        orElse: () => <String, dynamic>{},
      );

      if (queuedItem.isNotEmpty) {
        final data = queuedItem['data'] as Map<String, dynamic>? ?? {};
        final operation = queuedItem['operation'] as String?;
        if (operation == null || operation.isEmpty) {
          await _syncDatasource.removeFromQueue('document', documentId);
          await _syncDatasource.addToFailedQueue('document', documentId,
              'unknown', data, 'Missing operation metadata');
          return SyncResult(
            success: false,
            errorMessage: 'Missing operation metadata',
            documentsSynced: 0,
          );
        }

        try {
          final success = await _syncDocument(documentId, operation, data);
          if (success) {
            await _syncDatasource.removeFromQueue('document', documentId);
          } else {
            // Move to failed queue on failure
            await _syncDatasource.removeFromQueue('document', documentId);
            await _syncDatasource.addToFailedQueue(
                'document', documentId, operation, data, 'sync returned false');
          }
          return SyncResult(
            success: success,
            errorMessage: success ? null : 'Sync failed',
            documentsSynced: success ? 1 : 0,
          );
        } catch (e) {
          // Move to failed queue on exception
          await _syncDatasource.removeFromQueue('document', documentId);
          await _syncDatasource.addToFailedQueue(
              'document', documentId, operation, data, e.toString());
          // Return failure result instead of rethrowing to avoid duplicating error in outer catch
          return SyncResult(
            success: false,
            errorMessage: e.toString(),
            documentsSynced: 0,
          );
        }
      }

      return SyncResult(
        success: false,
        errorMessage: 'Document not found in cache or queue',
        documentsSynced: 0,
      );
    } catch (e) {
      return SyncResult(
        success: false,
        errorMessage: e.toString(),
        documentsSynced: 0,
      );
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
  final syncDatasource = ref.watch(pendingSyncDatasourceProvider);
  final syncService = SyncService(syncDatasource, ApiClient.defaultInstance());
  ref.onDispose(syncService.dispose);
  return syncService;
});
