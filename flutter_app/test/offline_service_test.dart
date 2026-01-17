// T132: Offline service tests
// Testing OfflineService for offline-first document access
// Run with: flutter test test/offline_service_test.dart

import 'dart:async';
import 'dart:typed_data';

import 'package:connectivity_plus/connectivity_plus.dart';
import 'package:flutter_test/flutter_test.dart';

// Offline queue item for testing
class OfflineQueueItem {
  final String id;
  final String documentId;
  final Map<String, dynamic> data;
  final DateTime queuedAt;
  int retryCount;

  OfflineQueueItem({
    required this.id,
    required this.documentId,
    required this.data,
    required this.queuedAt,
    this.retryCount = 0,
  });
}

// Mock offline service for testing
class MockOfflineService {
  List<OfflineQueueItem> _queue = [];
  final Map<String, Uint8List> _cachedDocuments = {};
  ConnectivityResult _connectionStatus = ConnectivityResult.wifi;
  bool _isOnline = true;

  // Event streams
  final StreamController<ConnectivityResult> _connectivityController =
      StreamController<ConnectivityResult>.broadcast();
  final StreamController<OfflineQueueItem> _queueController =
      StreamController<OfflineQueueItem>.broadcast();

  Stream<ConnectivityResult> get connectivityChanges =>
      _connectivityController.stream;
  Stream<OfflineQueueItem> get queueChanges => _queueController.stream;

  // Queue management
  Future<void> addToQueue(String documentId, Map<String, dynamic> data) async {
    final item = OfflineQueueItem(
      id: 'item-${DateTime.now().millisecondsSinceEpoch}-${_queue.length}',
      documentId: documentId,
      data: data,
      queuedAt: DateTime.now(),
    );
    _queue.add(item);
    _queueController.add(item);
  }

  Future<OfflineQueueItem?> getNextQueueItem() async {
    if (_queue.isEmpty) return null;
    return _queue.first;
  }

  Future<void> removeFromQueue(String itemId) async {
    _queue.removeWhere((item) => item.id == itemId);
  }

  Future<void> incrementRetryCount(String itemId) async {
    final item = _queue.firstWhere((item) => item.id == itemId);
    item.retryCount++;
  }

  Future<int> getQueueCount() async => _queue.length;

  Future<void> clearQueue() async {
    _queue.clear();
  }

  // Document caching
  Future<void> cacheDocument(String documentId, Uint8List content) async {
    _cachedDocuments[documentId] = content;
  }

  Future<Uint8List?> getCachedDocument(String documentId) async => _cachedDocuments[documentId];

  Future<void> removeCachedDocument(String documentId) async {
    _cachedDocuments.remove(documentId);
  }

  Future<List<String>> getAllCachedDocumentIds() async => _cachedDocuments.keys.toList();

  Future<int> getCacheSize() async {
    var size = 0;
    for (final doc in _cachedDocuments.values) {
      size += doc.lengthInBytes;
    }
    return size;
  }

  // Connectivity
  Future<ConnectivityResult> checkConnectivity() async => _connectionStatus;

  void setConnectivity(ConnectivityResult status) {
    _connectionStatus = status;
    _isOnline = status != ConnectivityResult.none;
    _connectivityController.add(status);
  }

  bool get isOnline => _isOnline;

  // Reset state for test isolation
  void reset() {
    _queue = [];
    _cachedDocuments.clear();
    _connectionStatus = ConnectivityResult.wifi;
    _isOnline = true;
  }

  // Cleanup
  void dispose() {
    _connectivityController.close();
    _queueController.close();
  }
}

