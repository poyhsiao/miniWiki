// Pending sync datasource for managing offline sync queue
// Handles storage and retrieval of sync queue items and cached documents

import 'dart:async';
import 'dart:typed_data';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:isar/isar.dart';
import 'package:path_provider/path_provider.dart';
import 'package:miniwiki/data/models/sync_queue_item.dart';
import 'package:miniwiki/data/models/cached_document.dart';

/// Helper function for exponential calculation
int _pow(int base, int exponent) {
  var result = 1;
  for (var i = 0; i < exponent; i++) {
    result *= base;
  }
  return result;
}

/// Datasource for managing pending sync operations and offline document caching
class PendingSyncDatasource {
  final Isar _isar;

  PendingSyncDatasource({required Isar isar}) : _isar = isar;

  /// Get the sync queue collection
  IsarCollection<SyncQueueItem> get _queueCollection => _isar.syncQueueItems;

  /// Get the cached documents collection
  IsarCollection<CachedDocument> get _cachedDocsCollection => _isar.cachedDocuments;

  // ============================================
  // SYNC QUEUE OPERATIONS
  // ============================================

  /// Add item to queue
  Future<void> addToQueue(SyncQueueItem item) async {
    await _isar.writeTxn(() async {
      await _queueCollection.put(item);
    });
  }

  /// Get next pending item (respecting retry limits and next retry time)
  Future<SyncQueueItem?> getNextPendingItem() async => await _isar.txn(() async {
      final now = DateTime.now();
      final items = await _queueCollection
          .where()
          .filter()
          .retryCountLessThan(3)
          .nextRetryAtLessThan(now)
          .sortByCreatedAt()
          .findAll();

      if (items.isEmpty) return null;

      // Return the earliest item that's ready for retry
      return items.firstWhere(
        (item) => item.retryCount < 3,
        orElse: () => items.first,
      );
    });

  /// Get all pending items
  Future<List<SyncQueueItem>> getAllPendingItems() async => await _isar.txn(() async {
      final now = DateTime.now();
      return await _queueCollection
          .where()
          .filter()
          .retryCountLessThan(3)
          .nextRetryAtLessThan(now)
          .findAll();
    });

  /// Get items ready for retry
  Future<List<SyncQueueItem>> getItemsReadyForRetry() async {
    final now = DateTime.now();
    return _isar.txn(() async => await _queueCollection
          .where()
          .filter()
          .retryCountLessThan(3)
          .nextRetryAtLessThan(now)
          .findAll());
  }

  /// Mark item as synced (remove from queue)
  Future<void> markAsSynced(String itemId) async {
    await _isar.writeTxn(() async {
      final item = await _queueCollection.get(int.tryParse(itemId) ?? 0);
      if (item != null) {
        await _queueCollection.delete(item.id);
      }
    });
  }

  /// Mark item as failed with exponential backoff
  Future<void> markAsFailed(String itemId, String error) async {
    await _isar.writeTxn(() async {
      final item = await _queueCollection.get(int.tryParse(itemId) ?? 0);
      if (item != null) {
        item.retryCount++;
        // Exponential backoff: 30s, 2min, 8min (for 3 retries)
        item.nextRetryAt = DateTime.now().add(Duration(seconds: 30 * _pow(4, item.retryCount - 1)));
        await _queueCollection.put(item);
      }
    });
  }

  /// Remove item from queue
  Future<void> removeFromQueue(String itemId) async {
    await _isar.writeTxn(() async {
      await _queueCollection.delete(int.tryParse(itemId) ?? 0);
    });
  }

  /// Clear queue
  Future<void> clearQueue({bool onlyCompleted = false}) async {
    await _isar.writeTxn(() async {
      if (onlyCompleted) {
        final items = await _queueCollection.where().findAll();
        for (final item in items) {
          if (item.retryCount == 0) {
            await _queueCollection.delete(item.id);
          }
        }
      } else {
        await _queueCollection.clear();
      }
    });
  }

  /// Get total queue count
  Future<int> getQueueCount() async => await _queueCollection.count();

  /// Get pending count (items ready for retry)
  Future<int> getPendingCount() async {
    final now = DateTime.now();
    return _isar.txn(() async => await _queueCollection
          .where()
          .filter()
          .retryCountLessThan(3)
          .nextRetryAtLessThan(now)
          .count());
  }

  /// Get failed count (items that exceeded retry limit)
  Future<int> getFailedCount() async => await _isar.txn(() async {
      final items = await _queueCollection.where().findAll();
      return items.where((item) => item.retryCount >= 3).length;
    });

  /// Get total failed attempts count
  Future<int> getTotalFailedCount() async => await _isar.txn(() async {
      final items = await _queueCollection.where().findAll();
      return items.fold<int>(0, (sum, item) => sum + item.retryCount);
    });

  // ============================================
  // DOCUMENT CACHING OPERATIONS
  // ============================================

