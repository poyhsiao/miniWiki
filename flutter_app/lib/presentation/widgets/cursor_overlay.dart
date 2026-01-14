import 'package:flutter/material.dart';
import 'package:flutter/foundation.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:miniwiki/presentation/providers/presence_provider.dart';
import 'package:miniwiki/services/websocket_service.dart';

/// Widget that overlays remote users' cursors on the document editor
class CursorOverlay extends ConsumerWidget {
  final Widget child;
  final Function(CursorPosition)? onCursorMoved;

  const CursorOverlay({
    required this.child,
    super.key,
    this.onCursorMoved,
  });

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final presenceState = ref.watch(presenceProvider);
    final presenceNotifier = ref.read(presenceProvider.notifier);

    if (presenceState.remoteCursors.isEmpty) {
      return child;
    }

    return Stack(
      children: [
        child,
        ...presenceState.remoteCursors.entries.map((entry) {
          final userId = entry.key;
          final cursor = entry.value;
          final user = presenceNotifier.getUser(userId);

          if (user == null) {
            return const SizedBox.shrink();
          }

          return _RemoteCursorWidget(
            user: user,
            cursor: cursor,
            onCursorMoved: (position) {
              onCursorMoved?.call(position);
            },
          );
        }),
      ],
    );
  }

  @override
  void debugFillProperties(DiagnosticPropertiesBuilder properties) {
    super.debugFillProperties(properties);
    properties.add(ObjectFlagProperty<Function(CursorPosition)?>.has(
        'onCursorMoved', onCursorMoved));
  }
}

/// Widget that displays a single remote user's cursor
class _RemoteCursorWidget extends StatelessWidget {
  final ActiveUser user;
  final CursorPosition cursor;
  final Function(CursorPosition)? onCursorMoved;

  const _RemoteCursorWidget({
    required this.user,
    required this.cursor,
    this.onCursorMoved,
  });

  @override
  Widget build(BuildContext context) => Positioned(
        left: cursor.x,
        top: cursor.y,
        child: MouseRegion(
          onHover: (event) {
            // Report local cursor position when hovering over this user's cursor
            onCursorMoved?.call(CursorPosition(
              x: event.localPosition.dx,
              y: event.localPosition.dy,
            ));
          },
          child: Column(
            mainAxisSize: MainAxisSize.min,
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              // Cursor indicator line
              Container(
                width: 2,
                height: 20,
                decoration: BoxDecoration(
                  color: _parseColor(user.color),
                  borderRadius: BorderRadius.circular(1),
                ),
              ),
              const SizedBox(height: 2),
              // User label
              Container(
                padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
                decoration: BoxDecoration(
                  color: _parseColor(user.color),
                  borderRadius: BorderRadius.circular(4),
                ),
                child: Text(
                  user.displayName,
                  style: TextStyle(
                    color: Colors.white,
                    fontSize: 12,
                    fontWeight: FontWeight.w500,
                  ),
                ),
              ),
            ],
          ),
        ),
      );

  @override
  void debugFillProperties(DiagnosticPropertiesBuilder properties) {
    super.debugFillProperties(properties);
    properties.add(DiagnosticsProperty<ActiveUser>('user', user));
    properties.add(DiagnosticsProperty<CursorPosition>('cursor', cursor));
    properties.add(ObjectFlagProperty<Function(CursorPosition)?>.has(
        'onCursorMoved', onCursorMoved));
  }

  Color _parseColor(String color) {
    try {
      final hexColor = color.replaceAll('#', '');
      final colorInt =
          int.parse(hexColor.length == 6 ? '0xFF$hexColor' : hexColor);
      return Color(colorInt);
    } catch (_) {
      return Colors.blue;
    }
  }
}

/// Widget that displays a list of active users with their colors
class ActiveUsersIndicator extends ConsumerWidget {
  const ActiveUsersIndicator({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final presenceState = ref.watch(presenceProvider);

    if (presenceState.activeUsers.isEmpty) {
      return const SizedBox.shrink();
    }

    return Container(
      padding: const EdgeInsets.all(8),
      decoration: BoxDecoration(
        color: Theme.of(context).colorScheme.surface.withOpacity(0.9),
        borderRadius: BorderRadius.circular(8),
        boxShadow: [
          BoxShadow(
            color: Colors.black.withOpacity(0.1),
            blurRadius: 4,
            offset: const Offset(0, 2),
          ),
        ],
      ),
      child: Column(
        mainAxisSize: MainAxisSize.min,
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text(
            'Active Users',
            style: Theme.of(context).textTheme.labelSmall?.copyWith(
                  fontWeight: FontWeight.bold,
                ),
          ),
          const SizedBox(height: 8),
          ...presenceState.activeUsers.map((user) => Padding(
                padding: const EdgeInsets.only(bottom: 4),
                child: Row(
                  children: [
                    // Color indicator
                    Container(
                      width: 12,
                      height: 12,
                      decoration: BoxDecoration(
                        color: _parseColor(user.color),
                        shape: BoxShape.circle,
                      ),
                    ),
                    const SizedBox(width: 8),
                    // User name
                    Expanded(
                      child: Text(
                        user.displayName,
                        style: Theme.of(context).textTheme.bodySmall,
                        overflow: TextOverflow.ellipsis,
                      ),
                    ),
                    // Last active time
                    Text(
                      _formatTimeAgo(user.lastActive),
                      style: Theme.of(context).textTheme.bodySmall?.copyWith(
                            color: Colors.grey,
                          ),
                    ),
                  ],
                ),
              )),
        ],
      ),
    );
  }

  String _formatTimeAgo(DateTime dateTime) {
    final difference = DateTime.now().difference(dateTime);

    if (difference.inSeconds < 60) return 'now';
    if (difference.inMinutes < 60) return '${difference.inMinutes}m';
    if (difference.inHours < 24) return '${difference.inHours}h';
    return '${difference.inDays}d';
  }

  Color _parseColor(String color) {
    try {
      final hexColor = color.replaceAll('#', '');
      final colorInt =
          int.parse(hexColor.length == 6 ? '0xFF$hexColor' : hexColor);
      return Color(colorInt);
    } catch (_) {
      return Colors.blue;
    }
  }
}
