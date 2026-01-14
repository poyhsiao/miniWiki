import 'package:miniwiki/domain/entities/document_version.dart';

/// Repository interface for version operations
///
/// This interface defines the contract for version data access,
/// supporting both online (API) and offline (local database) operations.
abstract class VersionRepository {
  /// List versions of a document
  ///
  /// [documentId] The ID of the document
  /// [limit] Maximum number of versions to return (default 20)
  /// [offset] Number of versions to skip
  ///
  /// Returns a list of versions sorted by version number (descending)
  Future<List<DocumentVersion>> listVersions(
    String documentId, {
    int limit = 20,
    int offset = 0,
  });

  /// Get a specific version
  ///
  /// [documentId] The ID of the document
  /// [versionNumber] The version number to retrieve
  ///
  /// Returns the version if found
  Future<DocumentVersion> getVersion(String documentId, int versionNumber);

  /// Create a new version (manual save point)
  ///
  /// [documentId] The ID of the document
  /// [title] The title at this version
  /// [content] The content to save in this version
  /// [changeSummary] Optional description of changes
  /// [vectorClock] CRDT vector clock for conflict resolution
  ///
  /// Returns the created version
  Future<DocumentVersion> createVersion({
    required String documentId,
    required String title,
    required Map<String, dynamic> content,
    String? changeSummary,
    Map<String, dynamic>? vectorClock,
  });

  /// Restore document to a specific version
  ///
  /// [documentId] The ID of the document
  /// [versionNumber] The version number to restore to
  ///
  /// Returns the restored version (new version created from old content)
  Future<DocumentVersion> restoreVersion(
    String documentId,
    int versionNumber,
  );

  /// Compare two versions
  ///
  /// [documentId] The ID of the document
  /// [fromVersion] The starting version number
  /// [toVersion] The ending version number
  ///
  /// Returns the diff between the two versions
  Future<Map<String, dynamic>> compareVersions(
    String documentId,
    int fromVersion,
    int toVersion,
  );

  /// Get total version count for a document
  ///
  /// [documentId] The ID of the document
  ///
  /// Returns the total number of versions
  Future<int> getVersionCount(String documentId);

  /// Get the current (latest) version of a document
  ///
  /// [documentId] The ID of the document
  ///
  /// Returns the current version
  Future<DocumentVersion> getCurrentVersion(String documentId);
}
