import 'package:miniwiki/domain/entities/document.dart';
import 'package:miniwiki/domain/repositories/document_repository.dart';
import 'package:miniwiki/services/sync_service.dart';

/// Service class for document-related business logic
///
/// This service wraps the repository and provides higher-level
/// operations for document management in the miniWiki app.
class DocumentService {
  final DocumentRepository _repository;
  final SyncService? _syncService;

  DocumentService(this._repository, [this._syncService]);

  /// Creates a new document in the specified space
  ///
  /// [spaceId] The ID of the space to create the document in
  /// [title] The title of the new document
  /// [parentId] Optional parent document ID for hierarchy
  /// [icon] Optional icon for the document
  /// [content] Initial content (empty map if not provided)
  ///
  /// Returns the created document
  Future<Document> createDocument({
    required String spaceId,
    required String title,
    String? parentId,
    String? icon,
    Map<String, dynamic>? content,
  }) {
    return _repository.createDocument(
      spaceId: spaceId,
      parentId: parentId,
      title: _sanitizeTitle(title),
      icon: icon,
      content: content ?? {},
    );
  }

  /// Fetches a document by its ID
  ///
  /// [id] The document ID
  /// [forceRefresh] If true, bypasses local cache
  ///
  /// Returns the document if found
  Future<Document> getDocument(String id, {bool forceRefresh = false}) {
    if (forceRefresh) {
      return _forceRefreshDocument(id);
    }
    return _repository.getDocument(id);
  }

  Future<Document> _forceRefreshDocument(String id) async {
    return _repository.getDocument(id);
  }

  /// Updates a document's title
  ///
  /// [id] The document ID
  /// [title] The new title
  ///
  /// Returns the updated document
  Future<Document> updateTitle(String id, String title) {
    return _repository.updateDocument(
      id: id,
      title: _sanitizeTitle(title),
    );
  }

  /// Updates a document's icon
  ///
  /// [id] The document ID
  /// [icon] The new icon identifier
  ///
  /// Returns the updated document
  Future<Document> updateIcon(String id, String? icon) {
    return _repository.updateDocument(
      id: id,
      icon: icon,
    );
  }

  /// Updates a document's content with auto-save
  ///
  /// [id] The document ID
  /// [content] The new content
  /// [autoSave] Whether to queue for auto-sync (default: true)
  ///
  /// Returns the updated document
  Future<Document> updateContent(
    String id,
    Map<String, dynamic> content, {
    bool autoSave = true,
  }) async {
    final document = await _repository.updateDocument(
      id: id,
      content: content,
    );

    if (autoSave && _syncService != null) {
      await _syncService!.queueDocumentForSync(id);
    }

    return document;
  }

  /// Performs a full document update with auto-save
  ///
  /// [id] The document ID
  /// [title] Optional new title
  /// [icon] Optional new icon
  /// [content] Optional new content
  /// [autoSave] Whether to queue for auto-sync (default: true)
  ///
  /// Returns the updated document
  Future<Document> updateDocument({
    required String id,
    String? title,
    String? icon,
    Map<String, dynamic>? content,
    bool autoSave = true,
  }) async {
    final document = await _repository.updateDocument(
      id: id,
      title: title != null ? _sanitizeTitle(title) : null,
      icon: icon,
      content: content,
    );

    if (autoSave && _syncService != null && content != null) {
      await _syncService!.queueDocumentForSync(id);
    }

    return document;
  }

  /// Soft deletes a document (archives it)
  ///
  /// [id] The document ID to delete
  ///
  /// Returns when the deletion is complete
  Future<void> deleteDocument(String id) {
    return _repository.deleteDocument(id);
  }

  /// Lists documents in a space
  ///
  /// [spaceId] The space ID to list documents from
  /// [parentId] Optional parent filter
  /// [limit] Maximum number of documents
  /// [offset] Number of documents to skip
  ///
  /// Returns a paginated list of documents
  Future<DocumentListResult> listDocuments({
    required String spaceId,
    String? parentId,
    int limit = 20,
    int offset = 0,
  }) {
    return _repository.listDocuments(
      spaceId: spaceId,
      parentId: parentId,
      limit: limit,
      offset: offset,
    );
  }

