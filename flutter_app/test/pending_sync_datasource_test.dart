import 'package:flutter_test/flutter_test.dart';
import 'package:miniwiki/data/datasources/local_storage.dart';
import 'package:miniwiki/data/datasources/pending_sync_datasource.dart';
import 'package:shared_preferences/shared_preferences.dart';

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();
  // Setup mock preferences
  setUpAll(() async {
    SharedPreferences.setMockInitialValues({});
  });

  group('PendingSyncDatasource Tests', () {
    late LocalStorageService storage;
    late PendingSyncDatasource datasource;

    setUp(() async {
      storage = await LocalStorageService.getInstance();
      await storage.clear(); // Clear before each test
      datasource = PendingSyncDatasource(storage: storage);
    });

    group('Sync Queue Operations', () {
      test('addToQueue and getPendingItems work', () async {
        // Arrange
        final data = {'title': 'Test'};

        // Act
        await datasource.addToQueue('document', 'doc1', 'create', data);
        final items = datasource.getPendingItems();

        // Assert
        expect(items.length, 1);
        expect(items[0]['entityType'], 'document');
        expect(items[0]['entityId'], 'doc1');
      });

      test('getQueueSize returns correct count', () async {
        // Arrange
        await datasource.addToQueue('document', 'doc1', 'create', {});
        await datasource.addToQueue('document', 'doc2', 'update', {});

        // Act
        final size = datasource.getQueueSize();

        // Assert
        expect(size, 2);
      });

      test('removeFromQueue deletes item', () async {
        // Arrange
        await datasource.addToQueue('document', 'doc1', 'create', {});

        // Act
        await datasource.removeFromQueue('document', 'doc1');
        final items = datasource.getPendingItems();

        // Assert
        expect(items, isEmpty);
      });

      test('clearQueue removes all items', () async {
        // Arrange
        await datasource.addToQueue('document', 'doc1', 'create', {});
        await datasource.addToQueue('document', 'doc2', 'update', {});

        // Act
        await datasource.clearQueue();
        final size = datasource.getQueueSize();

        // Assert
        expect(size, 0);
      });
    });

    group('Failed Queue Operations', () {
      test('addToFailedQueue and getFailedItems work', () async {
        // Arrange
        final data = {'title': 'Test'};
        const error = 'Network error';

        // Act
        await datasource.addToFailedQueue('document', 'doc1', 'create', data, error);
        final items = datasource.getFailedItems();

        // Assert
        expect(items.length, 1);
        expect(items[0]['entityType'], 'document');
        expect(items[0]['error'], error);
      });

      test('getFailedQueueSize returns correct count', () async {
        // Arrange
        await datasource.addToFailedQueue('document', 'doc1', 'create', {}, 'error1');
        await datasource.addToFailedQueue('document', 'doc2', 'update', {}, 'error2');

        // Act
        final size = datasource.getFailedQueueSize();

        // Assert
        expect(size, 2);
      });

      test('removeFromFailedQueue deletes item', () async {
        // Arrange
        await datasource.addToFailedQueue('document', 'doc1', 'create', {}, 'error');

        // Act
        await datasource.removeFromFailedQueue('document', 'doc1');
        final items = datasource.getFailedItems();

        // Assert
        expect(items, isEmpty);
      });

      test('clearFailedQueue removes all items', () async {
        // Arrange
        await datasource.addToFailedQueue('document', 'doc1', 'create', {}, 'error1');
        await datasource.addToFailedQueue('document', 'doc2', 'update', {}, 'error2');

        // Act
        await datasource.clearFailedQueue();
        final size = datasource.getFailedQueueSize();

        // Assert
        expect(size, 0);
      });
    });

    group('Document Cache Operations', () {
      test('cacheDocument and getCachedDocument work', () async {
        // Arrange
        final docData = {'id': 'doc1', 'title': 'Test Doc'};

        // Act
        await datasource.cacheDocument('doc1', docData);
        final result = datasource.getCachedDocument('doc1');

        // Assert
        expect(result, isNotNull);
        expect(result!['id'], 'doc1');
        expect(result['title'], 'Test Doc');
      });

      test('getCachedDocument returns null when not cached', () {
        // Act
        final result = datasource.getCachedDocument('nonexistent');

        // Assert
        expect(result, isNull);
      });

      test('removeCachedDocument deletes cached doc', () async {
        // Arrange
        await datasource.cacheDocument('doc1', {'id': 'doc1'});

        // Act
        await datasource.removeCachedDocument('doc1');
        final result = datasource.getCachedDocument('doc1');

        // Assert
        expect(result, isNull);
      });

      test('getCachedDocIds returns all IDs', () async {
        // Arrange
        await datasource.cacheDocument('doc1', {'id': 'doc1'});
        await datasource.cacheDocument('doc2', {'id': 'doc2'});

        // Act
        final ids = datasource.getCachedDocIds();

        // Assert
        expect(ids.length, 2);
        expect(ids, containsAll(['doc1', 'doc2']));
      });

      test('clearDocumentCache removes all cached data', () async {
        // Arrange
        await datasource.cacheDocument('doc1', {'id': 'doc1'});
        await datasource.cacheDocument('doc2', {'id': 'doc2'});

        // Act
        await datasource.clearDocumentCache();
        final ids = datasource.getCachedDocIds();

        // Assert
        expect(ids, isEmpty);
      });
    });

    group('Offline Content Operations', () {
      test('cacheContent and getCachedContent work', () async {
        // Arrange
        const content = '# Test Document\n\nThis is content.';

        // Act
        await datasource.cacheContent('doc1', content);
        final result = datasource.getCachedContent('doc1');

        // Assert
        expect(result, content);
      });

      test('getCachedContent returns null when not cached', () {
        // Act
        final result = datasource.getCachedContent('nonexistent');

        // Assert
        expect(result, isNull);
      });
    });

    group('Skipped Queue Operations', () {
      test('moveToSkippedQueue moves item from pending to skipped', () async {
        // Arrange
        await datasource.addToQueue('document', 'doc1', 'create', {});

        // Act
        final moved = await datasource.moveToSkippedQueue('document', 'doc1');
        final pendingItems = datasource.getPendingItems();
        final skippedItems = datasource.getSkippedItems();

        // Assert
        expect(moved, true);
        expect(pendingItems, isEmpty);
        expect(skippedItems.length, 1);
        expect(skippedItems[0]['entityType'], 'document');
        expect(skippedItems[0]['entityId'], 'doc1');
      });

      test('moveToSkippedQueue returns false when item not in pending queue', () async {
        // Act
        final moved = await datasource.moveToSkippedQueue('document', 'nonexistent');

        // Assert
        expect(moved, false);
      });

      test('getSkippedItems returns all skipped items', () async {
        // Arrange
        await datasource.addToQueue('document', 'doc1', 'create', {});
        await datasource.moveToSkippedQueue('document', 'doc1');

        // Act
        final items = datasource.getSkippedItems();

        // Assert
        expect(items.length, 1);
        expect(items[0]['entityType'], 'document');
      });

      test('getSkippedQueueSize returns correct count', () async {
        // Arrange
        await datasource.addToQueue('document', 'doc1', 'create', {});
        await datasource.addToQueue('document', 'doc2', 'update', {});
        await datasource.moveToSkippedQueue('document', 'doc1');
        await datasource.moveToSkippedQueue('document', 'doc2');

        // Act
        final size = datasource.getSkippedQueueSize();

        // Assert
        expect(size, 2);
      });

      test('clearSkippedQueue removes all skipped items', () async {
        // Arrange
        await datasource.addToQueue('document', 'doc1', 'create', {});
        await datasource.moveToSkippedQueue('document', 'doc1');

        // Act
        await datasource.clearSkippedQueue();
        final size = datasource.getSkippedQueueSize();

        // Assert
        expect(size, 0);
      });
    });

    group('Edge Cases', () {
      test('moveToSkippedQueue handles duplicate moves gracefully', () async {
        // Arrange
        await datasource.addToQueue('document', 'doc1', 'create', {});
        await datasource.moveToSkippedQueue('document', 'doc1');

        // Act - Try to move again
        final moved = await datasource.moveToSkippedQueue('document', 'doc1');

        // Assert - Should return false since item is no longer in pending queue
        expect(moved, false);
      });

      test('getAllQueues returns empty when no items', () {
        // Act
        final pending = datasource.getPendingItems();
        final failed = datasource.getFailedItems();
        final skipped = datasource.getSkippedItems();

        // Assert
        expect(pending, isEmpty);
        expect(failed, isEmpty);
        expect(skipped, isEmpty);
      });

      test('queue sizes are 0 when empty', () {
        // Act
        final pendingSize = datasource.getQueueSize();
        final failedSize = datasource.getFailedQueueSize();
        final skippedSize = datasource.getSkippedQueueSize();

        // Assert
        expect(pendingSize, 0);
        expect(failedSize, 0);
        expect(skippedSize, 0);
      });
    });
  });
}
