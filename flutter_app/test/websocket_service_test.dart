// T153: WebSocket service tests for real-time collaboration
// Testing data structures and mock implementations for real-time collaboration
// Run with: flutter test test/websocket_service_test.dart

import 'dart:async';
import 'dart:typed_data';
import 'package:flutter_test/flutter_test.dart';

// ============================================================================
// Mock Services - Standalone implementations for testing
// ============================================================================

/// Mock CrdtService for testing WebSocket sync operations
class MockCrdtService {
  final Map<String, Uint8List> _documentStates = {};
  final Map<String, Uint8List> _stateVectors = {};
  final List<Uint8List> _pendingUpdates = [];

  Future<Uint8List?> getDocumentState(String documentId) async {
    return _documentStates[documentId];
  }

  Future<void> setDocumentState(String documentId, Uint8List state) async {
    _documentStates[documentId] = state;
  }

  Future<Uint8List?> getStateVector(String documentId) async {
    return _stateVectors[documentId];
  }

  Future<void> setStateVector(String documentId, Uint8List vector) async {
    _stateVectors[documentId] = vector;
  }

  Future<void> applyUpdate(String documentId, Uint8List update) async {
    _pendingUpdates.add(update);
  }

  Future<Uint8List> diffUpdate(String documentId, Uint8List stateVector) async {
    return Uint8List.fromList([1, 2, 3, 4]);
  }

  Future<void> initializeDocument(String documentId) async {
    if (!_documentStates.containsKey(documentId)) {
      _documentStates[documentId] = Uint8List(0);
      _stateVectors[documentId] = Uint8List(0);
    }
  }

  List<Uint8List> getPendingUpdates() => List.from(_pendingUpdates);
  void clearPendingUpdates() => _pendingUpdates.clear();
}

/// Mock PresenceService for testing user presence tracking
class MockPresenceService {
  final Map<String, PresenceUser> _activeUsers = {};
  final Map<String, Map<String, CursorPosition>> _userCursors = {};
  final List<PresenceEvent> _events = [];
  final _presenceController = StreamController<PresenceEvent>.broadcast();

  Stream<PresenceEvent> get onPresenceChange => _presenceController.stream;

  Future<void> joinDocument(String documentId, String userId, String displayName, String color) async {
    final user = PresenceUser(
      userId: userId,
      displayName: displayName,
      color: color,
      joinedAt: DateTime.now(),
    );
    _activeUsers['$documentId:$userId'] = user;
    _userCursors['$documentId:$userId'] = {};
    _events.add(PresenceEvent(
      type: PresenceEventType.joined,
      documentId: documentId,
      userId: userId,
      timestamp: DateTime.now(),
    ));
    _presenceController.add(PresenceEvent(
      type: PresenceEventType.joined,
      documentId: documentId,
      userId: userId,
      timestamp: DateTime.now(),
    ));
  }

  Future<void> leaveDocument(String documentId, String userId) async {
    _activeUsers.remove('$documentId:$userId');
    _userCursors.remove('$documentId:$userId');
    _events.add(PresenceEvent(
      type: PresenceEventType.left,
      documentId: documentId,
      userId: userId,
      timestamp: DateTime.now(),
    ));
    _presenceController.add(PresenceEvent(
      type: PresenceEventType.left,
      documentId: documentId,
      userId: userId,
      timestamp: DateTime.now(),
    ));
  }

  Future<void> updateCursor(String documentId, String userId, CursorPosition position) async {
    if (_userCursors.containsKey('$documentId:$userId')) {
      _userCursors['$documentId:$userId']!['cursor'] = position;
    }
  }

  Future<List<PresenceUser>> getActiveUsers(String documentId) async {
    return _activeUsers.entries
        .where((e) => e.key.startsWith('$documentId:'))
        .map((e) => e.value)
        .toList();
  }

  Future<CursorPosition?> getCursor(String documentId, String userId) async {
    return _userCursors['$documentId:$userId']?['cursor'];
  }

  List<PresenceEvent> getEvents() => List.from(_events);
  void clearEvents() => _events.clear();
}

// ============================================================================
// WebSocket Service Tests
// ============================================================================