  /// Gets child documents of a parent
  ///
  /// [parentId] The parent document ID
  ///
  /// Returns a list of child documents
  Future<DocumentListResult> getChildren(String parentId) {
    return _repository.getDocumentChildren(parentId);
  }

  /// Gets the document path from root to the specified document
  ///
  /// [documentId] The target document ID
  ///
  /// Returns a list of documents from root to target
  Future<List<Document>> getDocumentPath(String documentId) {
    return _repository.getDocumentPath(documentId);
  }

  /// Gets version history for a document
  ///
  /// [documentId] The document ID
  /// [limit] Maximum versions to return
  /// [offset] Number of versions to skip
  ///
  /// Returns a paginated list of versions
  Future<VersionListResult> getVersions(
    String documentId, {
    int limit = 20,
    int offset = 0,
  }) {
    return _repository.getDocumentVersions(
      documentId,
      limit: limit,
      offset: offset,
    );
  }

  /// Creates a new version (save point)
  ///
  /// [documentId] The document ID
  /// [content] The content to save
  /// [title] The title at this version
  /// [changeSummary] Optional description of changes
  ///
  /// Returns the created version
  Future<DocumentVersion> createVersion({
    required String documentId,
    required Map<String, dynamic> content,
    required String title,
    String? changeSummary,
  }) {
    return _repository.createVersion(
      documentId: documentId,
      content: content,
      title: _sanitizeTitle(title),
      changeSummary: changeSummary,
    );
  }

  /// Restores a document to a previous version
  ///
  /// [documentId] The document ID
  /// [versionNumber] The version number to restore to
  ///
  /// Returns the restored document
  Future<Document> restoreVersion(
    String documentId,
    int versionNumber,
  ) {
    return _repository.restoreVersion(documentId, versionNumber);
  }

  /// Gets the diff between two versions
  ///
  /// [documentId] The document ID
  /// [fromVersion] Starting version number
  /// [toVersion] Ending version number
  ///
  /// Returns the diff between versions
  Future<VersionDiff> getVersionDiff(
    String documentId,
    int fromVersion,
    int toVersion,
  ) {
    return _repository.getVersionDiff(documentId, fromVersion, toVersion);
  }

  /// Searches documents by title in a space
  ///
  /// [spaceId] The space ID to search in
  /// [query] The search query
  /// [limit] Maximum results
  ///
  /// Returns matching documents
  Future<DocumentListResult> searchDocuments({
    required String spaceId,
    required String query,
    int limit = 20,
  }) async {
    final result = await _repository.listDocuments(
      spaceId: spaceId,
      limit: 100,
    );

    final lowerQuery = query.toLowerCase();
    final filtered = result.documents
        .where((doc) => doc.title.toLowerCase().contains(lowerQuery))
        .take(limit)
        .toList();

    return DocumentListResult(
      documents: filtered,
      total: filtered.length,
      limit: limit,
      offset: 0,
    );
  }

  /// Gets recently updated documents in a space
  ///
  /// [spaceId] The space ID
  /// [limit] Maximum results
  ///
  /// Returns documents sorted by update time
  Future<List<Document>> getRecentDocuments(
    String spaceId, {
    int limit = 10,
  }) async {
    final result = await _repository.listDocuments(
      spaceId: spaceId,
      limit: 50,
    );

    final sorted = List<Document>.from(result.documents)
      ..sort((a, b) {
        final aTime = a.updatedAt ?? DateTime(0);
        final bTime = b.updatedAt ?? DateTime(0);
        return bTime.compareTo(aTime);
      });

    return sorted.take(limit).toList();
  }

  /// Sanitizes document title
  ///
  /// Removes leading/trailing whitespace and limits length
  String _sanitizeTitle(String title) {
    return title.trim().substring(0, min(title.trim().length, 200));
  }

  int min(int a, int b) => a < b ? a : b;
}
