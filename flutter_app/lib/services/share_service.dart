import 'package:miniwiki/domain/entities/share_link.dart';
import 'package:miniwiki/domain/repositories/share_repository.dart';

/// Service for share link operations.
///
/// This service provides high-level share link functionality including
/// creating, managing, and deleting share links for documents.
class ShareService {
  final ShareRepository _repository;
  final String _baseUrl;

  ShareService(this._repository, [String? baseUrl])
      : _baseUrl = baseUrl ?? 'http://localhost:3000';

  /// Get the configured base URL for share links.
  String get baseUrl => _baseUrl;

  /// Create a new share link for a document.
  ///
  /// [documentId] is the document to share.
  /// [permission] is the access level ('view' or 'comment').
  /// [accessCode] is an optional password for the share link.
  /// [expiresAt] is an optional expiration date.
  /// [maxAccessCount] is an optional maximum number of accesses.
  ///
  /// Returns the created [ShareLink].
  Future<ShareLink> createShareLink({
    required String documentId,
    String permission = 'view',
    String? accessCode,
    DateTime? expiresAt,
    int? maxAccessCount,
  }) async {
    final request = CreateShareLinkRequest(
      documentId: documentId,
      permission: permission,
      accessCode: accessCode,
      expiresAt: expiresAt,
      maxAccessCount: maxAccessCount,
    );

    return _repository.createShareLink(request);
  }

  /// Get all share links for a document.
  Future<List<ShareLink>> getShareLinks(String documentId) {
    return _repository.getShareLinks(documentId);
  }

  /// Delete a share link.
  Future<void> deleteShareLink(String documentId, String token) {
    return _repository.deleteShareLink(documentId, token);
  }

  /// Copy a share link to clipboard.
  ///
  /// Returns true if successful.
  Future<bool> copyShareLink(ShareLink shareLink) {
    return _repository.copyShareLinkToClipboard(shareLink, _baseUrl);
  }

  /// Get a share link by token (for public access).
  Future<ShareLink?> getShareLinkByToken(String token) {
    return _repository.getShareLinkByToken(token);
  }

  /// Verify access code for a protected share link.
  Future<ShareLinkVerification> verifyAccessCode(
    String token,
    String accessCode,
  ) {
    return _repository.verifyAccessCode(token, accessCode);
  }

  /// Format expiration date for display.
  String formatExpiration(DateTime? expiresAt) {
    if (expiresAt == null) return 'Never';

    final now = DateTime.now();
    final difference = expiresAt.difference(now);

    if (difference.isNegative) {
      return 'Expired';
    }

    if (difference.inDays > 365) {
      final years = difference.inDays ~/ 365;
      return '${years}y left';
    }

    if (difference.inDays > 30) {
      final months = difference.inDays ~/ 30;
      return '${months}mo left';
    }

    if (difference.inDays > 0) {
      return '${difference.inDays}d left';
    }

    if (difference.inHours > 0) {
      return '${difference.inHours}h left';
    }

    return '${difference.inMinutes}m left';
  }

  /// Format access count for display.
  String formatAccessCount(int count, int? maxAccess) {
    if (maxAccess == null) return '$count accesses';

    return '$count/$maxAccess accesses';
  }
}
