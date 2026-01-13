// Sync provider for managing sync state with Riverpod
// Provides sync status, events, and actions throughout the app

import 'dart:async';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:miniwiki/services/sync_service.dart' as ss;
import 'package:miniwiki/services/offline_service.dart';
import 'package:miniwiki/services/crdt_service.dart';
import 'package:connectivity_plus/connectivity_plus.dart';

/// Sync state for Riverpod
class SyncState {
  final ss.SyncStatus status;
  final int pendingCount;
  final bool isOnline;
  final bool autoSyncEnabled;
  final String? lastError;
  final DateTime? lastSuccessfulSync;
  final List<ss.SyncEvent> recentEvents;
  final int syncIntervalSeconds;

  const SyncState({
    required this.status,
    required this.pendingCount,
    required this.isOnline,
    required this.autoSyncEnabled,
    this.lastError,
    this.lastSuccessfulSync,
    this.recentEvents = const [],
    this.syncIntervalSeconds = 30,
  });

  factory SyncState.initial() => SyncState(
        status: ss.SyncStatus.pending,
        pendingCount: 0,
        isOnline: true,
        autoSyncEnabled: true,
        recentEvents: [],
        syncIntervalSeconds: 30,
      );

  SyncState copyWith({
    ss.SyncStatus? status,
    int? pendingCount,
    bool? isOnline,
    bool? autoSyncEnabled,
    String? lastError,
    DateTime? lastSuccessfulSync,
    List<ss.SyncEvent>? recentEvents,
    int? syncIntervalSeconds,
  }) =>
      SyncState(
        status: status ?? this.status,
        pendingCount: pendingCount ?? this.pendingCount,
        isOnline: isOnline ?? this.isOnline,
        autoSyncEnabled: autoSyncEnabled ?? this.autoSyncEnabled,
        lastError: lastError ?? this.lastError,
        lastSuccessfulSync: lastSuccessfulSync ?? this.lastSuccessfulSync,
        recentEvents: recentEvents ?? this.recentEvents,
        syncIntervalSeconds: syncIntervalSeconds ?? this.syncIntervalSeconds,
      );
}

/// Sync state notifier - manages sync state and operations
class SyncStateNotifier extends StateNotifier<SyncState> {
  final ss.SyncService _syncService;
  final OfflineService _offlineService;

  StreamSubscription? _syncEventsSubscription;
  StreamSubscription? _offlineStateSubscription;

  SyncStateNotifier({
    required ss.SyncService syncService,
    required OfflineService offlineService,
  })  : _syncService = syncService,
        _offlineService = offlineService,
        super(SyncState.initial()) {
    _initialize();
  }

  /// Initialize sync state subscriptions
  void _initialize() {
    // Listen to sync events
    _syncEventsSubscription = _syncService.syncEvents.listen(_handleSyncEvent);

    // Listen to offline state changes
    _offlineStateSubscription = _offlineService.stateChanges.listen((offlineState) {
      state = state.copyWith(
        isOnline: offlineState.isOnline,
        pendingCount: offlineState.pendingQueueCount,
      );

      // Trigger sync when coming back online
      if (offlineState.isOnline && offlineState.pendingQueueCount > 0) {
        syncAllPending();
      }
    });
  }

  /// Handle sync events
  void _handleSyncEvent(ss.SyncEvent event) {
    final events = [...state.recentEvents, event];
    if (events.length > 10) {
      events.removeAt(0);
    }

    switch (event.type) {
      case ss.SyncEventType.started:
        state = state.copyWith(
          status: ss.SyncStatus.syncing,
          recentEvents: events,
        );
        break;
      case ss.SyncEventType.success:
        state = state.copyWith(
          status: ss.SyncStatus.completed,
          lastError: null,
          lastSuccessfulSync: event.timestamp,
          recentEvents: events,
        );
        break;
      case ss.SyncEventType.error:
        state = state.copyWith(
          status: ss.SyncStatus.failed,
          lastError: event.message,
          recentEvents: events,
        );
        break;
      case ss.SyncEventType.completed:
        state = state.copyWith(
          status: ss.SyncStatus.pending,
          recentEvents: events,
        );
        break;
      case ss.SyncEventType.online:
        state = state.copyWith(
          isOnline: true,
          recentEvents: events,
        );
        break;
      case ss.SyncEventType.offline:
        state = state.copyWith(
          isOnline: false,
          recentEvents: events,
        );
        break;
    }
  }

