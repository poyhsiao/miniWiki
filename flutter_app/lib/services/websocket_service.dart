import 'dart:async';
import 'dart:convert';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:web_socket_channel/web_socket_channel.dart';

final websocketServiceProvider = Provider<WebSocketService>((ref) {
  final service = WebSocketService();
  ref.onDispose(service.dispose);
  return service;
});

class WebSocketService {
  WebSocketChannel? _channel;
  final _messageController = StreamController<dynamic>.broadcast();
  final _connectionStateController =
      StreamController<ConnectionState>.broadcast();
  final _presenceController = StreamController<List<ActiveUser>>.broadcast();

  Stream<dynamic> get messages => _messageController.stream;
  Stream<ConnectionState> get connectionState =>
      _connectionStateController.stream;
  Stream<List<ActiveUser>> get presence => _presenceController.stream;

  ConnectionState _currentState = ConnectionState.disconnected;
  String? _currentDocumentId;
  String? _currentUserId;

  ConnectionState get currentState => _currentState;

  Future<void> connect({
    required String documentId,
    required String userId,
    String? authToken,
    String serverUrl = 'ws://localhost:8080',
  }) async {
    if (_channel != null) {
      await disconnect();
    }

    _currentDocumentId = documentId;
    _currentUserId = userId;

    _updateConnectionState(ConnectionState.connecting);

    try {
      final wsUrl =
          Uri.parse('$serverUrl/ws/documents/$documentId?user_id=$userId');

      if (authToken != null) {
        _channel = WebSocketChannel.connect(
          wsUrl,
          protocols: ['Bearer', authToken],
        );
      } else {
        _channel = WebSocketChannel.connect(wsUrl);
      }

      _channel!.stream.listen(
        _handleMessage,
        onError: _handleError,
        onDone: _handleDone,
        cancelOnError: false,
      );

      _updateConnectionState(ConnectionState.connected);

      unawaited(_sendJoinMessage());
    } on Object catch (e) {
      _updateConnectionState(ConnectionState.error);
      _messageController.addError(e);
    }
  }

  Future<void> disconnect() async {
    if (_channel != null) {
      await _sendLeaveMessage();
      await _channel!.sink.close();
      _channel = null;
    }

    _currentDocumentId = null;
    _currentUserId = null;

    _updateConnectionState(ConnectionState.disconnected);
  }

  void dispose() {
    _messageController.close();
    _connectionStateController.close();
    _presenceController.close();
    _channel?.sink.close();
  }

  Future<void> sendUpdate(List<int> update) async {
    if (_channel == null || _currentState != ConnectionState.connected) {
      throw WebSocketException('Not connected');
    }

    final message = {
      'type': 'DocumentUpdate',
      'document_id': _currentDocumentId,
      'user_id': _currentUserId,
      'payload': {
        'update': base64Encode(update),
      },
      'timestamp': DateTime.now().toIso8601String(),
    };

    _channel!.sink.add(jsonEncode(message));
  }

  Future<void> sendStateVector(List<int> stateVector) async {
    if (_channel == null || _currentState != ConnectionState.connected) {
      throw WebSocketException('Not connected');
    }

    final message = {
      'type': 'Sync',
      'document_id': _currentDocumentId,
      'user_id': _currentUserId,
      'payload': {
        'state_vector': base64Encode(stateVector),
      },
      'timestamp': DateTime.now().toIso8601String(),
    };

    _channel!.sink.add(jsonEncode(message));
  }

  Future<void> sendAwarenessUpdate(Map<String, dynamic> awarenessState) async {
    if (_channel == null || _currentState != ConnectionState.connected) {
      throw WebSocketException('Not connected');
    }

    final message = {
      'type': 'Awareness',
      'document_id': _currentDocumentId,
      'user_id': _currentUserId,
      'payload': awarenessState,
      'timestamp': DateTime.now().toIso8601String(),
    };

    _channel!.sink.add(jsonEncode(message));
  }

