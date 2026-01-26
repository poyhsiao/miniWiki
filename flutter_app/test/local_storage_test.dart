import 'package:flutter_test/flutter_test.dart';
import 'package:miniwiki/data/datasources/local_storage.dart';
import 'package:shared_preferences/shared_preferences.dart';

void main() {
  // Setup mock preferences for all tests
  setUpAll(() async {
    TestWidgetsFlutterBinding.ensureInitialized();
    SharedPreferences.setMockInitialValues({});
  });

  group('LocalStorageService Tests', () {
    late LocalStorageService storage;

    setUp(() async {
      storage = await LocalStorageService.getInstance();
      // Clear any existing data before each test
      await storage.clear();
    });

    test('LocalStorageService getInstance returns singleton', () async {
      // Act
      final instance1 = await LocalStorageService.getInstance();
      final instance2 = await LocalStorageService.getInstance();

      // Assert - Should return same instance
      expect(instance1, equals(instance2));
    });

    test('LocalStorageService setString and getString work', () async {
      // Arrange
      final storage = await LocalStorageService.getInstance();

      // Act
      await storage.setString('test_key', 'test_value');
      final result = storage.getString('test_key');

      // Assert
      expect(result, 'test_value');
    });

    test('LocalStorageService getString returns null for non-existent key', () async {
      // Arrange
      final storage = await LocalStorageService.getInstance();

      // Act
      final result = storage.getString('nonexistent_key');

      // Assert
      expect(result, isNull);
    });

    test('LocalStorageService getKeysByPrefix filters correctly', () async {
      // Arrange
      final storage = await LocalStorageService.getInstance();
      await storage.setString('prefix_key1', 'value1');
      await storage.setString('prefix_key2', 'value2');
      await storage.setString('other_key', 'value3');

      // Act
      final keys = storage.getKeysByPrefix('prefix_');

      // Assert
      expect(keys.length, 2);
      expect(keys, contains('prefix_key1'));
      expect(keys, contains('prefix_key2'));
      expect(keys, isNot(contains('other_key')));
    });

    test('LocalStorageService getValuesByPrefix returns values', () async {
      // Arrange
      final storage = await LocalStorageService.getInstance();
      await storage.setString('data_a', 'value_a');
      await storage.setString('data_b', 'value_b');
      await storage.setString('other_x', 'value_x');

      // Act
      final values = storage.getValuesByPrefix('data_');

      // Assert
      expect(values.length, 2);
      expect(values, contains('value_a'));
      expect(values, contains('value_b'));
    });

    test('LocalStorageService remove deletes key', () async {
      // Arrange
      final storage = await LocalStorageService.getInstance();
      await storage.setString('to_remove', 'value');

      // Act
      final removed = await storage.remove('to_remove');
      final result = storage.getString('to_remove');

      // Assert
      expect(removed, true);
      expect(result, isNull);
    });

    test('LocalStorageService removeByPrefix deletes multiple keys', () async {
      // Arrange
      final storage = await LocalStorageService.getInstance();
      await storage.setString('temp_key1', 'value1');
      await storage.setString('temp_key2', 'value2');
      await storage.setString('keep_key', 'value3');

      // Act
      await storage.removeByPrefix('temp_');
      final keys1 = storage.getKeysByPrefix('temp_');
      final keys2 = storage.getString('keep_key');

      // Assert
      expect(keys1, isEmpty);
      expect(keys2, 'value3');
    });

    test('LocalStorageService containsKey works correctly', () async {
      // Arrange
      final storage = await LocalStorageService.getInstance();
      await storage.setString('exists', 'value');

      // Act
      final exists = storage.containsKey('exists');
      final notExists = storage.containsKey('not_exists');

      // Assert
      expect(exists, true);
      expect(notExists, false);
    });

    test('LocalStorageService clear removes all data', () async {
      // Arrange
      final storage = await LocalStorageService.getInstance();
      await storage.setString('key1', 'value1');
      await storage.setString('key2', 'value2');

      // Act
      await storage.clear();
      final result1 = storage.getString('key1');
      final result2 = storage.getString('key2');

      // Assert
      expect(result1, isNull);
      expect(result2, isNull);
    });
  });

  group('DocumentCacheService Tests', () {
    late LocalStorageService storage;
    late DocumentCacheService docCache;

    setUp(() async {
      storage = await LocalStorageService.getInstance();
      await storage.clear(); // Clear before each test
      docCache = DocumentCacheService(storage);
    });

    test('DocumentCacheService caches and retrieves document', () async {
      // Arrange
      final docData = {'id': 'doc1', 'title': 'Test Doc'};

      // Act
      await docCache.cacheDocument('doc1', docData);
      final result = docCache.getDocument('doc1');

      // Assert
      expect(result, isNotNull);
      expect(result!['id'], 'doc1');
      expect(result['title'], 'Test Doc');
    });

    test('DocumentCacheService getDocument returns null when not cached', () {
      // Act
      final result = docCache.getDocument('nonexistent');

      // Assert
      expect(result, isNull);
    });

    test('DocumentCacheService removeDocument deletes cached doc', () async {
      // Arrange
      await docCache.cacheDocument('doc1', {'id': 'doc1'});

      // Act
      await docCache.removeDocument('doc1');
      final result = docCache.getDocument('doc1');

      // Assert
      expect(result, isNull);
    });

    test('DocumentCacheService getCachedDocIds returns all IDs', () async {
      // Arrange
      await docCache.cacheDocument('doc1', {'id': 'doc1'});
      await docCache.cacheDocument('doc2', {'id': 'doc2'});

      // Act
      final ids = docCache.getCachedDocIds();

      // Assert
      expect(ids.length, 2);
      expect(ids, containsAll(['doc1', 'doc2']));
    });

    test('DocumentCacheService cacheContent and getCachedContent work', () async {
      // Arrange
      const content = '# Test Document\n\nThis is content.';

      // Act
      await docCache.cacheContent('doc1', content);
      final result = docCache.getCachedContent('doc1');

      // Assert
      expect(result, content);
    });

    test('DocumentCacheService clearCache removes all cached data', () async {
      // Arrange
      await docCache.cacheDocument('doc1', {'id': 'doc1'});
      await docCache.cacheContent('doc1', 'content');

      // Act
      await docCache.clearCache();
      final doc = docCache.getDocument('doc1');
      final content = docCache.getCachedContent('doc1');

      // Assert
      expect(doc, isNull);
      expect(content, isNull);
    });
  });

  group('SyncQueueService Tests', () {
    late LocalStorageService storage;
    late SyncQueueService syncQueue;

    setUp(() async {
      storage = await LocalStorageService.getInstance();
      await storage.clear(); // Clear before each test
      syncQueue = SyncQueueService(storage);
    });

    test('SyncQueueService addItem and getQueueItems work', () async {
      // Arrange
      final data = {'title': 'Test'};

      // Act
      await syncQueue.addItem('document', 'doc1', 'create', data);
      final items = syncQueue.getQueueItems();

      // Assert
      expect(items.length, 1);
      expect(items[0]['entityType'], 'document');
      expect(items[0]['entityId'], 'doc1');
      expect(items[0]['operation'], 'create');
    });

    test('SyncQueueService getQueueSize returns correct count', () async {
      // Arrange
      await syncQueue.addItem('document', 'doc1', 'create', {});
      await syncQueue.addItem('document', 'doc2', 'update', {});

      // Act
      final size = syncQueue.getQueueSize();

      // Assert
      expect(size, 2);
    });

    test('SyncQueueService removeItem deletes from queue', () async {
      // Arrange
      await syncQueue.addItem('document', 'doc1', 'create', {});

      // Act
      await syncQueue.removeItem('document', 'doc1');
      final items = syncQueue.getQueueItems();

      // Assert
      expect(items, isEmpty);
    });

    test('SyncQueueService clearQueue removes all items', () async {
      // Arrange
      await syncQueue.addItem('document', 'doc1', 'create', {});
      await syncQueue.addItem('document', 'doc2', 'update', {});

      // Act
      await syncQueue.clearQueue();
      final size = syncQueue.getQueueSize();

      // Assert
      expect(size, 0);
    });

    test('SyncQueueService addFailedItem and getFailedItems work', () async {
      // Arrange
      final data = {'title': 'Test'};
      const error = 'Network error';

      // Act
      await syncQueue.addFailedItem('document', 'doc1', 'create', data, error);
      final items = syncQueue.getFailedItems();

      // Assert
      expect(items.length, 1);
      expect(items[0]['entityType'], 'document');
      expect(items[0]['error'], error);
    });

    test('SyncQueueService getFailedQueueSize returns correct count', () async {
      // Arrange
      await syncQueue.addFailedItem('document', 'doc1', 'create', {}, 'error1');
      await syncQueue.addFailedItem('document', 'doc2', 'update', {}, 'error2');

      // Act
      final size = syncQueue.getFailedQueueSize();

      // Assert
      expect(size, 2);
    });

    test('SyncQueueService removeFailedItem deletes from failed queue', () async {
      // Arrange
      await syncQueue.addFailedItem('document', 'doc1', 'create', {}, 'error');

      // Act
      await syncQueue.removeFailedItem('document', 'doc1');
      final items = syncQueue.getFailedItems();

      // Assert
      expect(items, isEmpty);
    });

    test('SyncQueueService clearFailedQueue removes all failed items', () async {
      // Arrange
      await syncQueue.addFailedItem('document', 'doc1', 'create', {}, 'error1');
      await syncQueue.addFailedItem('document', 'doc2', 'update', {}, 'error2');

      // Act
      await syncQueue.clearFailedQueue();
      final size = syncQueue.getFailedQueueSize();

      // Assert
      expect(size, 0);
    });

    test('SyncQueueService addSkippedItem and getSkippedItems work', () async {
      // Act
      await syncQueue.addSkippedItem('unknown_entity', 'entity1');
      final items = syncQueue.getSkippedItems();

      // Assert
      expect(items.length, 1);
      expect(items[0]['entityType'], 'unknown_entity');
      expect(items[0]['entityId'], 'entity1');
      expect(items[0]['reason'], 'Unsupported entity type');
    });

    test('SyncQueueService getSkippedQueueSize returns correct count', () async {
      // Arrange
      await syncQueue.addSkippedItem('entity1', 'id1');
      await syncQueue.addSkippedItem('entity2', 'id2');

      // Act
      final size = syncQueue.getSkippedQueueSize();

      // Assert
      expect(size, 2);
    });

    test('SyncQueueService removeSkippedItem deletes from skipped queue', () async {
      // Arrange
      await syncQueue.addSkippedItem('document', 'doc1');

      // Act
      await syncQueue.removeSkippedItem('document', 'doc1');
      final items = syncQueue.getSkippedItems();

      // Assert
      expect(items, isEmpty);
    });

    test('SyncQueueService clearSkippedQueue removes all skipped items', () async {
      // Arrange
      await syncQueue.addSkippedItem('document', 'doc1');
      await syncQueue.addSkippedItem('document', 'doc2');

      // Act
      await syncQueue.clearSkippedQueue();
      final size = syncQueue.getSkippedQueueSize();

      // Assert
      expect(size, 0);
    });
  });

  group('UserCacheService Tests', () {
    late LocalStorageService storage;
    late UserCacheService userCache;

    setUp(() async {
      storage = await LocalStorageService.getInstance();
      await storage.clear(); // Clear before each test
      userCache = UserCacheService(storage);
    });

    test('UserCacheService cacheUser and getCachedUser work', () async {
      // Arrange
      final userData = {
        'id': 'user1',
        'name': 'Test User',
        'email': 'test@example.com',
      };

      // Act
      await userCache.cacheUser(userData);
      final result = userCache.getCachedUser();

      // Assert
      expect(result, isNotNull);
      expect(result!['id'], 'user1');
      expect(result['name'], 'Test User');
      expect(result['email'], 'test@example.com');
    });

    test(
      'UserCacheService getCachedUser returns null when not cached',
      () async {
        // First clear any existing data
        await userCache.clearCache();

        // Act
        final result = userCache.getCachedUser();

        // Assert
        expect(result, isNull);
      },
    );

    test('UserCacheService clearCache removes user data', () async {
      // Arrange
      await userCache.cacheUser({'id': 'user1'});

      // Act
      await userCache.clearCache();
      final result = userCache.getCachedUser();

      // Assert
      expect(result, isNull);
    });
  });

  group('SpaceCacheService Tests', () {
    late LocalStorageService storage;
    late SpaceCacheService spaceCache;

    setUp(() async {
      storage = await LocalStorageService.getInstance();
      await storage.clear(); // Clear before each test
      spaceCache = SpaceCacheService(storage);
    });

    test('SpaceCacheService cacheSpace and getSpace work', () async {
      // Arrange
      final spaceData = {
        'id': 'space1',
        'name': 'Test Space',
        'description': 'A test space',
      };

      // Act
      await spaceCache.cacheSpace('space1', spaceData);
      final result = spaceCache.getSpace('space1');

      // Assert
      expect(result, isNotNull);
      expect(result!['id'], 'space1');
      expect(result['name'], 'Test Space');
    });

    test('SpaceCacheService getSpace returns null when not cached', () {
      // Act
      final result = spaceCache.getSpace('nonexistent');

      // Assert
      expect(result, isNull);
    });

    test('SpaceCacheService getCachedSpaceIds returns all IDs', () async {
      // Arrange
      await spaceCache.cacheSpace('space1', {'id': 'space1'});
      await spaceCache.cacheSpace('space2', {'id': 'space2'});

      // Act
      final ids = spaceCache.getCachedSpaceIds();

      // Assert
      expect(ids.length, 2);
      expect(ids, containsAll(['space1', 'space2']));
    });

    test('SpaceCacheService clearCache removes all spaces', () async {
      // Arrange
      await spaceCache.cacheSpace('space1', {'id': 'space1'});
      await spaceCache.cacheSpace('space2', {'id': 'space2'});

      // Act
      await spaceCache.clearCache();
      final ids = spaceCache.getCachedSpaceIds();

      // Assert
      expect(ids, isEmpty);
    });
  });

  group('initLocalStorage Tests', () {
    test('initLocalStorage initializes global storage', () async {
      // Act
      final storage = await initLocalStorage();

      // Assert
      expect(storage, isA<LocalStorageService>());
    });

    test('initLocalStorage returns same instance on subsequent calls', () async {
      // Act
      final storage1 = await initLocalStorage();
      final storage2 = await initLocalStorage();

      // Assert
      expect(storage1, equals(storage2));
    });
  });
}
