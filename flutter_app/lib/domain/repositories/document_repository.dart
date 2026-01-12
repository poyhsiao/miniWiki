import 'package:miniwiki/data/models/document_entity.dart';

/// Repository interface for document operations
abstract class DocumentRepository {
  /// Create a new document
  Future<DocumentEntity> createDocument({
    required String spaceId,
    required String? parentId,
    required String title,
    required String? icon,
    required Map<String, dynamic> content,
  });

  /// Get a document by ID
  Future<DocumentEntity> getDocument(String id);

  /// Update a document
  Future<DocumentEntity> updateDocument({
    required String id,
    required String? title,
    required String? icon,
    required Map<String, dynamic>? content,
  });

  /// Delete a document (soft delete)
  Future<void> deleteDocument(String id);

  /// List documents in a space
  Future<DocumentListResult> listDocuments({
    required String spaceId,
    String? parentId,
    int limit = 20,
    int offset = 0,
  });

  /// Get document versions
  Future<VersionListResult> getDocumentVersions(String documentId);

  /// Create a new version
  Future<DocumentVersion> createVersion({
    required String documentId,
    required Map<String, dynamic> content,
    required String title,
    String? changeSummary,
  });

  /// Restore document to a specific version
  Future<DocumentEntity> restoreVersion(String documentId, int versionNumber);
}

/// Result of listing documents
class DocumentListResult {
  final List<DocumentEntity> documents;
  final int total;
  final int limit;
  final int offset;

  const DocumentListResult({
    required this.documents,
    required this.total,
    required this.limit,
    required this.offset,
  });
}

/// Document version information
class DocumentVersion {
  final String id;
  final String documentId;
  final int versionNumber;
  final String title;
  final Map<String, dynamic> content;
  final String createdBy;
  final DateTime createdAt;
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
  final List<DocumentVersion> versions;
  final int total;

  const VersionListResult({
    required this.versions,
    required this.total,
  });
}
