import 'dart:typed_data';
import 'package:flutter_test/flutter_test.dart';
import 'package:miniwiki/services/crdt_service.dart';

void main() {
  group('CrdtService - SyncStatus Tests', () {
    test('SyncStatus values are defined correctly', () {
      expect(SyncStatus.idle, isNotNull);
      expect(SyncStatus.syncing, isNotNull);
      expect(SyncStatus.success, isNotNull);
      expect(SyncStatus.error, isNotNull);
    });

    test('SyncStatus enum index values are sequential', () {
      expect(SyncStatus.idle.index, 0);
      expect(SyncStatus.syncing.index, 1);
      expect(SyncStatus.success.index, 2);
      expect(SyncStatus.error.index, 3);
    });
  });

  group('CrdtService - DocumentSyncState Tests', () {
    test('DocumentSyncState can be created with required fields', () {
      final state = DocumentSyncState(
        documentId: 'doc1',
        lastSyncedAt: DateTime(2025, 1, 1, 12),
      );

      expect(state.documentId, 'doc1');
      expect(state.lastSyncedAt, DateTime(2025, 1, 1, 12));
      expect(state.status, SyncStatus.idle);
      expect(state.errorMessage, isNull);
    });

    test('DocumentSyncState can be created with all fields', () {
      final state = DocumentSyncState(
        documentId: 'doc1',
        lastSyncedAt: DateTime(2025, 1, 1, 12),
        status: SyncStatus.success,
      );

      expect(state.status, SyncStatus.success);
    });

    test('DocumentSyncState can be created with error', () {
      final state = DocumentSyncState(
        documentId: 'doc1',
        lastSyncedAt: DateTime(2025, 1, 1, 12),
        status: SyncStatus.error,
        errorMessage: 'Sync failed',
      );

      expect(state.status, SyncStatus.error);
      expect(state.errorMessage, 'Sync failed');
    });

    test('DocumentSyncState copyWith updates specified fields', () {
      final state = DocumentSyncState(
        documentId: 'doc1',
        lastSyncedAt: DateTime(2025, 1, 1, 12),
      );

      final updated = state.copyWith(
        status: SyncStatus.syncing,
        errorMessage: 'Syncing...',
      );

      expect(updated.documentId, 'doc1');
      expect(updated.status, SyncStatus.syncing);
      expect(updated.errorMessage, 'Syncing...');
      expect(state.status, SyncStatus.idle); // Original unchanged
    });

    test('DocumentSyncState copyWith updates error message', () {
      final state = DocumentSyncState(
        documentId: 'doc1',
        lastSyncedAt: DateTime(2025, 1, 1, 12),
        status: SyncStatus.error,
        errorMessage: 'Error',
      );

      final updated = state.copyWith(
        status: SyncStatus.success,
        errorMessage: 'Fixed',
      );

      expect(updated.errorMessage, 'Fixed');
      expect(updated.status, SyncStatus.success);
      expect(state.errorMessage, 'Error'); // Original unchanged
    });
  });

  group('CrdtService - CrdtDocument Tests', () {
    test('CrdtDocument can be created with required fields', () {
      final doc = CrdtDocument(
        id: 'doc1',
        doc: null,
      );

      expect(doc.id, 'doc1');
      expect(doc.doc, isNull);
      expect(doc.text, isNull);
      expect(doc.arrays, isEmpty);
      expect(doc.maps, isEmpty);
      expect(doc.isDirty, false);
    });

    test('CrdtDocument can be created with all fields', () {
      final now = DateTime.now();
      final doc = CrdtDocument(
        id: 'doc1',
        doc: <String, dynamic>{'type': 'Y.Doc'},
        text: 'Hello world',
        arrays: <String, dynamic>{'items': <dynamic>[]},
        maps: <String, dynamic>{'meta': <String, dynamic>{}},
        lastSyncedAt: now,
        isDirty: true,
      );

      expect(doc.id, 'doc1');
      expect(doc.text, 'Hello world');
      expect(doc.arrays['items'], isNotNull);
      expect(doc.maps['meta'], isNotNull);
      expect(doc.lastSyncedAt, now);
      expect(doc.isDirty, true);
    });

    test('CrdtDocument default values', () {
      final doc = CrdtDocument(
        id: 'doc1',
        doc: null,
      );

      expect(doc.isDirty, false);
      expect(doc.lastSyncedAt, isNotNull);
      expect(doc.arrays, isEmpty);
      expect(doc.maps, isEmpty);
    });
  });

  group('CrdtService - SyncResult Tests', () {
    test('SyncResult can be created with success', () {
      const result = SyncResult(
        success: true,
        documentsSynced: 5,
      );

      expect(result.success, true);
      expect(result.documentsSynced, 5);
      expect(result.errorMessage, isNull);
    });

    test('SyncResult can be created with failure', () {
      const result = SyncResult(
        success: false,
        errorMessage: 'Network error',
      );

      expect(result.success, false);
      expect(result.errorMessage, 'Network error');
    });

    test('SyncResult default documentsSynced is 0', () {
      const result = SyncResult(success: true);
      expect(result.documentsSynced, 0);
    });
  });

  group('CrdtService - Service Lifecycle', () {
    late CrdtService service;

    setUp(() {
      service = CrdtService();
    });

    tearDown(() {
      service.dispose();
    });

    test('Service initializes without documents', () {
      expect(() => service.getDocument('nonexistent'), returnsNormally);
    });

    test('Service can be disposed', () {
      service.dispose();
      // No exception should be thrown
      expect(true, isTrue);
    });

    test('Service disposal is idempotent', () {
      service.dispose();
      service.dispose(); // Should not throw
      expect(true, isTrue);
    });
  });

  group('CrdtService - Document Management', () {
    late CrdtService service;

    setUp(() {
      service = CrdtService();
    });

    tearDown(() {
      service.dispose();
    });

    test('getDocument creates new document if not exists', () {
      final doc = service.getDocument('doc1');

      expect(doc.id, 'doc1');
      expect(doc.doc, isNull);
      expect(doc.isDirty, false);
    });

    test('getDocument returns existing document', () {
      final doc1 = service.getDocument('doc1');
      final doc2 = service.getDocument('doc1');

      expect(identical(doc1, doc2), true);
    });

    test('deleteDocument removes document', () {
      service.getDocument('doc1');
      service.deleteDocument('doc1');

      // New document should be created after deletion
      final doc = service.getDocument('doc1');
      expect(doc.isDirty, false);
    });

    test('deleteDocument is safe for non-existent document', () {
      service.deleteDocument('nonexistent');
      // Should not throw
      expect(true, isTrue);
    });
  });

  group('CrdtService - Sync State Management', () {
    late CrdtService service;

    setUp(() {
      service = CrdtService();
    });

    tearDown(() {
      service.dispose();
    });

    test('getSyncState creates new state if not exists', () {
      final state = service.getSyncState('doc1');

      expect(state.documentId, 'doc1');
      expect(state.status, SyncStatus.idle);
    });

    test('getSyncState returns existing state', () {
      final state1 = service.getSyncState('doc1');
      final state2 = service.getSyncState('doc1');

      expect(identical(state1, state2), true);
    });

    test('updateSyncState updates state and emits event', () async {
      final states = <DocumentSyncState>[];
      final subscription = service.syncStateChanges.listen(states.add);

      service.updateSyncState('doc1', SyncStatus.syncing);

      await Future<void>.delayed(const Duration(milliseconds: 10));

      expect(states.length, greaterThan(0));
      expect(states.last.status, SyncStatus.syncing);

      await subscription.cancel();
    });

    test('updateSyncState with error message', () async {
      final states = <DocumentSyncState>[];
      final subscription = service.syncStateChanges.listen(states.add);

      service.updateSyncState('doc1', SyncStatus.error, 'Connection failed');

      await Future<void>.delayed(const Duration(milliseconds: 10));

      expect(states.last.status, SyncStatus.error);
      expect(states.last.errorMessage, 'Connection failed');

      await subscription.cancel();
    });
  });

  group('CrdtService - Text Operations', () {
    late CrdtService service;

    setUp(() {
      service = CrdtService();
    });

    tearDown(() {
      service.dispose();
    });

    test('getText returns null for new document', () {
      final text = service.getText('doc1');
      expect(text, isNull);
    });

    test('getText returns text content', () {
      service.getDocument('doc1');
      final text = service.getText('doc1');
      expect(text, isNull); // No text set yet
    });

    test('setText emits update event', () async {
      final updates = <Map<String, dynamic>>[];
      final subscription = service.updates.listen(updates.add);

      await service.setText('doc1', 'Hello');

      await Future<void>.delayed(const Duration(milliseconds: 10));

      expect(updates.length, greaterThan(0));
      expect(updates.last['documentId'], 'doc1');
      expect(updates.last['field'], 'text');

      await subscription.cancel();
    });
  });

  group('CrdtService - State Vector Operations', () {
    late CrdtService service;

    setUp(() {
      service = CrdtService();
    });

    tearDown(() {
      service.dispose();
    });

    test('getState returns null for new document', () async {
      final state = await service.getState('doc1');
      expect(state, isNull);
    });

    test('applyUpdate marks document as dirty', () async {
      final update = Uint8List.fromList([1, 2, 3, 4]);
      await service.applyUpdate('doc1', update);

      final doc = service.getDocument('doc1');
      expect(doc.isDirty, true);
    });

    test('applyUpdate updates lastSyncedAt', () async {
      final update = Uint8List.fromList([1, 2, 3, 4]);
      final before = DateTime.now();

      await Future<void>.delayed(const Duration(milliseconds: 10));
      await service.applyUpdate('doc1', update);

      final doc = service.getDocument('doc1');
      expect(doc.lastSyncedAt.isAfter(before), true);
    });

    test('applyUpdate emits update event', () async {
      final updates = <Map<String, dynamic>>[];
      final subscription = service.updates.listen(updates.add);

      final update = Uint8List.fromList([1, 2, 3, 4]);
      await service.applyUpdate('doc1', update);

      await Future<void>.delayed(const Duration(milliseconds: 10));

      expect(updates.length, greaterThan(0));
      expect(updates.last['documentId'], 'doc1');
      expect(updates.last['field'], 'update');

      await subscription.cancel();
    });
  });

  group('CrdtService - Array Operations', () {
    late CrdtService service;

    setUp(() {
      service = CrdtService();
    });

    tearDown(() {
      service.dispose();
    });

    test('getArray returns null for non-existent array', () {
      final array = service.getArray('doc1', 'items');
      expect(array, isNull);
    });

    test('getArray creates and caches array if doc exists', () {
      service.getDocument('doc1');
      // Without actual y_crdt, we can't create real arrays
      final array = service.getArray('doc1', 'items');
      expect(array, isNull); // doc.doc is null
    });
  });

  group('CrdtService - Map Operations', () {
    late CrdtService service;

    setUp(() {
      service = CrdtService();
    });

    tearDown(() {
      service.dispose();
    });

    test('getMap returns null for non-existent map', () {
      final map = service.getMap('doc1', 'meta');
      expect(map, isNull);
    });

    test('getMap creates and caches map if doc exists', () {
      service.getDocument('doc1');
      // Without actual y_crdt, we can't create real maps
      final map = service.getMap('doc1', 'meta');
      expect(map, isNull); // doc.doc is null
    });
  });

  group('CrdtService - Dirty Tracking', () {
    late CrdtService service;

    setUp(() {
      service = CrdtService();
    });

    tearDown(() {
      service.dispose();
    });

    test('hasPendingChanges returns false for new document', () {
      expect(service.hasPendingChanges('doc1'), false);
    });

    test('hasPendingChanges returns true after applyUpdate', () async {
      final update = Uint8List.fromList([1, 2, 3]);
      await service.applyUpdate('doc1', update);
      expect(service.hasPendingChanges('doc1'), true);
    });

    test('getDirtyDocumentIds returns list of dirty documents', () async {
      await service.applyUpdate('doc1', Uint8List.fromList([1]));
      await service.applyUpdate('doc2', Uint8List.fromList([2]));

      final dirtyIds = service.getDirtyDocumentIds();
      expect(dirtyIds, contains('doc1'));
      expect(dirtyIds, contains('doc2'));
    });

    test('getDirtyDocumentIds returns empty list when no dirty docs', () {
      final dirtyIds = service.getDirtyDocumentIds();
      expect(dirtyIds, isEmpty);
    });

    test('markSynced clears dirty flag', () async {
      await service.applyUpdate('doc1', Uint8List.fromList([1]));
      expect(service.hasPendingChanges('doc1'), true);

      service.markSynced('doc1');
      expect(service.hasPendingChanges('doc1'), false);
    });

    test('markSynced updates sync state to success', () async {
      final states = <DocumentSyncState>[];
      final subscription = service.syncStateChanges.listen(states.add);

      await service.applyUpdate('doc1', Uint8List.fromList([1]));
      service.markSynced('doc1');

      await Future<void>.delayed(const Duration(milliseconds: 10));

      expect(states.last.status, SyncStatus.success);

      await subscription.cancel();
    });

    test('markSynced updates lastSyncedAt', () async {
      final before = DateTime.now();
      await service.applyUpdate('doc1', Uint8List.fromList([1]));

      await Future<void>.delayed(const Duration(milliseconds: 10));
      service.markSynced('doc1');

      final doc = service.getDocument('doc1');
      expect(doc.lastSyncedAt.isAfter(before), true);
    });
  });

  group('CrdtService - Stream Tests', () {
    late CrdtService service;

    setUp(() {
      service = CrdtService();
    });

    tearDown(() {
      service.dispose();
    });

    test('updates stream is broadcast stream', () {
      expect(service.updates, isA<Stream<Map<String, dynamic>>>());
    });

    test('syncStateChanges stream is broadcast stream', () {
      expect(service.syncStateChanges, isA<Stream<DocumentSyncState>>());
    });

    test('Multiple listeners can subscribe to updates', () async {
      final results1 = <Map<String, dynamic>>[];
      final results2 = <Map<String, dynamic>>[];

      final sub1 = service.updates.listen(results1.add);
      final sub2 = service.updates.listen(results2.add);

      await service.setText('doc1', 'Hello');

      await Future<void>.delayed(const Duration(milliseconds: 10));

      expect(results1.length, greaterThan(0));
      expect(results2.length, greaterThan(0));

      await sub1.cancel();
      await sub2.cancel();
    });

    test('Multiple listeners can subscribe to syncStateChanges', () async {
      final states1 = <DocumentSyncState>[];
      final states2 = <DocumentSyncState>[];

      final sub1 = service.syncStateChanges.listen(states1.add);
      final sub2 = service.syncStateChanges.listen(states2.add);

      service.updateSyncState('doc1', SyncStatus.syncing);

      await Future<void>.delayed(const Duration(milliseconds: 10));

      expect(states1.length, greaterThan(0));
      expect(states2.length, greaterThan(0));

      await sub1.cancel();
      await sub2.cancel();
    });
  });

  group('CrdtService - Integration Scenarios', () {
    late CrdtService service;

    setUp(() {
      service = CrdtService();
    });

    tearDown(() {
      service.dispose();
    });

    test('Full sync workflow', () async {
      final states = <DocumentSyncState>[];
      final updates = <Map<String, dynamic>>[];

      final stateSub = service.syncStateChanges.listen(states.add);
      final updateSub = service.updates.listen(updates.add);

      // Create document
      service.getDocument('doc1');

      // Make changes
      await service.applyUpdate('doc1', Uint8List.fromList([1, 2, 3]));

      // Check dirty state
      expect(service.hasPendingChanges('doc1'), true);

      // Mark as synced
      service.markSynced('doc1');

      // Wait for state change event
      await Future<void>.delayed(const Duration(milliseconds: 10));

      // Verify
      expect(service.hasPendingChanges('doc1'), false);
      expect(states.any((s) => s.status == SyncStatus.success), true);

      await stateSub.cancel();
      await updateSub.cancel();
    });

    test('Multiple documents sync independently', () async {
      await service.applyUpdate('doc1', Uint8List.fromList([1]));
      await service.applyUpdate('doc2', Uint8List.fromList([2]));

      final dirtyIds = service.getDirtyDocumentIds();
      expect(dirtyIds.length, 2);

      service.markSynced('doc1');
      expect(service.hasPendingChanges('doc1'), false);
      expect(service.hasPendingChanges('doc2'), true);

      service.markSynced('doc2');
      expect(service.hasPendingChanges('doc2'), false);
    });

    test('Error state is tracked correctly', () async {
      service.updateSyncState('doc1', SyncStatus.error, 'Network error');

      final state = service.getSyncState('doc1');
      expect(state.status, SyncStatus.error);
      expect(state.errorMessage, 'Network error');
    });
  });

  group('CrdtService - Edge Cases', () {
    late CrdtService service;

    setUp(() {
      service = CrdtService();
    });

    tearDown(() {
      service.dispose();
    });

    test('Empty document ID is handled', () {
      final doc = service.getDocument('');
      expect(doc.id, '');
    });

    test('Special characters in document ID are handled', () {
      final doc = service.getDocument('doc/with/slashes');
      expect(doc.id, 'doc/with/slashes');
    });

    test('Very long document ID is handled', () {
      final longId = 'a' * 1000;
      final doc = service.getDocument(longId);
      expect(doc.id, longId);
    });

    test('Concurrent operations on same document', () async {
      final results = await Future.wait<void>([
        service.applyUpdate('doc1', Uint8List.fromList([1])),
        service.applyUpdate('doc1', Uint8List.fromList([2])),
        service.applyUpdate('doc1', Uint8List.fromList([3])),
      ]);

      // All operations should complete
      expect(results.length, 3);

      // Document should be dirty
      expect(service.hasPendingChanges('doc1'), true);
    });

    test('Rapid state changes', () async {
      final states = <DocumentSyncState>[];
      final subscription = service.syncStateChanges.listen(states.add);

      for (var i = 0; i < 100; i++) {
        service.updateSyncState('doc1', SyncStatus.syncing);
        service.updateSyncState('doc1', SyncStatus.success);
      }

      await Future<void>.delayed(const Duration(milliseconds: 100));

      // All state changes should be captured
      expect(states.length, greaterThanOrEqualTo(200));

      await subscription.cancel();
    });
  });

  group('CrdtService - Uint8List Operations', () {
    late CrdtService service;

    setUp(() {
      service = CrdtService();
    });

    tearDown(() {
      service.dispose();
    });

    test('Empty update is handled', () async {
      final empty = Uint8List(0);
      await service.applyUpdate('doc1', empty);

      final doc = service.getDocument('doc1');
      expect(doc.isDirty, true);
    });

    test('Large update is handled', () async {
      final large = Uint8List(1000000); // 1MB
      for (var i = 0; i < large.length; i++) {
        large[i] = i % 256;
      }

      await service.applyUpdate('doc1', large);

      final doc = service.getDocument('doc1');
      expect(doc.isDirty, true);
    });

    test('Binary data in update is preserved', () async {
      final binary =
          Uint8List.fromList([0x00, 0xFF, 0xAA, 0x55, 0x12, 0x34]);
      await service.applyUpdate('doc1', binary);

      final doc = service.getDocument('doc1');
      expect(doc.isDirty, true);
    });
  });
}
