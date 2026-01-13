// Pending sync datasource for managing offline sync queue
// Handles storage and retrieval of sync queue items

import 'dart:async';
import 'dart:typed_data';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:isar/isar.dart';
import 'package:path_provider/path_provider.dart';
import 'package:miniwiki/data/models/sync_queue_item.dart';

/// Datasource for managing pending sync operations
class PendingSyncDatasource {
  final Isar _isar;

  PendingSyncDatasource({required Isar isar}) : _isar = isar;

  /// Get the sync queue collection
  IsarCollection<SyncQueueItem> get _collection => _isar.syncQueueItems;

  /// Add item to queue
  Future<void> addToQueue(SyncQueueItem item) async {
    await _isar.writeTxn(() async {
      await _collection.put(item);
    });
  }

  /// Get next pending item
  Future<SyncQueueItem?> getNextPendingItem() async {
    return await _isar.txn(() async {
      final items = await _collection
          .where()
          .filter()
          .retryCountLessThan(3)
          .sortByCreatedAt()
          .findAll();

      final pending = items.firstWhere(
        (item) => item.retryCount < 3,
        orElse: () => items.first,
      );

      return pending;
    });
  }

  /// Mark item as synced
  Future<void> markAsSynced(String itemId) async {
    await _isar.writeTxn(() async {
      final item = await _collection.get(int.tryParse(itemId) ?? 0);
      if (item != null) {
        item.retryCount = 0;
        await _collection.delete(item.id);
      }
    });
  }

  /// Mark item as failed
  Future<void> markAsFailed(String itemId, String error) async {
    await _isar.writeTxn(() async {
      final item = await _collection.get(int.tryParse(itemId) ?? 0);
      if (item != null) {
        item.retryCount++;
        item.nextRetryAt = DateTime.now().add(Duration(seconds: 30 * item.retryCount));
        await _collection.put(item);
      }
    });
  }

  /// Remove item from queue
  Future<void> removeFromQueue(String itemId) async {
    await _isar.writeTxn(() async {
      await _collection.delete(int.tryParse(itemId) ?? 0);
    });
  }

  /// Clear queue
  Future<void> clearQueue({bool onlyCompleted = false}) async {
    await _isar.writeTxn(() async {
      if (onlyCompleted) {
        final items = await _collection.where().findAll();
        for (final item in items) {
          if (item.retryCount == 0) {
            await _collection.delete(item.id);
          }
        }
      } else {
        await _collection.clear();
      }
    });
  }

  /// Get total queue count
  Future<int> getQueueCount() async {
    return await _collection.count();
  }

  /// Get pending count
  Future<int> getPendingCount() async {
    return await _isar.txn(() async {
      final items = await _collection.where().findAll();
      return items.where((item) => item.retryCount < 3).length;
    });
  }

  /// Get failed count
  Future<int> getFailedCount() async {
    return await _isar.txn(() async {
      final items = await _collection.where().findAll();
      return items.where((item) => item.retryCount >= 3).length;
    });
  }

  /// Cache document
  Future<void> cacheDocument(String documentId, Uint8List content) async {
    // Document caching would be handled separately
    // This is a placeholder for the interface
  }

  /// Get cached document
  Future<Uint8List?> getCachedDocument(String documentId) async {
    // Document caching would be handled separately
    return null;
  }

  /// Remove cached document
  Future<void> removeCachedDocument(String documentId) async {
    // Document caching would be handled separately
  }

  /// Get all cached document IDs
  Future<List<String>> getAllCachedDocumentIds() async {
    // Document caching would be handled separately
    return [];
  }

  /// Get cache size
  Future<int> getCacheSize() async {
    // Document caching would be handled separately
    return 0;
  }

  /// Clear cache
  Future<void> clearCache() async {
    // Document caching would be handled separately
  }
}

/// Provider for PendingSyncDatasource
final pendingSyncDatasourceProvider = Provider<PendingSyncDatasource>((ref) {
  return PendingSyncDatasource(isar: isar);
});

/// Isar provider
final isar = FutureProvider<Isar>((ref) async {
  final dir = await getApplicationDocumentsDirectory();
  return await Isar.open(
    [SyncQueueItemSchema],
    directory: dir.path,
  );
});
