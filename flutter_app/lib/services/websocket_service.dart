import 'dart:async';
import 'dart:convert';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:web_socket_channel/web_socket_channel.dart';

final websocketServiceProvider = Provider<WebSocketService>((ref) => WebSocketService());

class WebSocketService {
  WebSocketChannel? _channel;
  final _messageController = StreamController<dynamic>.broadcast();
  final _connectionStateController = StreamController<ConnectionState>.broadcast();
  final _presenceController = StreamController<List<ActiveUser>>.broadcast();

  Stream<dynamic> get messages => _messageController.stream;
  Stream<ConnectionState> get connectionState => _connectionStateController.stream;
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
      final wsUrl = Uri.parse('$serverUrl/ws/documents/$documentId?user_id=$userId');

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
    } catch (e) {
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
      'payload': {},
      'timestamp': DateTime.now().toIso8601String(),
    };

    _channel!.sink.add(jsonEncode(message));
  }

  void _handleMessage(data) {
    try {
      final json = jsonDecode(data as String) as Map<String, dynamic>;
      final type = json['type'] as String;

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
    } catch (e) {
      _messageController.addError(WebSocketException('Failed to parse message: $e'));
    }
  }

  void _handleSyncMessage(Map<String, dynamic> json) {
    final payload = json['payload'] as Map<String, dynamic>;
    if (payload['update'] != null) {
      base64Decode(payload['update'] as String);
    }
    if (payload['state_vector'] != null) {
      base64Decode(payload['state_vector'] as String);
    }
  }

  void _handleAwarenessMessage(Map<String, dynamic> json) {
  }

  void _handleCursorMessage(Map<String, dynamic> json) {
  }

  void _handleDocumentUpdate(Map<String, dynamic> json) {
    final payload = json['payload'] as Map<String, dynamic>;
    if (payload['update'] != null) {
      base64Decode(payload['update'] as String);
    }
  }

  void _handleUserJoin(Map<String, dynamic> json) {
  }

  void _handleUserLeave(Map<String, dynamic> json) {
    json['payload'] as Map<String, dynamic>;
  }

  void _handleError(error) {
    _updateConnectionState(ConnectionState.error);
    _messageController.addError(error as Object);
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
      'payload': {},
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
      'payload': {},
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
    required this.lastActive, this.cursor,
  });

  ActiveUser copyWith({
    String? userId,
    String? displayName,
    String? color,
    CursorPosition? cursor,
    DateTime? lastActive,
  }) => ActiveUser(
      userId: userId ?? this.userId,
      displayName: displayName ?? this.displayName,
      color: color ?? this.color,
      cursor: cursor ?? this.cursor,
      lastActive: lastActive ?? this.lastActive,
    );

  factory ActiveUser.fromJson(Map<String, dynamic> json) => ActiveUser(
      userId: json['user_id'] as String,
      displayName: json['display_name'] as String,
      color: json['color'] as String,
      cursor: json['cursor'] != null
          ? CursorPosition(
              x: (json['cursor']['x'] as num).toDouble(),
              y: (json['cursor']['y'] as num).toDouble(),
              selectionStart: json['cursor']['selection_start'] as int?,
              selectionEnd: json['cursor']['selection_end'] as int?,
            )
          : null,
      lastActive: DateTime.parse(json['last_active'] as String),
    );
}

class WebSocketException implements Exception {
  final String message;

  WebSocketException(this.message);

  @override
  String toString() => 'WebSocketException: $message';
}