  /// Cache a document for offline access
  Future<void> cacheDocument(String documentId, Uint8List content) async {
    await _isar.writeTxn(() async {
      final existing = await _cachedDocsCollection
          .where()
          .documentIdEqualTo(documentId)
          .findFirst();

      if (existing != null) {
        existing.content = content;
        existing.cachedAt = DateTime.now();
        existing.isDirty = true;
        await _cachedDocsCollection.put(existing);
      } else {
        final cachedDoc = CachedDocument(documentId: documentId);
        cachedDoc.content = content;
        cachedDoc.cachedAt = DateTime.now();
        cachedDoc.expiresAt = DateTime.now().add(const Duration(days: 7));
        await _cachedDocsCollection.put(cachedDoc);
      }
    });
  }

  /// Cache a document with full metadata
  Future<void> cacheDocumentWithMeta({
    required String documentId,
    required String title,
    required String spaceId,
    required Uint8List content, String? parentId,
    String? stateVector,
    int version = 0,
    DateTime? expiresAt,
    int priority = 0,
  }) async {
    await _isar.writeTxn(() async {
      final existing = await _cachedDocsCollection
          .where()
          .documentIdEqualTo(documentId)
          .findFirst();

      if (existing != null) {
        existing
          ..title = title
          ..spaceId = spaceId
          ..parentId = parentId
          ..content = content
          ..stateVector = stateVector
          ..version = version
          ..modifiedAt = DateTime.now()
          ..cachedAt = DateTime.now()
          ..priority = priority
          ..isDirty = true;
        if (expiresAt != null) {
          existing.expiresAt = expiresAt;
        }
        await _cachedDocsCollection.put(existing);
      } else {
        final cachedDoc = CachedDocument(
          documentId: documentId,
          title: title,
          spaceId: spaceId,
          parentId: parentId,
        );
        cachedDoc.content = content;
        cachedDoc.stateVector = stateVector;
        cachedDoc.version = version;
        cachedDoc.modifiedAt = DateTime.now();
        cachedDoc.cachedAt = DateTime.now();
        cachedDoc.expiresAt = expiresAt ?? DateTime.now().add(const Duration(days: 7));
        cachedDoc.priority = priority;
        cachedDoc.isDirty = true;
        await _cachedDocsCollection.put(cachedDoc);
      }
    });
  }

  /// Get cached document content
  Future<Uint8List?> getCachedDocument(String documentId) async {
    final doc = await _cachedDocsCollection
        .where()
        .documentIdEqualTo(documentId)
        .findFirst();

    if (doc == null || doc.content == null) return null;

    // Check if expired
    if (doc.isExpired) {
      await removeCachedDocument(documentId);
      return null;
    }

    return Uint8List.fromList(doc.content!);
  }

  /// Get full cached document with metadata
  Future<CachedDocument?> getCachedDocumentWithMeta(String documentId) async {
    final doc = await _cachedDocsCollection
        .where()
        .documentIdEqualTo(documentId)
        .findFirst();

    if (doc == null) return null;

    // Check if expired
    if (doc.isExpired) {
      await removeCachedDocument(documentId);
      return null;
    }

    return doc;
  }

  /// Remove cached document
  Future<void> removeCachedDocument(String documentId) async {
    await _isar.writeTxn(() async {
      final doc = await _cachedDocsCollection
          .where()
          .documentIdEqualTo(documentId)
          .findFirst();
      if (doc != null) {
        await _cachedDocsCollection.delete(doc.id);
      }
    });
  }

  /// Get all cached document IDs
  Future<List<String>> getAllCachedDocumentIds() async {
    final docs = await _cachedDocsCollection.where().findAll();
    return docs.map((doc) => doc.documentId).toList();
  }

  /// Get all valid (non-expired) cached documents
  Future<List<CachedDocument>> getAllValidCachedDocuments() async {
    final docs = await _cachedDocsCollection.where().findAll();
    return docs.where((doc) => !doc.isExpired).toList();
  }

  /// Get cached documents for a space
  Future<List<CachedDocument>> getCachedDocumentsForSpace(String spaceId) async {
    final docs = await _cachedDocsCollection
        .where()
        .spaceIdEqualTo(spaceId)
        .findAll();
    return docs.where((doc) => !doc.isExpired).toList();
  }

  /// Get dirty (unsynced) cached documents
  Future<List<CachedDocument>> getDirtyDocuments() async {
    final docs = await _cachedDocsCollection.where().findAll();
    return docs.where((doc) => doc.isDirty && !doc.isExpired).toList();
  }

  /// Mark document as synced
  Future<void> markDocumentSynced(String documentId, String? stateVector) async {
    await _isar.writeTxn(() async {
      final doc = await _cachedDocsCollection
          .where()
          .documentIdEqualTo(documentId)
          .findFirst();
      if (doc != null) {
        doc.isDirty = false;
        doc.stateVector = stateVector;
        doc.modifiedAt = DateTime.now();
        await _cachedDocsCollection.put(doc);
      }
    });
  }

