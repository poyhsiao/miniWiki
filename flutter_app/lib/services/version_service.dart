import 'package:miniwiki/domain/entities/document_version.dart';
import 'package:miniwiki/domain/repositories/version_repository.dart';

/// Service for document version operations
///
/// This service provides a clean API for version operations,
/// handling business logic and error handling.
class VersionService {
  final VersionRepository versionRepository;

  VersionService({required this.versionRepository});

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
  }) async {
    if (documentId.isEmpty) {
      throw ArgumentError('Document ID cannot be empty');
    }

    if (limit < 1 || limit > 100) {
      throw ArgumentError('Limit must be between 1 and 100');
    }

    if (offset < 0) {
      throw ArgumentError('Offset must be non-negative');
    }

    return versionRepository.listVersions(
      documentId,
      limit: limit,
      offset: offset,
    );
  }

  /// Get a specific version
  ///
  /// [documentId] The ID of the document
  /// [versionNumber] The version number to retrieve
  ///
  /// Returns the version if found
  Future<DocumentVersion> getVersion(
    String documentId,
    int versionNumber,
  ) async {
    if (documentId.isEmpty) {
      throw ArgumentError('Document ID cannot be empty');
    }

    if (versionNumber < 1) {
      throw ArgumentError('Version number must be positive');
    }

    return versionRepository.getVersion(documentId, versionNumber);
  }

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
  }) async {
    if (documentId.isEmpty) {
      throw ArgumentError('Document ID cannot be empty');
    }

    if (title.isEmpty) {
      throw ArgumentError('Title cannot be empty');
    }

    if (content.isEmpty) {
      throw ArgumentError('Content cannot be empty');
    }

    return versionRepository.createVersion(
      documentId: documentId,
      title: title,
      content: content,
      changeSummary: changeSummary,
      vectorClock: vectorClock,
    );
  }

  /// Restore document to a specific version
  ///
  /// [documentId] The ID of the document
  /// [versionNumber] The version number to restore to
  ///
  /// Returns the restored version (new version created from old content)
  Future<DocumentVersion> restoreVersion(
    String documentId,
    int versionNumber,
  ) async {
    if (documentId.isEmpty) {
      throw ArgumentError('Document ID cannot be empty');
    }

    if (versionNumber < 1) {
      throw ArgumentError('Version number must be positive');
    }

    return versionRepository.restoreVersion(documentId, versionNumber);
  }

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
  ) async {
    if (documentId.isEmpty) {
      throw ArgumentError('Document ID cannot be empty');
    }

    if (fromVersion < 1 || toVersion < 1) {
      throw ArgumentError('Version numbers must be positive');
    }

    if (fromVersion >= toVersion) {
      throw ArgumentError('fromVersion must be less than toVersion');
    }

    return versionRepository.compareVersions(
      documentId,
      fromVersion,
      toVersion,
    );
  }

  /// Get total version count for a document
  ///
  /// [documentId] The ID of the document
  ///
  /// Returns the total number of versions
  Future<int> getVersionCount(String documentId) async {
    if (documentId.isEmpty) {
      throw ArgumentError('Document ID cannot be empty');
    }

    return versionRepository.getVersionCount(documentId);
  }

  /// Get the current (latest) version of a document
  ///
  /// [documentId] The ID of the document
  ///
  /// Returns the current version
  Future<DocumentVersion> getCurrentVersion(String documentId) async {
    if (documentId.isEmpty) {
      throw ArgumentError('Document ID cannot be empty');
    }

    return versionRepository.getCurrentVersion(documentId);
  }
}
