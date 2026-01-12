import 'package:miniwiki/domain/entities/document.dart';

/// Repository interface for document operations
///
/// This interface defines the contract for document data access,
/// supporting both online (API) and offline (local database) operations.
abstract class DocumentRepository {
  /// Create a new document
  ///
  /// [spaceId] The ID of the space to create the document in
  /// [parentId] The ID of the parent document (optional, for hierarchy)
  /// [title] The title of the document
  /// [icon] The icon for the document (optional)
  /// [content] The initial content as JSON (Yjs CRDT state)
  ///
  /// Returns the created document
  Future<Document> createDocument({
    required String spaceId,
    required String? parentId,
    required String title,
    required String? icon,
    required Map<String, dynamic> content,
  });

  /// Get a document by ID
  ///
  /// [id] The ID of the document to retrieve
  ///
  /// Returns the document if found, throws exception otherwise
  Future<Document> getDocument(String id);

  /// Update a document
  ///
  /// [id] The ID of the document to update
  /// [title] The new title (optional)
  /// [icon] The new icon (optional)
  /// [content] The new content (optional)
  ///
  /// Returns the updated document
  Future<Document> updateDocument({
    required String id,
    String? title,
    String? icon,
    Map<String, dynamic>? content,
  });

  /// Delete a document (soft delete)
  ///
  /// [id] The ID of the document to delete
  ///
  /// The document will be marked as archived, not permanently deleted
  Future<void> deleteDocument(String id);

  /// List documents in a space
  ///
  /// [spaceId] The ID of the space to list documents from
  /// [parentId] Filter by parent document ID (optional)
  /// [limit] Maximum number of documents to return (default 20)
  /// [offset] Number of documents to skip (for pagination)
  ///
  /// Returns a list of documents with pagination info
  Future<DocumentListResult> listDocuments({
    required String spaceId,
    String? parentId,
    int limit = 20,
    int offset = 0,
  });

  /// Get children documents of a parent document
  ///
  /// [parentId] The ID of the parent document
  ///
  /// Returns a list of child documents
  Future<DocumentListResult> getDocumentChildren(String parentId);

  /// Get the full path from root to a document (breadcrumb)
  ///
  /// [documentId] The ID of the document
  ///
  /// Returns a list of documents from root to the target document
  Future<List<Document>> getDocumentPath(String documentId);

  /// Get document versions/history
  ///
  /// [documentId] The ID of the document
  /// [limit] Maximum number of versions to return (default 20)
  /// [offset] Number of versions to skip
  ///
  /// Returns a list of versions with pagination info
  Future<VersionListResult> getDocumentVersions(
    String documentId, {
    int limit = 20,
    int offset = 0,
  });

  /// Create a new version (manual save point)
  ///
  /// [documentId] The ID of the document
  /// [content] The content to save in this version
  /// [title] The title at this version
  /// [changeSummary] Optional description of changes
  ///
  /// Returns the created version
  Future<DocumentVersion> createVersion({
    required String documentId,
    required Map<String, dynamic> content,
    required String title,
    String? changeSummary,
  });

  /// Restore document to a specific version
  ///
  /// [documentId] The ID of the document
  /// [versionNumber] The version number to restore to
  ///
  /// Returns the restored document
  Future<Document> restoreVersion(String documentId, int versionNumber);

  /// Get version diff between two versions
  ///
  /// [documentId] The ID of the document
  /// [fromVersion] The starting version number
  /// [toVersion] The ending version number
  ///
  /// Returns the diff between the two versions
  Future<VersionDiff> getVersionDiff(
    String documentId,
    int fromVersion,
    int toVersion,
  );
}

/// Result of listing documents
class DocumentListResult {
  /// List of documents in this result
  final List<Document> documents;

  /// Total number of documents matching the query
  final int total;

  /// Maximum number of documents requested
  final int limit;

  /// Number of documents skipped
  final int offset;

  const DocumentListResult({
    required this.documents,
    required this.total,
    required this.limit,
    required this.offset,
  });

  /// Whether there are more documents to load
  bool get hasMore => offset + documents.length < total;
}

/// Document version information
class DocumentVersion {
  /// Unique identifier for the version
  final String id;

  /// ID of the parent document
  final String documentId;

  /// Sequential version number
  final int versionNumber;

  /// Title at this version
  final String title;

  /// Content at this version (Yjs CRDT state)
  final Map<String, dynamic> content;

  /// ID of the user who created this version
  final String createdBy;

  /// Timestamp when this version was created
  final DateTime createdAt;

  /// Optional description of changes
  final String? changeSummary;

  const DocumentVersion({
    required this.id,
    required this.documentId,
    required this.versionNumber,
    required this.title,
    required this.content,
    required this.createdBy,
    required this.createdAt,
    this.changeSummary,
  });
}

/// Result of listing versions
class VersionListResult {
  /// List of versions in this result
  final List<DocumentVersion> versions;

  /// Total number of versions
  final int total;

  const VersionListResult({
    required this.versions,
    required this.total,
  });

  /// Whether there are more versions to load
  bool get hasMore => versions.length < total;
}

/// Version diff information
class VersionDiff {
  /// The starting version number
  final int fromVersion;

  /// The ending version number
  final int toVersion;

  /// Content at the from version
  final Map<String, dynamic> fromContent;

  /// Content at the to version
  final Map<String, dynamic> toContent;

  const VersionDiff({
    required this.fromVersion,
    required this.toVersion,
    required this.fromContent,
    required this.toContent,
  });
}