  /// Sync all pending documents
  Future<void> syncAllPending() async {
    if (!state.isOnline) return;

    state = state.copyWith(status: ss.SyncStatus.syncing);

    try {
      final summary = await _syncService.syncAllDirtyDocuments();
      state = state.copyWith(
        status: summary.success ? ss.SyncStatus.completed : ss.SyncStatus.failed,
        pendingCount: 0,
        lastSuccessfulSync: summary.success ? DateTime.now() : null,
        lastError: summary.success ? null : 'Some documents failed to sync',
      );
    } catch (e) {
      state = state.copyWith(
        status: ss.SyncStatus.failed,
        lastError: e.toString(),
      );
    }
  }

  /// Sync a single document
  Future<void> syncDocument(String documentId) async {
    if (!state.isOnline) return;

    state = state.copyWith(status: ss.SyncStatus.syncing);

    try {
      final result = await _syncService.syncDocument(documentId);
      state = state.copyWith(
        status: result.success ? ss.SyncStatus.completed : ss.SyncStatus.failed,
        lastError: result.success ? null : result.errorMessage,
      );
    } catch (e) {
      state = state.copyWith(
        status: ss.SyncStatus.failed,
        lastError: e.toString(),
      );
    }
  }

  /// Enable or disable auto-sync
  void setAutoSync(bool enabled) {
    _syncService.setAutoSync(enabled);
    state = state.copyWith(autoSyncEnabled: enabled);
  }

  /// Set sync interval
  void setSyncInterval(int seconds) {
    _syncService.setSyncInterval(seconds);
    state = state.copyWith(syncIntervalSeconds: seconds);
  }

  /// Get pending sync count
  Future<int> getPendingCount() async {
    return await _syncService.getPendingSyncCount();
  }

  /// Clear sync error
  void clearError() {
    state = state.copyWith(lastError: null);
  }

  /// Dispose subscriptions
  @override
  void dispose() {
    _syncEventsSubscription?.cancel();
    _offlineStateSubscription?.cancel();
    super.dispose();
  }
}

/// Simple sync status provider (for widgets that only need status)
final syncStatusProvider = Provider<ss.SyncStatus>((ref) {
  return ref.watch(syncStateProvider).status;
});

/// Online status provider
final isOnlineProvider = Provider<bool>((ref) {
  return ref.watch(syncStateProvider).isOnline;
});

/// Pending sync count provider
final pendingSyncCountProvider = Provider<int>((ref) {
  return ref.watch(syncStateProvider).pendingCount;
});

/// Auto-sync enabled provider
final autoSyncEnabledProvider = Provider<bool>((ref) {
  return ref.watch(syncStateProvider).autoSyncEnabled;
});

/// Sync error provider
final syncErrorProvider = Provider<String?>((ref) {
  return ref.watch(syncStateProvider).lastError;
});

/// Sync state provider (for full state access)
final syncStateProvider = StateNotifierProvider<SyncStateNotifier, SyncState>((ref) {
  // Services should be initialized elsewhere and provided
  // This is a placeholder for dependency injection
  final syncService = ss.SyncService(CrdtService());
  final offlineService = OfflineService(
    syncDatasource: throw UnimplementedError('SyncDatasource not provided'),
    connectivity: Connectivity(),
  );

  return SyncStateNotifier(
    syncService: syncService,
    offlineService: offlineService,
  );
});
