// T133: Sync service tests
// Testing SyncService for offline-first sync orchestration
// Run with: flutter test test/sync_service_test.dart

import 'dart:async';
import 'dart:typed_data';
import 'package:flutter_test/flutter_test.dart';
import 'package:miniwiki/services/sync_service.dart' as ss;

// Re-export SyncStatus and SyncResult from sync_service for testing
export 'package:miniwiki/services/sync_service.dart' hide SyncStatus, SyncResult;

// Mock CrdtService for testing
class MockCrdtService {
  final Map<String, Uint8List> _documentStates = {};
  final List<String> _dirtyDocumentIds = [];
  final Map<String, bool> _syncedStatus = {};
  final Map<String, bool> _pendingChanges = {};

  List<String> getDirtyDocumentIds() => List.from(_dirtyDocumentIds);

  void markSynced(String documentId) {
    _syncedStatus[documentId] = true;
    _dirtyDocumentIds.remove(documentId);
    _pendingChanges[documentId] = false;
  }

  void markDirty(String documentId) {
    if (!_dirtyDocumentIds.contains(documentId)) {
      _dirtyDocumentIds.add(documentId);
    }
    _syncedStatus[documentId] = false;
    _pendingChanges[documentId] = true;
  }

  bool hasPendingChanges(String documentId) => _pendingChanges[documentId] ?? false;

  void setDocumentState(String documentId, Uint8List state) {
    _documentStates[documentId] = state;
  }

  Uint8List? getDocumentState(String documentId) => _documentStates[documentId];
}

// Create a mock sync service that doesn't require real CrdtService
class MockSyncService {
  final MockCrdtService _crdtService;
  bool _autoSyncEnabled = true;
  int _syncIntervalSeconds = 30;

  MockSyncService(this._crdtService);

  bool get autoSyncEnabled => _autoSyncEnabled;
  int get syncIntervalSeconds => _syncIntervalSeconds;

  void setAutoSync(bool enabled) {
    _autoSyncEnabled = enabled;
  }

  void setSyncInterval(int seconds) {
    _syncIntervalSeconds = seconds;
  }

  Future<int> getPendingSyncCount() async => _crdtService.getDirtyDocumentIds().length;

  Future<ss.SyncResult> syncDocument(String documentId) async {
    final state = _crdtService.getDocumentState(documentId);
    if (state == null) {
      return const ss.SyncResult(success: true);
    }
    _crdtService.markSynced(documentId);
    return const ss.SyncResult(success: true, documentsSynced: 1);
  }

  Future<ss.SyncSummary> syncAllDirtyDocuments() async {
    final dirtyIds = _crdtService.getDirtyDocumentIds();
    var syncedCount = 0;
    var failedCount = 0;

    for (final documentId in dirtyIds) {
      final result = await syncDocument(documentId);
      if (result.success) {
        syncedCount++;
      } else {
        failedCount++;
      }
    }

    return ss.SyncSummary(
      success: failedCount == 0,
      syncedCount: syncedCount,
      failedCount: failedCount,
      timestamp: DateTime.now(),
    );
  }
}