  Future<void> sendCursorUpdate(CursorPosition position) async {
    if (_channel == null || _currentState != ConnectionState.connected) {
      throw WebSocketException('Not connected');
    }

    final message = {
      'type': 'Cursor',
      'document_id': _currentDocumentId,
      'user_id': _currentUserId,
      'payload': position.toJson(),
      'timestamp': DateTime.now().toIso8601String(),
    };

    _channel!.sink.add(jsonEncode(message));
  }

  Future<void> ping() async {
    if (_channel == null || _currentState != ConnectionState.connected) {
      return;
    }

    final message = {
      'type': 'Ping',
      'document_id': _currentDocumentId,
      'user_id': _currentUserId,
      'payload': <String, dynamic>{},
      'timestamp': DateTime.now().toIso8601String(),
    };

    _channel!.sink.add(jsonEncode(message));
  }

  void _handleMessage(Object? data) {
    // Guard against null or non-String data
    if (data == null || data is! String) {
      // Silently ignore invalid message types to avoid noise in logs
      return;
    }

    try {
      final decoded = jsonDecode(data);
      if (decoded is! Map<String, dynamic>) {
        // Ignore non-Map payloads to avoid TypeError
        return;
      }
      final json = decoded;
      // Safely extract and validate message type
      final type = json['type'];
      if (type is! String) {
        // Log warning for malformed type field and skip processing
        return;
      }

      switch (type) {
        case 'Sync':
          _handleSyncMessage(json);
          break;
        case 'Awareness':
          _handleAwarenessMessage(json);
          break;
        case 'Cursor':
          _handleCursorMessage(json);
          break;
        case 'DocumentUpdate':
          _handleDocumentUpdate(json);
          break;
        case 'UserJoin':
          _handleUserJoin(json);
          break;
        case 'UserLeave':
          _handleUserLeave(json);
          break;
        case 'Pong':
          break;
      }

      _messageController.add(json);
    } on Object catch (e) {
      _messageController
          .addError(WebSocketException('Failed to parse message: $e'));
    }
  }

  void _handleSyncMessage(Map<String, dynamic> json) {
    final payload = json['payload'] as Map<String, dynamic>;
    if (payload['update'] != null) {
      // ignore: unused_local_variable
      final decodedUpdate = base64Decode(payload['update'] as String);
      // TODO(kimhsiao): Apply the decoded Yjs update to the local document
    }
    if (payload['state_vector'] != null) {
      // ignore: unused_local_variable
      final decodedStateVector =
          base64Decode(payload['state_vector'] as String);
      // TODO(kimhsiao): Process the state vector for sync negotiation
    }
  }

  void _handleAwarenessMessage(Map<String, dynamic> json) {}

  void _handleCursorMessage(Map<String, dynamic> json) {}

  void _handleDocumentUpdate(Map<String, dynamic> json) {
    final payload = json['payload'] as Map<String, dynamic>;
    if (payload['update'] != null) {
      // ignore: unused_local_variable
      final decoded = base64Decode(payload['update'] as String);
      // TODO(kimhsiao): Apply the decoded Yjs update to the local document
      // This will be implemented when integrating with the CRDT service
    }
  }

  void _handleUserJoin(Map<String, dynamic> json) {}

  void _handleUserLeave(Map<String, dynamic> json) {
    // final payload = json['payload'] as Map<String, dynamic>;
    // TODO(kimhsiao): Process user leave payload if needed
  }

  void _handleError(Object error) {
    _updateConnectionState(ConnectionState.error);
    _messageController.addError(error);
  }

  void _handleDone() {
    _updateConnectionState(ConnectionState.disconnected);
  }

  void _updateConnectionState(ConnectionState state) {
    _currentState = state;
    _connectionStateController.add(state);
  }

  Future<void> _sendJoinMessage() async {
    if (_channel == null) return;

    final message = {
      'type': 'UserJoin',
      'document_id': _currentDocumentId,
      'user_id': _currentUserId,
      'payload': <String, dynamic>{},
      'timestamp': DateTime.now().toIso8601String(),
    };

    _channel!.sink.add(jsonEncode(message));
  }

