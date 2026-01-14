import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:miniwiki/presentation/providers/sync_provider.dart';
import 'package:miniwiki/services/sync_service.dart' as ss;

/// Sync status indicator widget
/// Displays current sync status with visual feedback
class SyncStatusIndicator extends ConsumerWidget {
  /// Whether to show detailed status
  final bool showDetails;

  /// Icon size
  final double iconSize;

  /// Custom color for online state
  final Color? onlineColor;

  /// Custom color for offline state
  final Color? offlineColor;

  /// Custom color for syncing state
  final Color? syncingColor;

  /// Custom color for error state
  final Color? errorColor;

  const SyncStatusIndicator({
    super.key,
    this.showDetails = false,
    this.iconSize = 20,
    this.onlineColor,
    this.offlineColor,
    this.syncingColor,
    this.errorColor,
  });

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final syncState = ref.watch(syncStateProvider);

    return Row(
      mainAxisSize: MainAxisSize.min,
      children: [
        _buildStatusIcon(syncState.status, syncState.isOnline),
        if (showDetails) ...[
          const SizedBox(width: 8),
          _buildStatusText(context, syncState),
        ],
      ],
    );
  }

  Widget _buildStatusIcon(ss.SyncStatus status, bool isOnline) {
    Color iconColor;
    IconData iconData;
    Widget iconWidget;

    switch (status) {
      case ss.SyncStatus.syncing:
        iconColor = syncingColor ?? Colors.blue;
        iconData = Icons.sync;
        iconWidget = TweenAnimationBuilder<double>(
          tween: Tween(begin: 0, end: 1),
          duration: const Duration(seconds: 1),
          builder: (context, value, child) => Transform.rotate(
              angle: value * 2 * 3.14159,
              child: child,
            ),
          child: Icon(iconData, size: iconSize, color: iconColor),
        );
        break;

      case ss.SyncStatus.failed:
        iconColor = errorColor ?? Colors.red;
        iconData = Icons.sync_problem;
        iconWidget = Icon(iconData, size: iconSize, color: iconColor);
        break;

      case ss.SyncStatus.completed:
        if (isOnline) {
          iconColor = onlineColor ?? Colors.green;
          iconData = Icons.cloud_done;
        } else {
          iconColor = offlineColor ?? Colors.orange;
          iconData = Icons.cloud_off;
        }
        iconWidget = Icon(iconData, size: iconSize, color: iconColor);
        break;

      case ss.SyncStatus.pending:
      default:
        if (isOnline) {
          iconColor = onlineColor ?? Colors.green;
          iconData = Icons.cloud;
        } else {
          iconColor = offlineColor ?? Colors.orange;
          iconData = Icons.cloud_off;
        }
        iconWidget = Icon(iconData, size: iconSize, color: iconColor);
        break;
    }

    return iconWidget;
  }

  Widget _buildStatusText(BuildContext context, SyncState syncState) {
    var textStyle = Theme.of(context).textTheme.bodySmall ?? const TextStyle();

    String statusText;
    Color? statusColor;

    switch (syncState.status) {
      case ss.SyncStatus.syncing:
        statusText = 'Syncing...';
        statusColor = syncingColor ?? Colors.blue;
        break;

      case ss.SyncStatus.failed:
        statusText = syncState.lastError ?? 'Sync failed';
        statusColor = errorColor ?? Colors.red;
        break;

      case ss.SyncStatus.completed:
        if (syncState.isOnline) {
          statusText = 'Synced';
          statusColor = onlineColor ?? Colors.green;
        } else {
          statusText = 'Offline';
          statusColor = offlineColor ?? Colors.orange;
        }
        break;

      case ss.SyncStatus.pending:
      default:
        if (syncState.isOnline) {
          statusText = 'Online';
          statusColor = onlineColor ?? Colors.green;
        } else {
          statusText = 'Offline';
          statusColor = offlineColor ?? Colors.orange;
        }
        break;
    }

    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text(
          statusText,
          style: textStyle.copyWith(color: statusColor),
        ),
        if (syncState.pendingCount > 0 && syncState.isOnline)
          Text(
            '${syncState.pendingCount} pending',
            style: textStyle.copyWith(
              color: Colors.orange,
              fontSize: (textStyle.fontSize ?? 10) * 0.8,
            ),
          ),
      ],
    );
  }

  @override
  void debugFillProperties(DiagnosticPropertiesBuilder properties) {
    super.debugFillProperties(properties);
    properties.add(DiagnosticsProperty<bool>('showDetails', showDetails));
    properties.add(DoubleProperty('iconSize', iconSize));
    properties.add(ColorProperty('onlineColor', onlineColor));
    properties.add(ColorProperty('offlineColor', offlineColor));
    properties.add(ColorProperty('syncingColor', syncingColor));
    properties.add(ColorProperty('errorColor', errorColor));
  }
}

/// Compact sync status indicator
/// Shows only the icon with tooltip
class SyncStatusIcon extends ConsumerWidget {
  /// Icon size
  final double size;

  const SyncStatusIcon({super.key, this.size = 20});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final syncState = ref.watch(syncStateProvider);

