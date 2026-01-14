import 'package:riverpod/riverpod.dart';
import 'package:miniwiki/domain/entities/document_version.dart';
import 'package:miniwiki/services/version_service.dart';
import 'package:miniwiki/core/config/providers.dart';

/// State for the version list
class VersionListState {
  final List<DocumentVersion> versions;
  final int total;
  final bool isLoading;
  final bool isLoadingMore;
  final String? error;
  final String documentId;
  final int limit;

  const VersionListState({
    required this.documentId,
    this.versions = const [],
    this.total = 0,
    this.isLoading = false,
    this.isLoadingMore = false,
    this.error,
    this.limit = 20,
  });

  VersionListState copyWith({
    List<DocumentVersion>? versions,
    int? total,
    bool? isLoading,
    bool? isLoadingMore,
    String? error,
    String? documentId,
    int? limit,
  }) =>
      VersionListState(
        versions: versions ?? this.versions,
        total: total ?? this.total,
        isLoading: isLoading ?? this.isLoading,
        isLoadingMore: isLoadingMore ?? this.isLoadingMore,
        error: error ?? this.error,
        documentId: documentId ?? this.documentId,
        limit: limit ?? this.limit,
      );

  bool get hasMore => versions.length < total;
}

/// State for version comparison
class VersionComparisonState {
  final DocumentVersion? fromVersion;
  final DocumentVersion? toVersion;
  final Map<String, dynamic>? diff;
  final bool isComparing;
  final String? error;

  const VersionComparisonState({
    this.fromVersion,
    this.toVersion,
    this.diff,
    this.isComparing = false,
    this.error,
  });

  VersionComparisonState copyWith({
    DocumentVersion? fromVersion,
    DocumentVersion? toVersion,
    Map<String, dynamic>? diff,
    bool? isComparing,
    String? error,
  }) =>
      VersionComparisonState(
        fromVersion: fromVersion ?? this.fromVersion,
        toVersion: toVersion ?? this.toVersion,
        diff: diff ?? this.diff,
        isComparing: isComparing ?? this.isComparing,
        error: error ?? this.error,
      );

  bool get canCompare => fromVersion != null && toVersion != null;
}

/// Provider for version list state
class VersionListNotifier extends StateNotifier<VersionListState> {
  final VersionService _service;

  VersionListNotifier(this._service, String documentId)
      : super(VersionListState(documentId: documentId));

  Future<void> loadVersions({int? limit}) async {
    state = state.copyWith(isLoading: true, error: null);

    try {
      final versions = await _service.listVersions(
        state.documentId,
        limit: limit ?? state.limit,
      );
      final count = await _service.getVersionCount(state.documentId);

      state = state.copyWith(
        versions: versions,
        total: count,
        isLoading: false,
      );
    } catch (e) {
      state = state.copyWith(
        isLoading: false,
        error: e.toString(),
      );
    }
  }

  Future<void> loadMoreVersions() async {
    if (!state.hasMore || state.isLoadingMore) return;

    state = state.copyWith(isLoadingMore: true);

    try {
      final moreVersions = await _service.listVersions(
        state.documentId,
        limit: state.limit,
        offset: state.versions.length,
      );

      state = state.copyWith(
        versions: [...state.versions, ...moreVersions],
        isLoadingMore: false,
      );
    } catch (e) {
      state = state.copyWith(
        isLoadingMore: false,
        error: e.toString(),
      );
    }
  }

  Future<void> refreshVersions() async {
    await loadVersions();
  }

  Future<void> restoreVersion(int versionNumber) async {
    state = state.copyWith(isLoading: true, error: null);

    try {
      await _service.restoreVersion(state.documentId, versionNumber);
      // Reload versions after restore
      await loadVersions();
    } catch (e) {
      state = state.copyWith(
        isLoading: false,
        error: e.toString(),
      );
      rethrow;
    }
  }
}

/// Provider for version comparison state
class VersionComparisonNotifier extends StateNotifier<VersionComparisonState> {
  final VersionService _service;

  VersionComparisonNotifier(this._service)
      : super(const VersionComparisonState());

  void selectFromVersion(DocumentVersion version) {
    state = state.copyWith(fromVersion: version);
    if (state.canCompare) {
      compareVersions();
    }
  }

  void selectToVersion(DocumentVersion version) {
    state = state.copyWith(toVersion: version);
    if (state.canCompare) {
      compareVersions();
    }
  }

  Future<void> compareVersions() async {
    if (!state.canCompare) return;

    state = state.copyWith(isComparing: true, error: null);

    try {
      final diff = await _service.compareVersions(
        state.fromVersion!.documentId,
        state.fromVersion!.versionNumber,
        state.toVersion!.versionNumber,
      );

      state = state.copyWith(
        diff: diff,
        isComparing: false,
      );
    } catch (e) {
      state = state.copyWith(
        isComparing: false,
        error: e.toString(),
      );
    }
  }

  void clearComparison() {
    state = const VersionComparisonState();
  }
}

/// Provider for version list notifier
final versionListNotifierProvider =
    StateNotifierProvider.family<VersionListNotifier, VersionListState, String>(
  (ref, documentId) {
    final service = ref.watch(versionServiceProvider);
    return VersionListNotifier(service, documentId);
  },
);

/// Provider for version comparison notifier
final versionComparisonNotifierProvider =
    StateNotifierProvider<VersionComparisonNotifier, VersionComparisonState>(
  (ref) {
    final service = ref.watch(versionServiceProvider);
    return VersionComparisonNotifier(service);
  },
);