  Future<void> _sendLeaveMessage() async {
    if (_channel == null) return;

    final message = {
      'type': 'UserLeave',
      'document_id': _currentDocumentId,
      'user_id': _currentUserId,
      'payload': <String, dynamic>{},
      'timestamp': DateTime.now().toIso8601String(),
    };

    _channel!.sink.add(jsonEncode(message));
  }
}

enum ConnectionState {
  disconnected,
  connecting,
  connected,
  error,
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
        'selection_start': selectionStart,
        'selection_end': selectionEnd,
      };
}

class ActiveUser {
  final String userId;
  final String displayName;
  final String color;
  final CursorPosition? cursor;
  final DateTime lastActive;

  ActiveUser({
    required this.userId,
    required this.displayName,
    required this.color,
    required this.lastActive,
    this.cursor,
  });

  ActiveUser copyWith({
    String? userId,
    String? displayName,
    String? color,
    CursorPosition? cursor,
    DateTime? lastActive,
  }) =>
      ActiveUser(
        userId: userId ?? this.userId,
        displayName: displayName ?? this.displayName,
        color: color ?? this.color,
        cursor: cursor ?? this.cursor,
        lastActive: lastActive ?? this.lastActive,
      );

  factory ActiveUser.fromJson(Map<String, dynamic> json) {
    // Validate required String fields
    if (json['user_id'] is! String) {
      throw ArgumentError('user_id must be a String');
    }
    if (json['display_name'] is! String) {
      throw ArgumentError('display_name must be a String');
    }
    if (json['color'] is! String) {
      throw ArgumentError('color must be a String');
    }
    if (json['last_active'] is! String) {
      throw ArgumentError('last_active must be a String');
    }

    // Safely parse cursor with runtime type check
    CursorPosition? cursor;
    if (json['cursor'] != null) {
      try {
        if (json['cursor'] is Map<String, dynamic>) {
          final cursorMap = json['cursor'] as Map<String, dynamic>;
          // Validate required numeric fields before constructing
          if (cursorMap['x'] is num && cursorMap['y'] is num) {
            cursor = CursorPosition(
              x: (cursorMap['x'] as num).toDouble(),
              y: (cursorMap['y'] as num).toDouble(),
              selectionStart: (cursorMap['selection_start'] as num?)?.toInt(),
              selectionEnd: (cursorMap['selection_end'] as num?)?.toInt(),
            );
          }
        } else if (json['cursor'] is Map) {
          final cursorMap = (json['cursor'] as Map).cast<String, dynamic>();
          if (cursorMap['x'] is num && cursorMap['y'] is num) {
            cursor = CursorPosition(
              x: (cursorMap['x'] as num).toDouble(),
              y: (cursorMap['y'] as num).toDouble(),
              selectionStart: (cursorMap['selection_start'] as num?)?.toInt(),
              selectionEnd: (cursorMap['selection_end'] as num?)?.toInt(),
            );
          }
        }
        // If not a Map or missing required fields, cursor remains null
      } catch (_) {
        // Malformed cursor data, treat as no cursor
        cursor = null;
      }
    }

    // Safely parse lastActive with tryParse
    DateTime? lastActive;
    final lastActiveStr = json['last_active'] as String?;
    if (lastActiveStr != null) {
      lastActive = DateTime.tryParse(lastActiveStr);
      if (lastActive == null) {
        throw FormatException(
            'Invalid last_active format for user ${json['user_id']}: $lastActiveStr');
      }
    }

    return ActiveUser(
      userId: json['user_id'] as String,
      displayName: json['display_name'] as String,
      color: json['color'] as String,
      cursor: cursor,
      lastActive: lastActive!,
    );
  }
}

class WebSocketException implements Exception {
  final String message;

  WebSocketException(this.message);

  @override
  String toString() => 'WebSocketException: $message';
}