void main() {
  group('WebSocketService - Connection State', () {
    test('ConnectionState values are defined correctly', () {
      expect(WebSocketConnectionState.disconnected.toString(), 'WebSocketConnectionState.disconnected');
      expect(WebSocketConnectionState.connecting.toString(), 'WebSocketConnectionState.connecting');
      expect(WebSocketConnectionState.connected.toString(), 'WebSocketConnectionState.connected');
      expect(WebSocketConnectionState.reconnecting.toString(), 'WebSocketConnectionState.reconnecting');
      expect(WebSocketConnectionState.error.toString(), 'WebSocketConnectionState.error');
    });

    test('ConnectionState can be compared', () {
      expect(WebSocketConnectionState.disconnected, WebSocketConnectionState.disconnected);
      expect(WebSocketConnectionState.connected, isNot(WebSocketConnectionState.disconnected));
    });
  });

  group('WebSocketService - Message Types', () {
    test('WebSocketMessageType values are defined correctly', () {
      expect(WebSocketMessageType.sync.toString(), 'WebSocketMessageType.sync');
      expect(WebSocketMessageType.awareness.toString(), 'WebSocketMessageType.awareness');
      expect(WebSocketMessageType.cursor.toString(), 'WebSocketMessageType.cursor');
      expect(WebSocketMessageType.userJoin.toString(), 'WebSocketMessageType.userJoin');
      expect(WebSocketMessageType.userLeave.toString(), 'WebSocketMessageType.userLeave');
      expect(WebSocketMessageType.ping.toString(), 'WebSocketMessageType.ping');
      expect(WebSocketMessageType.pong.toString(), 'WebSocketMessageType.pong');
    });
  });

  group('WebSocketService - Sync Message', () {
    test('SyncMessage can be encoded and decoded', () {
      final message = SyncMessage(
        documentId: 'doc-123',
        stateVector: Uint8List.fromList([1, 0, 1, 0]),
        update: Uint8List.fromList([0, 1, 0, 1, 1, 0]),
      );

      final encoded = message.toJson();
      final decoded = SyncMessage.fromJson(encoded);

      expect(decoded.documentId, message.documentId);
      expect(decoded.stateVector, message.stateVector);
      expect(decoded.update, message.update);
    });

    test('SyncMessage handles null fields', () {
      final message = SyncMessage(
        documentId: 'doc-456',
        stateVector: null,
        update: null,
      );

      final encoded = message.toJson();
      final decoded = SyncMessage.fromJson(encoded);

      expect(decoded.documentId, message.documentId);
      expect(decoded.stateVector, isNull);
      expect(decoded.update, isNull);
    });
  });

  group('WebSocketService - Awareness Message', () {
    test('AwarenessMessage can be created with user info', () {
      final user = AwarenessUser(
        userId: 'user-123',
        displayName: 'John Doe',
        color: '#FF6B6B',
      );

      final message = AwarenessMessage(
        documentId: 'doc-123',
        user: user,
        cursor: CursorPosition(x: 100.5, y: 200.75),
      );

      expect(message.user.displayName, 'John Doe');
      expect(message.user.color, '#FF6B6B');
      expect(message.cursor?.x, 100.5);
      expect(message.cursor?.y, 200.75);
    });

    test('AwarenessMessage handles null cursor', () {
      final user = AwarenessUser(
        userId: 'user-456',
        displayName: 'Jane Doe',
        color: '#4ECDC4',
      );

      final message = AwarenessMessage(
        documentId: 'doc-456',
        user: user,
        cursor: null,
      );

      expect(message.cursor, isNull);
    });
  });

  group('WebSocketService - Cursor Position', () {
    test('CursorPosition stores coordinates correctly', () {
      final cursor = CursorPosition(
        x: 150.25,
        y: 300.5,
        selectionStart: 50,
        selectionEnd: 75,
      );

      expect(cursor.x, 150.25);
      expect(cursor.y, 300.5);
      expect(cursor.selectionStart, 50);
      expect(cursor.selectionEnd, 75);
    });

    test('CursorPosition handles null selection', () {
      final cursor = CursorPosition(x: 100.0, y: 200.0);

      expect(cursor.selectionStart, isNull);
      expect(cursor.selectionEnd, isNull);
    });
  });

  group('WebSocketService - Presence User', () {
    test('PresenceUser stores user information', () {
      final user = PresenceUser(
        userId: 'user-789',
        displayName: 'Test User',
        color: '#45B7D1',
        joinedAt: DateTime(2024, 1, 15, 10, 30),
      );

      expect(user.userId, 'user-789');
      expect(user.displayName, 'Test User');
      expect(user.color, '#45B7D1');
    });
  });

  group('WebSocketService - Presence Event', () {
    test('PresenceEvent with joined type', () {
      final event = PresenceEvent(
        type: PresenceEventType.joined,
        documentId: 'doc-123',
        userId: 'user-456',
        timestamp: DateTime.now(),
      );

      expect(event.type, PresenceEventType.joined);
      expect(event.documentId, 'doc-123');
      expect(event.userId, 'user-456');
    });

    test('PresenceEventType values are defined correctly', () {
      expect(PresenceEventType.joined.toString(), 'PresenceEventType.joined');
      expect(PresenceEventType.left.toString(), 'PresenceEventType.left');
      expect(PresenceEventType.cursorUpdate.toString(), 'PresenceEventType.cursorUpdate');
    });
  });

  group('WebSocketService - Connection Config', () {
    test('WebSocketConfig default values', () {
      final config = WebSocketConfig();

      expect(config.reconnectInterval, 3000);
      expect(config.maxReconnectAttempts, 5);
      expect(config.heartbeatInterval, 30000);
      expect(config.pingTimeout, 5000);
    });

    test('WebSocketConfig with custom values', () {
      final config = WebSocketConfig(
        reconnectInterval: 5000,
        maxReconnectAttempts: 10,
        heartbeatInterval: 60000,
        pingTimeout: 10000,
      );

      expect(config.reconnectInterval, 5000);
      expect(config.maxReconnectAttempts, 10);
      expect(config.heartbeatInterval, 60000);
      expect(config.pingTimeout, 10000);
    });
  });

  group('WebSocketService - Mock Integration', () {
    late MockCrdtService mockCrdtService;
    late MockPresenceService mockPresenceService;

    setUp(() {
      mockCrdtService = MockCrdtService();
      mockPresenceService = MockPresenceService();
    });

    test('MockCrdtService stores and retrieves document state', () async {
      final testData = Uint8List.fromList([1, 2, 3, 4, 5]);
      await mockCrdtService.setDocumentState('doc-1', testData);

      final retrieved = await mockCrdtService.getDocumentState('doc-1');
      expect(retrieved, testData);
    });

    test('MockCrdtService handles null state', () async {
      final state = await mockCrdtService.getDocumentState('nonexistent');
      expect(state, isNull);
    });

    test('MockCrdtService applies and tracks updates', () async {
      final update = Uint8List.fromList([10, 20, 30]);
      await mockCrdtService.applyUpdate('doc-1', update);

      final pending = mockCrdtService.getPendingUpdates();
      expect(pending.length, 1);
      expect(pending.first, update);
    });

    test('MockPresenceService tracks user joins and leaves', () async {
      await mockPresenceService.joinDocument('doc-1', 'user-1', 'User One', '#FF0000');

      final users = await mockPresenceService.getActiveUsers('doc-1');
      expect(users.length, 1);
      expect(users.first.displayName, 'User One');

      await mockPresenceService.leaveDocument('doc-1', 'user-1');

      final usersAfterLeave = await mockPresenceService.getActiveUsers('doc-1');
      expect(usersAfterLeave.length, 0);
    });

    test('MockPresenceService tracks cursor updates', () async {
      await mockPresenceService.joinDocument('doc-2', 'user-2', 'User Two', '#00FF00');

      final cursor = CursorPosition(x: 250.0, y: 350.0);
      await mockPresenceService.updateCursor('doc-2', 'user-2', cursor);

      final retrievedCursor = await mockPresenceService.getCursor('doc-2', 'user-2');
      expect(retrievedCursor?.x, 250.0);
      expect(retrievedCursor?.y, 350.0);
    });

    test('MockPresenceService emits presence events', () async {
      final events = <PresenceEvent>[];
      final subscription = mockPresenceService.onPresenceChange.listen((event) {
        events.add(event);
      });

      await mockPresenceService.joinDocument('doc-3', 'user-3', 'User Three', '#0000FF');
      await mockPresenceService.leaveDocument('doc-3', 'user-3');

      expect(events.length, 2);
      expect(events[0].type, PresenceEventType.joined);
      expect(events[1].type, PresenceEventType.left);

      await subscription.cancel();
    });
  });

  group('WebSocketService - End-to-End Simulation', () {
    test('simulate real-time collaboration flow', () async {
      final mockCrdtService = MockCrdtService();
      final mockPresenceService = MockPresenceService();

      // Initialize document
      await mockCrdtService.initializeDocument('doc-collab');

      // User 1 joins
      await mockPresenceService.joinDocument('doc-collab', 'user-1', 'Alice', '#FF6B6B');

      // User 1 makes an edit
      final editUpdate = Uint8List.fromList([1, 2, 3]);
      await mockCrdtService.applyUpdate('doc-collab', editUpdate);

      // User 2 joins
      await mockPresenceService.joinDocument('doc-collab', 'user-2', 'Bob', '#4ECDC4');

      // User 2 updates cursor position
      await mockPresenceService.updateCursor(
        'doc-collab',
        'user-2',
        CursorPosition(x: 100.0, y: 200.0),
      );

      // Verify both users are present
      final activeUsers = await mockPresenceService.getActiveUsers('doc-collab');
      expect(activeUsers.length, 2);
      expect(activeUsers.any((u) => u.displayName == 'Alice'), true);
      expect(activeUsers.any((u) => u.displayName == 'Bob'), true);

      // Verify pending updates
      final pendingUpdates = mockCrdtService.getPendingUpdates();
      expect(pendingUpdates.length, 1);
    });

    test('simulate user presence across multiple documents', () async {
      final mockPresenceService = MockPresenceService();

      // User in multiple documents
      await mockPresenceService.joinDocument('doc-A', 'user-multi', 'Multi User', '#FF0000');
      await mockPresenceService.joinDocument('doc-B', 'user-multi', 'Multi User', '#FF0000');

      // Get users in each document
      final usersInA = await mockPresenceService.getActiveUsers('doc-A');
      final usersInB = await mockPresenceService.getActiveUsers('doc-B');

      expect(usersInA.length, 1);
      expect(usersInB.length, 1);

      // User leaves one document
      await mockPresenceService.leaveDocument('doc-A', 'user-multi');

      final usersInAAfter = await mockPresenceService.getActiveUsers('doc-A');
      expect(usersInAAfter.length, 0);
      expect((await mockPresenceService.getActiveUsers('doc-B')).length, 1);
    });
  });
}

