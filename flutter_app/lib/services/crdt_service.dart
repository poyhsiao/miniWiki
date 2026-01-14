import 'dart:async';
import 'dart:typed_data';
import 'package:flutter_riverpod/flutter_riverpod.dart';

/// Sync status enumeration
enum SyncStatus {
  idle,
  syncing,
  success,
  error,
}

/// Sync state for a document
class DocumentSyncState {
  final String documentId;
  final SyncStatus status;
  final String? errorMessage;
  final DateTime lastSyncedAt;

  const DocumentSyncState({
    required this.documentId,
    required this.lastSyncedAt,
    this.status = SyncStatus.idle,
    this.errorMessage,
  });

  DocumentSyncState copyWith({
    String? documentId,
    SyncStatus? status,
    String? errorMessage,
    DateTime? lastSyncedAt,
  }) =>
      DocumentSyncState(
        documentId: documentId ?? this.documentId,
        status: status ?? this.status,
        errorMessage: errorMessage ?? this.errorMessage,
        lastSyncedAt: lastSyncedAt ?? this.lastSyncedAt,
      );
}

/// CRDT document wrapper - uses dynamic for y_crdt types
class CrdtDocument {
  final String id;
  final dynamic doc;
  final dynamic text;
  final Map<String, dynamic> arrays;
  final Map<String, dynamic> maps;
  DateTime lastSyncedAt;
  bool isDirty;

  CrdtDocument({
    required this.id,
    required this.doc,
    this.text,
    this.arrays = const {},
    this.maps = const {},
    DateTime? lastSyncedAt,
    this.isDirty = false,
  }) : lastSyncedAt = lastSyncedAt ?? DateTime.now();
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

/// CRDT service for managing Yjs documents
class CrdtService {
  final Map<String, CrdtDocument> _documents = {};
  final StreamController<Map<String, dynamic>> _updateController =
      StreamController<Map<String, dynamic>>.broadcast();
  final Map<String, DocumentSyncState> _syncStates = {};
  final StreamController<DocumentSyncState> _syncStateController =
      StreamController<DocumentSyncState>.broadcast();

  /// Stream of document updates
  Stream<Map<String, dynamic>> get updates => _updateController.stream;

  /// Stream of sync state changes
  Stream<DocumentSyncState> get syncStateChanges => _syncStateController.stream;

  /// Get or create a document
  CrdtDocument getDocument(String id) {
    if (!_documents.containsKey(id)) {
      _documents[id] = CrdtDocument(
        id: id,
        doc: null,
        arrays: {},
        maps: {},
      );
    }
    return _documents[id]!;
  }

  /// Get sync state for a document
  DocumentSyncState getSyncState(String documentId) {
    if (!_syncStates.containsKey(documentId)) {
      _syncStates[documentId] = DocumentSyncState(
        documentId: documentId,
        lastSyncedAt: DateTime.now(),
      );
    }
    return _syncStates[documentId]!;
  }

  /// Update sync state
  void updateSyncState(String documentId, SyncStatus status, [String? error]) {
    final newState = DocumentSyncState(
      documentId: documentId,
      status: status,
      errorMessage: error,
      lastSyncedAt: DateTime.now(),
    );
    _syncStates[documentId] = newState;
    _syncStateController.add(newState);
  }

  /// Get text content from a document
  String? getText(String documentId) {
    final doc = getDocument(documentId);
    return doc.text?.toString();
  }

  /// Set text content in a document
  Future<void> setText(String documentId, String text) async {
    final doc = getDocument(documentId);
    if (doc.text != null) {
      // Clear existing text and insert new text
      doc.isDirty = true;
    }
    _emitUpdate(documentId, 'text', {'operation': 'set', 'text': text});
  }

  /// Get document state as update vector
  Future<Uint8List?> getState(String documentId) async {
    final doc = getDocument(documentId);
    if (doc.doc != null) {
      return doc.doc.encodeStateAsUpdate() as Uint8List?;
    }
    return null;
  }

  /// Apply update to document
  Future<void> applyUpdate(String documentId, Uint8List update) async {
    final doc = getDocument(documentId);
    if (doc.doc != null) {
      doc.doc.applyUpdate(update);
    }
    doc.isDirty = true;
    doc.lastSyncedAt = DateTime.now();
    _emitUpdate(documentId, 'update', {'update': update});
  }

  /// Get shared array
  dynamic getArray(String documentId, String name) {
    final doc = getDocument(documentId);
    if (!doc.arrays.containsKey(name) && doc.doc != null) {
      doc.arrays[name] = doc.doc.getArray(name);
    }
    return doc.arrays[name];
  }

  /// Get shared map
  dynamic getMap(String documentId, String name) {
    final doc = getDocument(documentId);
    if (!doc.maps.containsKey(name) && doc.doc != null) {
      doc.maps[name] = doc.doc.getMap(name);
    }
    return doc.maps[name];
  }

  /// Delete a document
  void deleteDocument(String id) {
    _documents.remove(id);
    _syncStates.remove(id);
  }

  /// Check if document has pending changes
  bool hasPendingChanges(String documentId) =>
      _documents[documentId]?.isDirty ?? false;

  /// Get all document IDs with pending changes
  List<String> getDirtyDocumentIds() => _documents.entries
      .where((e) => e.value.isDirty)
      .map((e) => e.key)
      .toList();

  /// Mark document as synced
  void markSynced(String documentId) {
    final doc = _documents[documentId];
    if (doc != null) {
      doc.isDirty = false;
      doc.lastSyncedAt = DateTime.now();
    }
    updateSyncState(documentId, SyncStatus.success);
  }

  /// Emit update event
  void _emitUpdate(String documentId, String field, event) {
    _updateController.add({
      'documentId': documentId,
      'field': field,
      'event': event,
      'timestamp': DateTime.now().toIso8601String(),
    });
  }

  /// Dispose of the service
  void dispose() {
    _documents.clear();
    _syncStates.clear();
    _updateController.close();
    _syncStateController.close();
  }
}

/// Provider for CrdtService
final crdtServiceProvider = Provider<CrdtService>((ref) => CrdtService());