  /// Update document content and mark dirty
  Future<void> updateDocumentContent(String documentId, Uint8List content) async {
    await _isar.writeTxn(() async {
      final doc = await _cachedDocsCollection
          .where()
          .documentIdEqualTo(documentId)
          .findFirst();
      if (doc != null) {
        doc.content = content;
        doc.isDirty = true;
        doc.modifiedAt = DateTime.now();
        doc.cachedAt = DateTime.now();
        await _cachedDocsCollection.put(doc);
      }
    });
  }

  /// Get cache size in bytes
  Future<int> getCacheSize() async {
    final docs = await _cachedDocsCollection.where().findAll();
    var totalSize = 0;
    for (final doc in docs) {
      totalSize += doc.content?.length ?? 0;
    }
    return totalSize;
  }

  /// Get cache statistics
  Future<CacheStats> getCacheStats() async {
    final docs = await _cachedDocsCollection.where().findAll();

    var validCount = 0;
    var expiredCount = 0;
    var dirtyCount = 0;
    var totalBytes = 0;

    for (final doc in docs) {
      if (doc.isExpired) {
        expiredCount++;
      } else {
        validCount++;
        totalBytes += doc.content?.length ?? 0;
        if (doc.isDirty) dirtyCount++;
      }
    }

    return CacheStats(
      totalDocuments: docs.length,
      validDocuments: validCount,
      expiredDocuments: expiredCount,
      dirtyDocuments: dirtyCount,
      totalBytes: totalBytes,
    );
  }

  /// Clear all cached documents
  Future<void> clearCache() async {
    await _isar.writeTxn(() async {
      await _cachedDocsCollection.clear();
    });
  }

  /// Clear expired cache entries
  Future<void> clearExpiredCache() async {
    await _isar.writeTxn(() async {
      final docs = await _cachedDocsCollection.where().findAll();
      for (final doc in docs) {
        if (doc.isExpired) {
          await _cachedDocsCollection.delete(doc.id);
        }
      }
    });
  }

  /// Clear cache for a specific space
  Future<void> clearCacheForSpace(String spaceId) async {
    await _isar.writeTxn(() async {
      final docs = await _cachedDocsCollection
          .where()
          .spaceIdEqualTo(spaceId)
          .findAll();
      for (final doc in docs) {
        await _cachedDocsCollection.delete(doc.id);
      }
    });
  }

  /// Set cache TTL for a document
  Future<void> setDocumentCacheTtl(String documentId, Duration ttl) async {
    await _isar.writeTxn(() async {
      final doc = await _cachedDocsCollection
          .where()
          .documentIdEqualTo(documentId)
          .findFirst();
      if (doc != null) {
        doc.expiresAt = DateTime.now().add(ttl);
        await _cachedDocsCollection.put(doc);
      }
    });
  }

  /// Extend all cache TTLs
  Future<void> extendAllCacheTtl(Duration ttl) async {
    await _isar.writeTxn(() async {
      final docs = await _cachedDocsCollection.where().findAll();
      final newExpiry = DateTime.now().add(ttl);
      for (final doc in docs) {
        doc.expiresAt = newExpiry;
        await _cachedDocsCollection.put(doc);
      }
    });
  }

  /// Prune cache to max size (removes oldest/lowest priority items)
  Future<void> pruneCache(int maxBytes) async {
    await _isar.writeTxn(() async {
      final docs = await _cachedDocsCollection
          .where()
          .sortByPriority()
          .thenByCachedAt()
          .findAll();

      var currentSize = 0;
      for (final doc in docs) {
        final docSize = doc.content?.length ?? 0;
        if (currentSize + docSize > maxBytes) {
          await _cachedDocsCollection.delete(doc.id);
        } else {
          currentSize += docSize;
        }
      }
    });
  }
}

/// Cache statistics
class CacheStats {
  final int totalDocuments;
  final int validDocuments;
  final int expiredDocuments;
  final int dirtyDocuments;
  final int totalBytes;

  const CacheStats({
    required this.totalDocuments,
    required this.validDocuments,
    required this.expiredDocuments,
    required this.dirtyDocuments,
    required this.totalBytes,
  });

  String get totalBytesFormatted {
    if (totalBytes < 1024) return '$totalBytes B';
    if (totalBytes < 1024 * 1024) return '${(totalBytes / 1024).toStringAsFixed(1)} KB';
    return '${(totalBytes / (1024 * 1024)).toStringAsFixed(1)} MB';
  }

  int get validBytes => totalBytes;
}

/// Isar provider - needs to be initialized with proper schema
final isarProvider = FutureProvider<Isar>((ref) async {
  final dir = await getApplicationDocumentsDirectory();
  return Isar.open(
    [SyncQueueItemSchema, CachedDocumentSchema],
    directory: dir.path,
    name: 'miniwiki_sync',
  );
});

/// Provider for PendingSyncDatasource
final pendingSyncDatasourceProvider = Provider<PendingSyncDatasource>((ref) {
  final isar = ref.watch(isarProvider).value;
  if (isar == null) {
    throw StateError('Isar not initialized');
  }
  return PendingSyncDatasource(isar: isar);
});
