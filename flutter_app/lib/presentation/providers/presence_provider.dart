import 'dart:async';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:miniwiki/services/websocket_service.dart';

final presenceProvider = StateNotifierProvider<PresenceNotifier, PresenceState>((ref) {
  final wsService = ref.watch(websocketServiceProvider);
  return PresenceNotifier(wsService);
});

class PresenceState {
  final List<ActiveUser> activeUsers;
  final Map<String, CursorPosition> remoteCursors;
  final bool isLoading;
  final String? error;

  PresenceState({
    this.activeUsers = const [],
    this.remoteCursors = const {},
    this.isLoading = false,
    this.error,
  });

  PresenceState copyWith({
    List<ActiveUser>? activeUsers,
    Map<String, CursorPosition>? remoteCursors,
    bool? isLoading,
    String? error,
  }) => PresenceState(
      activeUsers: activeUsers ?? this.activeUsers,
      remoteCursors: remoteCursors ?? this.remoteCursors,
      isLoading: isLoading ?? this.isLoading,
      error: error ?? this.error,
    );
}

class PresenceNotifier extends StateNotifier<PresenceState> {
  final WebSocketService _wsService;
  StreamSubscription? _presenceSubscription;
  StreamSubscription? _messageSubscription;

  PresenceNotifier(this._wsService) : super(PresenceState()) {
    _setupPresenceListener();
  }

  void _setupPresenceListener() {
    _presenceSubscription = _wsService.presence.listen(
      (users) {
        state = state.copyWith(
          activeUsers: users,
          isLoading: false,
        );
      },
      onError: (error) {
        state = state.copyWith(
          error: error.toString(),
          isLoading: false,
        );
      },
    );
  }

  Future<void> connectToDocument(String documentId, String userId) async {
    state = state.copyWith(isLoading: true);

    try {
      await _wsService.connect(
        documentId: documentId,
        userId: userId,
      );

      _messageSubscription = _wsService.messages.listen(_handleWsMessage);
    } catch (e) {
      state = state.copyWith(
        error: 'Failed to connect: $e',
        isLoading: false,
      );
    }
  }

  Future<void> disconnectFromDocument() async {
    await _wsService.disconnect();
    _messageSubscription?.cancel();
    state = PresenceState();
  }

  void _handleWsMessage(message) {
    if (message is! Map<String, dynamic>) return;

    final type = message['type'] as String?;
    if (type == null) return;

    switch (type) {
      case 'Awareness':
        _handleAwarenessUpdate(message);
        break;
      case 'Cursor':
        _handleCursorUpdate(message);
        break;
      case 'UserJoin':
        _handleUserJoin(message);
        break;
      case 'UserLeave':
        _handleUserLeave(message);
        break;
    }
  }

  void _handleAwarenessUpdate(Map<String, dynamic> message) {
    message['payload'] as Map<String, dynamic>;
    final userId = message['user_id'] as String;

    state = state.copyWith(
      activeUsers: state.activeUsers.map((user) {
        if (user.userId == userId) {
          return user.copyWith();
        }
        return user;
      }).toList(),
    );
  }

  void _handleCursorUpdate(Map<String, dynamic> message) {
    final payload = message['payload'] as Map<String, dynamic>;
    final userId = message['user_id'] as String;

    final cursor = CursorPosition(
      x: (payload['x'] as num).toDouble(),
      y: (payload['y'] as num).toDouble(),
      selectionStart: payload['selection_start'] as int?,
      selectionEnd: payload['selection_end'] as int?,
    );

    final newCursors = Map<String, CursorPosition>.from(state.remoteCursors);
    newCursors[userId] = cursor;

    state = state.copyWith(remoteCursors: newCursors);
  }

  void _handleUserJoin(Map<String, dynamic> message) {
    final payload = message['payload'] as Map<String, dynamic>;
    final userId = message['user_id'] as String;
    final displayName = payload['display_name'] as String? ?? 'Anonymous';
    final color = payload['color'] as String? ?? '#3B82F6';

    final newUser = ActiveUser(
      userId: userId,
      displayName: displayName,
      color: color,
      lastActive: DateTime.now(),
    );

    final newUsers = List<ActiveUser>.from(state.activeUsers);
    if (!newUsers.any((u) => u.userId == userId)) {
      newUsers.add(newUser);
    }

    state = state.copyWith(activeUsers: newUsers);
  }

  void _handleUserLeave(Map<String, dynamic> message) {
    final userId = message['user_id'] as String;

    final newUsers = state.activeUsers.where((u) => u.userId != userId).toList();
    final newCursors = Map<String, CursorPosition>.from(state.remoteCursors)..remove(userId);

    state = state.copyWith(
      activeUsers: newUsers,
      remoteCursors: newCursors,
    );
  }

  Future<void> updateMyPresence(Map<String, dynamic> awarenessState) async {
    try {
      await _wsService.sendAwarenessUpdate(awarenessState);
    } catch (e) {
      state = state.copyWith(error: 'Failed to update presence: $e');
    }
  }

  Future<void> updateMyCursor(CursorPosition position) async {
    try {
      await _wsService.sendCursorUpdate(position);
    } catch (e) {
      state = state.copyWith(error: 'Failed to update cursor: $e');
    }
  }

  ActiveUser? getUser(String userId) {
    try {
      return state.activeUsers.firstWhere((u) => u.userId == userId);
    } catch (e) {
      return null;
    }
  }

  CursorPosition? getCursor(String userId) => state.remoteCursors[userId];

  @override
  void dispose() {
    _presenceSubscription?.cancel();
    _messageSubscription?.cancel();
    super.dispose();
  }
}