// ============================================================================
// Data Structures
// ============================================================================

enum WebSocketConnectionState {
  disconnected,
  connecting,
  connected,
  reconnecting,
  error,
}

enum WebSocketMessageType {
  sync,
  awareness,
  cursor,
  userJoin,
  userLeave,
  ping,
  pong,
}

class SyncMessage {
  final String documentId;
  final Uint8List? stateVector;
  final Uint8List? update;

  SyncMessage({
    required this.documentId,
    this.stateVector,
    this.update,
  });

  Map<String, dynamic> toJson() => {
        'documentId': documentId,
        'stateVector': stateVector?.toList(),
        'update': update?.toList(),
      };

  factory SyncMessage.fromJson(Map<String, dynamic> json) => SyncMessage(
        documentId: json['documentId'] as String,
        stateVector: (json['stateVector'] as List?)?.cast<int>().let((i) => Uint8List.fromList(i)),
        update: (json['update'] as List?)?.cast<int>().let((i) => Uint8List.fromList(i)),
      );
}

class AwarenessMessage {
  final String documentId;
  final AwarenessUser user;
  final CursorPosition? cursor;

  AwarenessMessage({
    required this.documentId,
    required this.user,
    this.cursor,
  });

  Map<String, dynamic> toJson() => {
        'documentId': documentId,
        'user': user.toJson(),
        'cursor': cursor?.toJson(),
      };

