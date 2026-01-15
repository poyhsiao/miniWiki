import 'package:miniwiki/domain/entities/share_link.dart';

/// Repository interface for share link operations.
///
/// This interface defines the contract for share link data operations,
/// following the repository pattern for clean architecture separation.
abstract class ShareRepository {
  /// Create a new share link for a document.
  ///
  /// [request] contains the share link creation parameters including
  /// document ID, optional access code, expiration, and permissions.
  ///
  /// Returns the created [ShareLink].
  Future<ShareLink> createShareLink(CreateShareLinkRequest request);

  /// Get all share links for a specific document.
  ///
  /// [documentId] is the ID of the document.
  ///
  /// Returns a list of [ShareLink]s for the document.
  Future<List<ShareLink>> getShareLinks(String documentId);

  /// Get a share link by its token (public endpoint).
  ///
  /// [token] is the share link token.
  ///
  /// Returns the [ShareLink] if found, or null if not found.
  Future<ShareLink?> getShareLinkByToken(String token);

  /// Verify access code for a protected share link.
  ///
  /// [token] is the share link token.
  /// [accessCode] is the access code to verify.
  ///
  /// Returns [ShareLinkVerification] if verification succeeds.
  Future<ShareLinkVerification> verifyAccessCode(
    String token,
    String accessCode,
  );

  /// Delete (deactivate) a share link.
  ///
  /// [documentId] is the document ID.
  /// [token] is the share link token.
  ///
  /// Throws an exception if the share link doesn't exist or
  /// user doesn't have permission to delete it.
  Future<void> deleteShareLink(String documentId, String token);

  /// Copy share link URL to clipboard.
  ///
  /// [shareLink] is the share link to copy.
  /// [baseUrl] is the base URL for constructing the full share URL.
  ///
  /// Returns true if the copy was successful.
  Future<bool> copyShareLinkToClipboard(ShareLink shareLink, String baseUrl);
}