    return Tooltip(
      message: _getTooltipMessage(syncState),
      child: _buildStatusIcon(syncState.status, syncState.isOnline),
    );
  }

  String _getTooltipMessage(SyncState syncState) {
    switch (syncState.status) {
      case ss.SyncStatus.syncing:
        return 'Syncing... ${syncState.pendingCount} pending';
      case ss.SyncStatus.failed:
        return syncState.lastError ?? 'Sync failed';
      case ss.SyncStatus.completed:
        return syncState.isOnline ? 'Synced' : 'Offline - changes will sync when online';
      case ss.SyncStatus.pending:
      default:
        return syncState.isOnline ? 'Online' : 'Offline';
    }
  }

  Widget _buildStatusIcon(ss.SyncStatus status, bool isOnline) {
    Color iconColor;
    IconData iconData;

    switch (status) {
      case ss.SyncStatus.syncing:
        iconColor = Colors.blue;
        iconData = Icons.sync;
        break;

      case ss.SyncStatus.failed:
        iconColor = Colors.red;
        iconData = Icons.sync_problem;
        break;

      case ss.SyncStatus.completed:
        if (isOnline) {
          iconColor = Colors.green;
          iconData = Icons.cloud_done;
        } else {
          iconColor = Colors.orange;
          iconData = Icons.cloud_off;
        }
        break;

      case ss.SyncStatus.pending:
      default:
        if (isOnline) {
          iconColor = Colors.green;
          iconData = Icons.cloud;
        } else {
          iconColor = Colors.orange;
          iconData = Icons.cloud_off;
        }
        break;
    }

    if (status == ss.SyncStatus.syncing) {
      return TweenAnimationBuilder<double>(
        tween: Tween(begin: 0, end: 1),
        duration: const Duration(seconds: 1),
        builder: (context, value, child) => Transform.rotate(
            angle: value * 2 * 3.14159,
            child: Icon(iconData, size: size, color: iconColor),
          ),
      );
    }

    return Icon(iconData, size: size, color: iconColor);
  }

  @override
  void debugFillProperties(DiagnosticPropertiesBuilder properties) {
    super.debugFillProperties(properties);
    properties.add(DoubleProperty('size', size));
  }
}

/// Sync status banner
/// Shows a banner when sync status changes
class SyncStatusBanner extends ConsumerWidget {
  /// Whether to auto-hide
  final bool autoHide;

  /// Duration before auto-hide
  final Duration autoHideDuration;

  const SyncStatusBanner({
    super.key,
    this.autoHide = true,
    this.autoHideDuration = const Duration(seconds: 5),
  });

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final syncState = ref.watch(syncStateProvider);

    // Only show banner for important states
    if (syncState.status == ss.SyncStatus.completed ||
        syncState.status == ss.SyncStatus.pending) {
      return const SizedBox.shrink();
    }

    final (icon, color, message) = _getBannerContent(syncState);

    return Material(
      color: color.withOpacity(0.1),
      borderRadius: BorderRadius.circular(8),
      child: Padding(
        padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 12),
        child: Row(
          children: [
            Icon(icon, color: color, size: 24),
            const SizedBox(width: 12),
            Expanded(
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Text(
                    message,
                    style: TextStyle(
                      color: color,
                      fontWeight: FontWeight.w500,
                    ),
                  ),
                  if (syncState.lastError != null)
                    Text(
                      syncState.lastError!,
                      style: TextStyle(
                        color: color.withOpacity(0.8),
                        fontSize: 12,
                      ),
                      maxLines: 2,
                      overflow: TextOverflow.ellipsis,
                    ),
                ],
              ),
            ),
            if (syncState.status == ss.SyncStatus.syncing)
              SizedBox(
                width: 20,
                height: 20,
                child: CircularProgressIndicator(
                  strokeWidth: 2,
                  valueColor: AlwaysStoppedAnimation<Color>(color),
                ),
              ),
          ],
        ),
      ),
    );
  }

  (IconData, Color, String) _getBannerContent(SyncState syncState) {
    switch (syncState.status) {
      case ss.SyncStatus.syncing:
        return (
          Icons.sync,
          Colors.blue,
          'Syncing ${syncState.pendingCount} documents...'
        );
      case ss.SyncStatus.failed:
        return (
          Icons.sync_problem,
          Colors.red,
          syncState.lastError ?? 'Sync failed'
        );
      case ss.SyncStatus.completed:
      case ss.SyncStatus.pending:
      default:
        return (
          Icons.cloud,
          Colors.green,
          syncState.isOnline ? 'All changes synced' : 'Working offline'
        );
    }
  }

  @override
  void debugFillProperties(DiagnosticPropertiesBuilder properties) {
    super.debugFillProperties(properties);
    properties.add(DiagnosticsProperty<bool>('autoHide', autoHide));
    properties.add(DiagnosticsProperty<Duration>('autoHideDuration', autoHideDuration));
  }
}

/// Sync status row
/// Shows sync status with action button
class SyncStatusRow extends ConsumerWidget {
  const SyncStatusRow({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final syncState = ref.watch(syncStateProvider);
    final syncNotifier = ref.read(syncStateProvider.notifier);

    return Card(
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Row(
          children: [
            const Expanded(
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Text(
                    'Sync Status',
                    style: TextStyle(
                      fontWeight: FontWeight.bold,
                      fontSize: 16,
                    ),
                  ),
                  SizedBox(height: 4),
                  SyncStatusIndicator(
                    showDetails: true,
                    onlineColor: Colors.green,
                    offlineColor: Colors.orange,
                    syncingColor: Colors.blue,
                    errorColor: Colors.red,
                  ),
                ],
              ),
            ),
            const SizedBox(width: 16),
            ElevatedButton.icon(
              onPressed: syncState.isOnline &&
                      (syncState.pendingCount > 0 ||
                          syncState.status == ss.SyncStatus.pending)
                  ? syncNotifier.syncAllPending
                  : null,
              icon: const Icon(Icons.refresh),
              label: const Text('Sync Now'),
            ),
          ],
        ),
      ),
    );
  }
}