  factory AwarenessMessage.fromJson(Map<String, dynamic> json) => AwarenessMessage(
        documentId: json['documentId'] as String,
        user: AwarenessUser.fromJson(json['user'] as Map<String, dynamic>),
        cursor: json['cursor'] != null
            ? CursorPosition.fromJson(json['cursor'] as Map<String, dynamic>)
            : null,
      );
}

class AwarenessUser {
  final String userId;
  final String displayName;
  final String color;

  AwarenessUser({
    required this.userId,
    required this.displayName,
    required this.color,
  });

  Map<String, dynamic> toJson() => {
        'userId': userId,
        'displayName': displayName,
        'color': color,
      };

  factory AwarenessUser.fromJson(Map<String, dynamic> json) => AwarenessUser(
        userId: json['userId'] as String,
        displayName: json['displayName'] as String,
        color: json['color'] as String,
      );
}

class CursorPosition {
  final double x;
  final double y;
  final int? selectionStart;
  final int? selectionEnd;

  CursorPosition({
    required this.x,
    required this.y,
    this.selectionStart,
    this.selectionEnd,
  });

  Map<String, dynamic> toJson() => {
        'x': x,
        'y': y,
        'selectionStart': selectionStart,
        'selectionEnd': selectionEnd,
      };

  factory CursorPosition.fromJson(Map<String, dynamic> json) => CursorPosition(
        x: (json['x'] as num).toDouble(),
        y: (json['y'] as num).toDouble(),
        selectionStart: json['selectionStart'] as int?,
        selectionEnd: json['selectionEnd'] as int?,
      );
}

class PresenceUser {
  final String userId;
  final String displayName;
  final String color;
  final DateTime joinedAt;

  PresenceUser({
    required this.userId,
    required this.displayName,
    required this.color,
    required this.joinedAt,
  });
}

enum PresenceEventType {
  joined,
  left,
  cursorUpdate,
}

class PresenceEvent {
  final PresenceEventType type;
  final String documentId;
  final String userId;
  final DateTime timestamp;

  PresenceEvent({
    required this.type,
    required this.documentId,
    required this.userId,
    required this.timestamp,
  });
}

class WebSocketConfig {
  final int reconnectInterval;
  final int maxReconnectAttempts;
  final int heartbeatInterval;
  final int pingTimeout;

  WebSocketConfig({
    this.reconnectInterval = 3000,
    this.maxReconnectAttempts = 5,
    this.heartbeatInterval = 30000,
    this.pingTimeout = 5000,
  });
}

// Extension for null safety
extension LetExtension<T> on T? {
  R? let<R>(R Function(T) f) {
    if (this == null) return null;
    return f(this!);
  }
}
