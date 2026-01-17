// Pending sync datasource for managing offline sync queue
// Uses SharedPreferences for Web-compatible local storage

import 'dart:async';
import 'package:flutter/foundation.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:miniwiki/data/datasources/local_storage.dart';

/// Datasource for managing pending sync operations and offline document caching
class PendingSyncDatasource {
  final DocumentCacheService _documentCache;
  final SyncQueueService _syncQueue;

  PendingSyncDatasource({required LocalStorageService storage})
      : _documentCache = DocumentCacheService(storage),
        _syncQueue = SyncQueueService(storage);

  // ============================================
  // SYNC QUEUE OPERATIONS
  // ============================================

  /// Add item to queue
  Future<void> addToQueue(String entityType, String entityId, String operation,
      Map<String, dynamic> data) async {
    await _syncQueue.addItem(entityType, entityId, operation, data);
  }

  /// Get all pending items
  List<Map<String, dynamic>> getPendingItems() => _syncQueue.getQueueItems();

  /// Remove item from queue
  Future<void> removeFromQueue(String entityType, String entityId) async {
    await _syncQueue.removeItem(entityType, entityId);
  }

  /// Clear queue
  Future<void> clearQueue() async {
    await _syncQueue.clearQueue();
  }

  /// Get queue size
  int getQueueSize() => _syncQueue.getQueueSize();

  // ============================================
  // FAILED QUEUE OPERATIONS
  // ============================================

  /// Add item to failed queue
  Future<void> addToFailedQueue(String entityType, String entityId,
      String operation, Map<String, dynamic> data, String error) async {
    await _syncQueue.addFailedItem(
        entityType, entityId, operation, data, error);
  }

  /// Get all failed items
  List<Map<String, dynamic>> getFailedItems() => _syncQueue.getFailedItems();

  /// Remove item from failed queue
  Future<void> removeFromFailedQueue(String entityType, String entityId) async {
    await _syncQueue.removeFailedItem(entityType, entityId);
  }

  /// Clear failed queue
  Future<void> clearFailedQueue() async {
    await _syncQueue.clearFailedQueue();
  }

  /// Get failed queue size
  int getFailedQueueSize() => _syncQueue.getFailedQueueSize();

  // ============================================
  // DOCUMENT CACHE OPERATIONS
  // ============================================

  /// Cache a document
  Future<void> cacheDocument(String docId, Map<String, dynamic> data) async {
    await _documentCache.cacheDocument(docId, data);
  }

  /// Get cached document
  Map<String, dynamic>? getCachedDocument(String docId) => _documentCache.getDocument(docId);

  /// Remove cached document
  Future<void> removeCachedDocument(String docId) async {
    await _documentCache.removeDocument(docId);
  }

  /// Get all cached document IDs
  List<String> getCachedDocIds() => _documentCache.getCachedDocIds();

  /// Clear document cache
  Future<void> clearDocumentCache() async {
    await _documentCache.clearCache();
  }

  // ============================================
  // OFFLINE CONTENT OPERATIONS
  // ============================================

  /// Cache document content for offline access
  Future<void> cacheContent(String docId, String content) async {
    await _documentCache.cacheContent(docId, content);
  }

  /// Get cached content
  String? getCachedContent(String docId) => _documentCache.getCachedContent(docId);

  // ============================================
  // SKIPPED QUEUE OPERATIONS
  // ============================================

  /// Move item from pending queue to skipped queue
  ///
  /// Note: If removal from pending queue fails and rollback also fails,
  /// the item may exist in both queues. A reconciliation mechanism should
  /// be implemented to detect and resolve such inconsistencies if needed.
  Future<bool> moveToSkippedQueue(String entityType, String entityId) async {
    // Check if pending item exists before attempting to move
    final pendingItems = _syncQueue.getQueueItems();
    final hasPending = pendingItems.any(
      (item) =>
          item['entityType'] == entityType && item['entityId'] == entityId,
    );

    if (!hasPending) {
      return false; // Nothing to move
    }

    // Try to add to skipped queue first
    var skippedAdded = false;
    try {
      await _syncQueue.addSkippedItem(entityType, entityId);
      skippedAdded = true;
    } catch (e) {
      // Failed to add to skipped queue, propagate error
      rethrow;
    }

    // Try to remove from pending queue
    try {
      await _syncQueue.removeItem(entityType, entityId);
      return true;
    } catch (e) {
      // Rollback: remove from skipped queue only if it was added
      if (skippedAdded) {
        try {
          // Check if skipped item exists before removing (idempotent)
          final skippedItems = _syncQueue.getSkippedItems();
          final hasSkipped = skippedItems.any(
            (item) =>
                item['entityType'] == entityType &&
                item['entityId'] == entityId,
          );
          if (hasSkipped) {
            await _syncQueue.removeSkippedItem(entityType, entityId);
          }
        } catch (rollbackError) {
          // Log rollback failure so it is visible
          // TODO: Replace with proper logging when logging infrastructure is available
          debugPrint(
              'Rollback failed for $entityType:$entityId: $rollbackError');
        }
      }
      rethrow;
    }
  }

  /// Get all skipped items
  List<Map<String, dynamic>> getSkippedItems() => _syncQueue.getSkippedItems();

  /// Get skipped queue size
  int getSkippedQueueSize() => _syncQueue.getSkippedQueueSize();

  /// Clear skipped queue
  Future<void> clearSkippedQueue() async {
    await _syncQueue.clearSkippedQueue();
  }
}

/// Provider for PendingSyncDatasource
final pendingSyncDatasourceProvider = Provider<PendingSyncDatasource>((ref) {
  final storage = ref.watch(localStorageServiceProvider);
  return PendingSyncDatasource(storage: storage);
});
