import 'package:miniwiki/domain/entities/share_link.dart';
import 'package:miniwiki/services/providers.dart';
import 'package:miniwiki/services/share_service.dart';
import 'package:riverpod/riverpod.dart';

/// State for share links list
class ShareLinksState {
  final List<ShareLink> shareLinks;
  final bool isLoading;
  final String? error;
  final String documentId;

  const ShareLinksState({
    required this.documentId,
    this.shareLinks = const [],
    this.isLoading = false,
    this.error,
  });

  // Sentinel value to detect explicit null
  static const _unset = Object();

  ShareLinksState copyWith({
    List<ShareLink>? shareLinks,
    bool? isLoading,
    Object? error = _unset,
    String? documentId,
  }) =>
      ShareLinksState(
        documentId: documentId ?? this.documentId,
        shareLinks: shareLinks ?? this.shareLinks,
        isLoading: isLoading ?? this.isLoading,
        error: identical(error, _unset) ? this.error : error as String?,
      );
}

/// State for share link creation
class ShareLinkCreateState {
  final String permission;
  final bool requireAccessCode;
  final String accessCode;
  final DateTime? expiresAt;
  final int? maxAccessCount;
  final bool isCreating;
  final ShareLink? createdLink;
  final String? error;

  const ShareLinkCreateState({
    this.permission = 'view',
    this.requireAccessCode = false,
    this.accessCode = '',
    this.expiresAt,
    this.maxAccessCount,
    this.isCreating = false,
    this.createdLink,
    this.error,
  });

  // Sentinel value to detect explicit null
  static const _clearSentinel = Object();

  ShareLinkCreateState copyWith({
    String? permission,
    bool? requireAccessCode,
    String? accessCode,
    Object? expiresAt = _clearSentinel,
    Object? maxAccessCount = _clearSentinel,
    bool? isCreating,
    Object? createdLink = _clearSentinel,
    Object? error = _clearSentinel,
  }) =>
      ShareLinkCreateState(
        permission: permission ?? this.permission,
        requireAccessCode: requireAccessCode ?? this.requireAccessCode,
        accessCode: accessCode ?? this.accessCode,
        expiresAt: identical(expiresAt, _clearSentinel)
            ? this.expiresAt
            : expiresAt as DateTime?,
        maxAccessCount: identical(maxAccessCount, _clearSentinel)
            ? this.maxAccessCount
            : maxAccessCount as int?,
        isCreating: isCreating ?? this.isCreating,
        createdLink: identical(createdLink, _clearSentinel)
            ? this.createdLink
            : createdLink as ShareLink?,
        error: identical(error, _clearSentinel) ? this.error : error as String?,
      );

  bool get isValid => !requireAccessCode || accessCode.length >= 4;
}

/// Provider for share links list state
class ShareLinksNotifier extends StateNotifier<ShareLinksState> {
  final ShareService _service;
  final String documentId;

  ShareLinksNotifier(this._service, this.documentId)
      : super(ShareLinksState(documentId: documentId));

  Future<void> loadShareLinks() async {
    state = state.copyWith(isLoading: true, error: null);

    try {
      final links = await _service.getShareLinks(documentId);
      state = state.copyWith(
        shareLinks: links,
        isLoading: false,
      );
    } catch (e) {
      state = state.copyWith(
        isLoading: false,
        error: e.toString(),
      );
    }
  }

  Future<bool> deleteShareLink(String token) async {
    state = state.copyWith(isLoading: true, error: null);

    try {
      await _service.deleteShareLink(documentId, token);
      state = state.copyWith(
        shareLinks: state.shareLinks.where((l) => l.token != token).toList(),
        isLoading: false,
      );
      return true;
    } catch (e) {
      state = state.copyWith(
        isLoading: false,
        error: e.toString(),
      );
      return false;
    }
  }

  Future<bool> copyShareLink(ShareLink link) async => _service.copyShareLink(link);
}

/// Provider for share link creation state
class ShareLinkCreateNotifier extends StateNotifier<ShareLinkCreateState> {
  final ShareService _service;
  final String documentId;

  ShareLinkCreateNotifier(this._service, this.documentId)
      : super(const ShareLinkCreateState());

  void setPermission(String value) {
    state = state.copyWith(permission: value);
  }

  void setRequireAccessCode(bool value) {
    state = state.copyWith(
      requireAccessCode: value,
      accessCode: value ? state.accessCode : '',
    );
  }

  void setAccessCode(String value) {
    state = state.copyWith(accessCode: value);
  }

  void setExpiresAt(DateTime? value) {
    state = state.copyWith(expiresAt: value);
  }

  void setMaxAccessCount(int? value) {
    state = state.copyWith(maxAccessCount: value);
  }

  void clearError() {
    state = state.copyWith(error: null);
  }

  void clearCreatedLink() {
    state = state.copyWith(createdLink: null);
  }

  Future<bool> createShareLink() async {
    if (!state.isValid) {
      state =
          state.copyWith(error: 'Access code must be at least 4 characters');
      return false;
    }

    state = state.copyWith(isCreating: true, error: null);

    try {
      final link = await _service.createShareLink(
        documentId: documentId,
        permission: state.permission,
        accessCode: state.requireAccessCode ? state.accessCode : null,
        expiresAt: state.expiresAt,
        maxAccessCount: state.maxAccessCount,
      );
      state = state.copyWith(
        isCreating: false,
        createdLink: link,
      );
      return true;
    } catch (e) {
      state = state.copyWith(
        isCreating: false,
        error: e.toString(),
      );
      return false;
    }
  }

  void reset() {
    state = const ShareLinkCreateState();
  }
}

/// Provider for share links notifier
final shareLinksNotifierProvider =
    StateNotifierProvider.family<ShareLinksNotifier, ShareLinksState, String>(
  (ref, documentId) {
    final service = ref.watch(shareServiceProvider);
    return ShareLinksNotifier(service, documentId);
  },
);

/// Provider for share link creation notifier
final shareLinkCreateNotifierProvider = StateNotifierProvider.family<
    ShareLinkCreateNotifier, ShareLinkCreateState, String>((ref, documentId) {
  final service = ref.watch(shareServiceProvider);
  return ShareLinkCreateNotifier(service, documentId);
});