void main() {
  group('OfflineService - Queue Management', () {
    late MockOfflineService offlineService;

    setUp(() {
      offlineService = MockOfflineService();
    });

    tearDown(() {
      offlineService.dispose();
    });

    test('addToQueue increases queue count', () async {
      expect(await offlineService.getQueueCount(), 0);

      await offlineService.addToQueue('doc-1', {'operation': 'update'});
      expect(await offlineService.getQueueCount(), 1);

      await offlineService.addToQueue('doc-2', {'operation': 'create'});
      expect(await offlineService.getQueueCount(), 2);
    });

    test('removeFromQueue decreases queue count', () async {
      await offlineService.addToQueue('doc-1', {'operation': 'update'});
      await offlineService.addToQueue('doc-2', {'operation': 'create'});
      expect(await offlineService.getQueueCount(), 2);

      // Get and remove the first item
      final items = await offlineService.getNextQueueItem();
      expect(items, isNotNull);
      await offlineService.removeFromQueue(items!.id);
      expect(await offlineService.getQueueCount(), 1);
    });

    test('clearQueue removes all items', () async {
      await offlineService.addToQueue('doc-1', {'operation': 'update'});
      await offlineService.addToQueue('doc-2', {'operation': 'create'});
      await offlineService.addToQueue('doc-3', {'operation': 'delete'});

      expect(await offlineService.getQueueCount(), 3);

      await offlineService.clearQueue();
      expect(await offlineService.getQueueCount(), 0);
    });

    test('incrementRetryCount increases retry count', () async {
      await offlineService.addToQueue('doc-1', {'operation': 'update'});
      final item = await offlineService.getNextQueueItem();

      expect(item?.retryCount, 0);

      await offlineService.incrementRetryCount(item!.id);
      expect(item.retryCount, 1);

      await offlineService.incrementRetryCount(item.id);
      expect(item.retryCount, 2);
    });

    test('queue preserves order', () async {
      // Clear any previous state
      await offlineService.clearQueue();

      for (var i = 1; i <= 5; i++) {
        await offlineService.addToQueue('doc-$i', {'index': i});
      }

      for (var i = 1; i <= 5; i++) {
        final item = await offlineService.getNextQueueItem();
        expect(item, isNotNull);
        expect(item!.data['index'], i);
        await offlineService.removeFromQueue(item.id);
      }
    });
  });

  group('OfflineService - Document Caching', () {
    late MockOfflineService offlineService;

    setUp(() {
      offlineService = MockOfflineService();
    });

    tearDown(() {
      offlineService.dispose();
    });

    test('cacheDocument stores document', () async {
      final content = Uint8List.fromList([1, 2, 3, 4, 5]);

      await offlineService.cacheDocument('doc-1', content);
      final cached = await offlineService.getCachedDocument('doc-1');

      expect(cached, content);
    });

    test('getCachedDocument returns null for uncached document', () async {
      final cached = await offlineService.getCachedDocument('nonexistent');
      expect(cached, isNull);
    });

    test('removeCachedDocument removes document', () async {
      final content = Uint8List.fromList([1, 2, 3]);
      await offlineService.cacheDocument('doc-1', content);

      expect(await offlineService.getCachedDocument('doc-1'), isNotNull);

      await offlineService.removeCachedDocument('doc-1');
      expect(await offlineService.getCachedDocument('doc-1'), isNull);
    });

    test('getAllCachedDocumentIds returns all cached document IDs', () async {
      await offlineService.cacheDocument('doc-1', Uint8List.fromList([1]));
      await offlineService.cacheDocument('doc-2', Uint8List.fromList([2]));
      await offlineService.cacheDocument('doc-3', Uint8List.fromList([3]));

      final ids = await offlineService.getAllCachedDocumentIds();
      expect(ids.length, 3);
      expect(ids, containsAll(['doc-1', 'doc-2', 'doc-3']));
    });

    test('getCacheSize calculates total size', () async {
      await offlineService.cacheDocument('doc-1', Uint8List.fromList([1, 2, 3]));
      await offlineService.cacheDocument('doc-2', Uint8List.fromList([4, 5, 6, 7]));

      final size = await offlineService.getCacheSize();
      expect(size, 7); // 3 + 4 bytes
    });
  });

  group('OfflineService - Connectivity', () {
    late MockOfflineService offlineService;

    setUp(() {
      offlineService = MockOfflineService();
    });

    tearDown(() {
      offlineService.dispose();
    });

    test('checkConnectivity returns current status', () async {
      expect(await offlineService.checkConnectivity(), ConnectivityResult.wifi);

      offlineService.setConnectivity(ConnectivityResult.mobile);
      expect(await offlineService.checkConnectivity(), ConnectivityResult.mobile);
    });

    test('setConnectivity changes online status', () {
      expect(offlineService.isOnline, true);

      offlineService.setConnectivity(ConnectivityResult.none);
      expect(offlineService.isOnline, false);

      offlineService.setConnectivity(ConnectivityResult.wifi);
      expect(offlineService.isOnline, true);
    });

    test('connectivityChanges emits on setConnectivity', () async {
      final changes = <ConnectivityResult>[];
      final subscription = offlineService.connectivityChanges.listen(changes.add);

      // Give the stream listener time to register
      await Future.delayed(const Duration(milliseconds: 10));

      offlineService.setConnectivity(ConnectivityResult.mobile);
      offlineService.setConnectivity(ConnectivityResult.none);

      // Wait for events to be processed
      await Future.delayed(const Duration(milliseconds: 50));

      await subscription.cancel();

      expect(changes, contains(ConnectivityResult.mobile));
      expect(changes, contains(ConnectivityResult.none));
    });
  });

  group('OfflineService - Integration Scenarios', () {
    late MockOfflineService offlineService;

    setUp(() {
      offlineService = MockOfflineService();
    });

    tearDown(() {
      offlineService.dispose();
    });

    test('offline workflow: queue items while offline, process when online', () async {
      // Clear any previous state
      await offlineService.clearQueue();

      // Go offline
      offlineService.setConnectivity(ConnectivityResult.none);
      expect(offlineService.isOnline, false);

      // Queue updates while offline
      await offlineService.addToQueue('doc-1', {'content': 'Hello'});
      await offlineService.addToQueue('doc-2', {'content': 'World'});
      expect(await offlineService.getQueueCount(), 2);

      // Go online
      offlineService.setConnectivity(ConnectivityResult.wifi);
      expect(offlineService.isOnline, true);

      // Process queue
      final items = <OfflineQueueItem>[];
      for (var i = 0; i < 2; i++) {
        final item = await offlineService.getNextQueueItem();
        if (item != null) {
          items.add(item);
          await offlineService.removeFromQueue(item.id);
        }
      }

      expect(items.length, 2);
      expect(await offlineService.getQueueCount(), 0);
    });

    test('document caching survives connectivity changes', () async {
      // Cache document
      final content = Uint8List.fromList([1, 2, 3, 4, 5]);
      await offlineService.cacheDocument('doc-1', content);

      // Go offline
      offlineService.setConnectivity(ConnectivityResult.none);
      expect(offlineService.isOnline, false);

      // Document should still be available
      final cached = await offlineService.getCachedDocument('doc-1');
      expect(cached, content);

      // Go back online
      offlineService.setConnectivity(ConnectivityResult.wifi);
      expect(offlineService.isOnline, true);

      // Document still available
      final cached2 = await offlineService.getCachedDocument('doc-1');
      expect(cached2, content);
    });

    test('retry mechanism after sync failure', () async {
      await offlineService.addToQueue('doc-1', {'operation': 'update'});
      final item = await offlineService.getNextQueueItem();

      // Simulate first sync attempt (failure)
      await offlineService.incrementRetryCount(item!.id);
      expect(item.retryCount, 1);

      // Simulate second sync attempt (failure)
      await offlineService.incrementRetryCount(item.id);
      expect(item.retryCount, 2);

      // Simulate third attempt success
      await offlineService.removeFromQueue(item.id);
      expect(await offlineService.getQueueCount(), 0);
    });

    test('cache size tracking during heavy usage', () async {
      // Add multiple documents
      for (var i = 0; i < 10; i++) {
        final content = Uint8List.fromList(List.generate(100, (idx) => idx % 256));
        await offlineService.cacheDocument('doc-$i', content);
      }

      final size = await offlineService.getCacheSize();
      expect(size, 1000); // 10 documents * 100 bytes each

      // Remove half
      for (var i = 0; i < 5; i++) {
        await offlineService.removeCachedDocument('doc-$i');
      }

      final sizeAfter = await offlineService.getCacheSize();
      expect(sizeAfter, 500);
    });
  });

  group('OfflineQueueItem - Data Structure', () {
    test('OfflineQueueItem creation', () {
      final item = OfflineQueueItem(
        id: 'test-1',
        documentId: 'doc-1',
        data: {'operation': 'update'},
        queuedAt: DateTime(2024),
      );

      expect(item.id, 'test-1');
      expect(item.documentId, 'doc-1');
      expect(item.data['operation'], 'update');
      expect(item.retryCount, 0);
    });

    test('OfflineQueueItem with retry count', () {
      final item = OfflineQueueItem(
        id: 'test-1',
        documentId: 'doc-1',
        data: {},
        queuedAt: DateTime.now(),
        retryCount: 3,
      );

      expect(item.retryCount, 3);
    });
  });
}