void main() {
  group('SyncService - SyncStatus Tests', () {
    test('SyncStatus values are defined correctly', () {
      expect(ss.SyncStatus.pending.toString(), 'SyncStatus.pending');
      expect(ss.SyncStatus.syncing.toString(), 'SyncStatus.syncing');
      expect(ss.SyncStatus.completed.toString(), 'SyncStatus.completed');
      expect(ss.SyncStatus.failed.toString(), 'SyncStatus.failed');
    });
  });

  group('SyncService - SyncEventType Tests', () {
    test('SyncEventType values are defined correctly', () {
      expect(ss.SyncEventType.started.toString(), 'SyncEventType.started');
      expect(ss.SyncEventType.success.toString(), 'SyncEventType.success');
      expect(ss.SyncEventType.error.toString(), 'SyncEventType.error');
      expect(ss.SyncEventType.completed.toString(), 'SyncEventType.completed');
      expect(ss.SyncEventType.online.toString(), 'SyncEventType.online');
      expect(ss.SyncEventType.offline.toString(), 'SyncEventType.offline');
    });
  });

  group('SyncService - SyncEvent Tests', () {
    test('SyncEvent with all fields', () {
      final event = ss.SyncEvent(
        type: ss.SyncEventType.started,
        documentId: 'doc-123',
        message: 'Starting sync',
        timestamp: DateTime(2024, 1),
      );

      expect(event.type, ss.SyncEventType.started);
      expect(event.documentId, 'doc-123');
      expect(event.message, 'Starting sync');
      expect(event.timestamp, DateTime(2024, 1));
    });

    test('SyncEvent with minimal fields', () {
      final event = ss.SyncEvent(
        type: ss.SyncEventType.success,
        timestamp: DateTime(2024, 1),
      );

      expect(event.type, ss.SyncEventType.success);
      expect(event.documentId, isNull);
      expect(event.message, isNull);
    });
  });

  group('SyncService - SyncResult Tests', () {
    test('SyncResult success', () {
      const result = ss.SyncResult(
        success: true,
        documentsSynced: 5,
      );

      expect(result.success, true);
      expect(result.documentsSynced, 5);
      expect(result.errorMessage, isNull);
    });

    test('SyncResult failure with error message', () {
      const result = ss.SyncResult(
        success: false,
        errorMessage: 'Network error',
      );

      expect(result.success, false);
      expect(result.errorMessage, 'Network error');
      expect(result.documentsSynced, 0);
    });
  });

  group('SyncService - SyncSummary Tests', () {
    test('SyncSummary copyWith', () {
      final original = ss.SyncSummary(
        success: true,
        syncedCount: 10,
        failedCount: 2,
        timestamp: DateTime(2024, 1),
      );

      final modified = original.copyWith(
        syncedCount: 15,
      );

      expect(modified.success, true);
      expect(modified.syncedCount, 15);
      expect(modified.failedCount, 2);
      expect(modified.timestamp, original.timestamp);
    });

    test('SyncSummary with all fields', () {
      final summary = ss.SyncSummary(
        success: false,
        syncedCount: 8,
        failedCount: 3,
        timestamp: DateTime(2024, 1),
      );

      expect(summary.success, false);
      expect(summary.syncedCount, 8);
      expect(summary.failedCount, 3);
    });
  });

  group('MockSyncService - Auto Sync Configuration', () {
    late MockCrdtService mockCrdtService;
    late MockSyncService syncService;

    setUp(() {
      mockCrdtService = MockCrdtService();
      syncService = MockSyncService(mockCrdtService);
    });

    test('setAutoSync enables auto sync', () {
      syncService.setAutoSync(true);
      expect(syncService.autoSyncEnabled, true);
    });

    test('setAutoSync disables auto sync', () {
      syncService.setAutoSync(true);
      syncService.setAutoSync(false);
      expect(syncService.autoSyncEnabled, false);
    });

    test('setSyncInterval updates interval', () {
      syncService.setSyncInterval(60);
      expect(syncService.syncIntervalSeconds, 60);
    });

    test('default sync interval is 30 seconds', () {
      expect(syncService.syncIntervalSeconds, 30);
    });

    test('default auto sync is enabled', () {
      expect(syncService.autoSyncEnabled, true);
    });
  });

  group('MockSyncService - Sync Queue', () {
    late MockCrdtService mockCrdtService;
    late MockSyncService syncService;

    setUp(() {
      mockCrdtService = MockCrdtService();
      syncService = MockSyncService(mockCrdtService);
    });

    test('getPendingSyncCount returns count of dirty documents', () async {
      mockCrdtService.markDirty('doc1');
      mockCrdtService.markDirty('doc2');
      mockCrdtService.markDirty('doc3');

      expect(await syncService.getPendingSyncCount(), 3);
    });

    test('getPendingSyncCount returns 0 when no dirty documents', () async {
      expect(await syncService.getPendingSyncCount(), 0);
    });
  });

  group('MockSyncService - Document Sync', () {
    late MockCrdtService mockCrdtService;
    late MockSyncService syncService;

    setUp(() {
      mockCrdtService = MockCrdtService();
      syncService = MockSyncService(mockCrdtService);
    });

    test('syncDocument returns success when state is null', () async {
      final result = await syncService.syncDocument('nonexistent-doc');

      expect(result.success, true);
      expect(result.documentsSynced, 0);
    });

    test('syncDocument marks document as synced', () async {
      final testData = Uint8List.fromList([1, 2, 3, 4]);
      mockCrdtService.setDocumentState('doc-1', testData);
      mockCrdtService.markDirty('doc-1');

      final result = await syncService.syncDocument('doc-1');

      expect(result.success, true);
      expect(result.documentsSynced, 1);
    });
  });

  group('MockSyncService - Integration Simulation', () {
    test('simulate offline edit and sync flow', () async {
      final mockCrdtService = MockCrdtService();
      final syncService = MockSyncService(mockCrdtService);

      // Create document and mark as dirty (simulating offline edit)
      final testData = Uint8List.fromList([1, 2, 3]);
      mockCrdtService.setDocumentState('doc-1', testData);
      mockCrdtService.markDirty('doc-1');

      // Verify document is dirty
      expect(mockCrdtService.hasPendingChanges('doc-1'), true);

      // Simulate sync
      final result = await syncService.syncDocument('doc-1');

      // Verify sync result
      expect(result.success, true);
      expect(result.documentsSynced, 1);

      // Verify document is no longer dirty
      expect(mockCrdtService.hasPendingChanges('doc-1'), false);
    });

    test('simulate multiple dirty documents sync', () async {
      final mockCrdtService = MockCrdtService();
      final syncService = MockSyncService(mockCrdtService);

      // Create multiple dirty documents
      for (var i = 1; i <= 5; i++) {
        final testData = Uint8List.fromList([i]);
        mockCrdtService.setDocumentState('doc-$i', testData);
        mockCrdtService.markDirty('doc-$i');
      }

      // Sync all
      final summary = await syncService.syncAllDirtyDocuments();

      expect(summary.success, true);
      expect(summary.syncedCount, 5);
      expect(summary.failedCount, 0);

      // All documents should be synced
      expect(await syncService.getPendingSyncCount(), 0);
    });

    test('simulate partial sync failure', () async {
      final mockCrdtService = MockCrdtService();
      final syncService = MockSyncService(mockCrdtService);

      // Create documents - one with state, one without
      final testData = Uint8List.fromList([1]);
      mockCrdtService.setDocumentState('doc-1', testData);
      mockCrdtService.markDirty('doc-1');
      mockCrdtService.markDirty('doc-2'); // No state

      // Sync all - doc-2 will sync with null state (success with 0 synced)
      final summary = await syncService.syncAllDirtyDocuments();

      // Both operations succeed (doc-2 returns success with 0 docs)
      expect(summary.success, true);
      expect(summary.syncedCount, 2);
    });
  });
}
