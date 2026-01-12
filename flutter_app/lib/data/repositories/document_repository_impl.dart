import 'package:riverpod/riverpod.dart';
import 'package:miniwiki/core/network/api_client.dart';
import 'package:miniwiki/domain/entities/document.dart';
import 'package:miniwiki/domain/repositories/document_repository.dart';
import 'package:miniwiki/data/datasources/isar_datasource.dart';
import 'package:miniwiki/data/models/document_entity.dart';

/// Implementation of DocumentRepository that handles both
/// online (API) and offline (local database) operations
class DocumentRepositoryImpl implements DocumentRepository {
  final ApiClient _apiClient;
  final IsarDatabase _isar;

  DocumentRepositoryImpl(this._apiClient, this._isar);

  @override
  Future<Document> createDocument({
    required String spaceId,
    required String? parentId,
    required String title,
    required String? icon,
    required Map<String, dynamic> content,
  }) async {
    try {
      final response =
          await _apiClient.post('/spaces/$spaceId/documents', data: {
        'title': title,
        'icon': icon,
        'parent_id': parentId,
        'content': content,
      });

      final doc = response.data['data']['document'] as Map<String, dynamic>;
      final document = Document.fromJson(doc);

      // Save to local storage
      await _saveDocumentLocally(document);

      return document;
    } catch (e) {
      // Offline create - create local document
      final now = DateTime.now();
      final localDoc = Document(
        id: DateTime.now().millisecondsSinceEpoch.toString(),
        spaceId: spaceId,
        parentId: parentId,
        title: title,
        icon: icon,
        content: content,
        contentSize: content.length,
        createdBy: 'local',
        lastEditedBy: 'local',
        createdAt: now,
        updatedAt: now,
        isSynced: false,
        isDirty: true,
      );

      await _saveDocumentLocally(localDoc);
      return localDoc;
    }
  }

  @override
  Future<Document> getDocument(String id) async {
    try {
      final response = await _apiClient.get('/documents/$id');
      final doc = response.data['data'] as Map<String, dynamic>;
      final document = Document.fromJson(doc);

      await _saveDocumentLocally(document);

      return document;
    } catch (e) {
      // Fallback to local storage if offline
      final entity = await _isar.getDocumentById(id);
      if (entity != null) {
        return _entityToDocument(entity);
      }
      throw Exception('Document not found');
    }
  }

  @override
  Future<Document> updateDocument({
    required String id,
    String? title,
    String? icon,
    Map<String, dynamic>? content,
  }) async {
    try {
      final requestData = <String, dynamic>{};
      if (title != null) requestData['title'] = title;
      if (icon != null) requestData['icon'] = icon;
      if (content != null) requestData['content'] = content;

      final response =
          await _apiClient.patch('/documents/$id', data: requestData);

      final doc = response.data['data']['document'] as Map<String, dynamic>;
      final document = Document.fromJson(doc);

      await _saveDocumentLocally(document);

      return document;
    } catch (e) {
      // Offline update - update local document
      final existingEntity = await _isar.getDocumentById(id);
      if (existingEntity != null) {
        final existingDoc = _entityToDocument(existingEntity);
        final updatedDoc = existingDoc.copyWith(
          title: title ?? existingDoc.title,
          icon: icon ?? existingDoc.icon,
          content: content ?? existingDoc.content,
          updatedAt: DateTime.now(),
          isSynced: false,
          isDirty: true,
        );
        await _saveDocumentLocally(updatedDoc);
        return updatedDoc;
      }
      rethrow;
    }
  }

  @override
  Future<void> deleteDocument(String id) async {
    try {
      await _apiClient.delete('/documents/$id');
      await _isar.deleteDocument(id);
    } catch (e) {
      // Offline delete - mark as dirty with delete operation
      final existingEntity = await _isar.getDocumentById(id);
      if (existingEntity != null) {
        final existingDoc = _entityToDocument(existingEntity);
        final deletedDoc = existingDoc.copyWith(
          isArchived: true,
          updatedAt: DateTime.now(),
          isSynced: false,
          isDirty: true,
        );
        await _saveDocumentLocally(deletedDoc);
      }
    }
  }

  @override
  Future<DocumentListResult> listDocuments({
    required String spaceId,
    String? parentId,
    int limit = 20,
    int offset = 0,
  }) async {
    try {
      final queryParams = <String, dynamic>{
        'limit': limit,
        'offset': offset,
      };
      if (parentId != null) {
        queryParams['parent_id'] = parentId;
      }

      final response = await _apiClient.get(
        '/spaces/$spaceId/documents',
        queryParams: queryParams,
      );

      final data = response.data['data'] as Map<String, dynamic>;
      final documentsJson = data['documents'] as List;
      final total = data['total'] as int;

      final documents = documentsJson
          .map((doc) => Document.fromJson(doc as Map<String, dynamic>))
          .toList();

      // Save to local storage
      for (final doc in documents) {
        await _saveDocumentLocally(doc);
      }

      return DocumentListResult(
        documents: documents,
        total: total,
        limit: limit,
        offset: offset,
      );
    } catch (e) {
      // Fallback to local storage
      final localEntities = parentId != null
          ? await _isar.getDocumentsByParent(parentId)
          : await _isar.getDocumentsBySpace(spaceId);

      final documents = localEntities
          .map((entity) => _entityToDocument(entity))
          .toList();

      final start = offset.clamp(0, documents.length);
      final end = (offset + limit).clamp(0, documents.length);

      return DocumentListResult(
        documents: documents.sublist(start, end),
        total: documents.length,
        limit: limit,
        offset: offset,
      );
    }
  }

  @override
  Future<DocumentListResult> getDocumentChildren(String parentId) async {
    try {
      final response = await _apiClient.get('/documents/$parentId/children');
      final data = response.data['data'] as Map<String, dynamic>;
      final documentsJson = data['documents'] as List;
      final total = data['total'] as int;

      final documents = documentsJson
          .map((doc) => Document.fromJson(doc as Map<String, dynamic>))
          .toList();

      return DocumentListResult(
        documents: documents,
        total: total,
        limit: documents.length,
        offset: 0,
      );
    } catch (e) {
      // Fallback to local storage
      final localEntities = await _isar.getDocumentsByParent(parentId);
      final documents = localEntities
          .map((entity) => _entityToDocument(entity))
          .toList();
      return DocumentListResult(
        documents: documents,
        total: documents.length,
        limit: documents.length,
        offset: 0,
      );
    }
  }

  @override
  Future<List<Document>> getDocumentPath(String documentId) async {
    try {
      final response = await _apiClient.get('/documents/$documentId/path');
      final data = response.data['data'] as Map<String, dynamic>;
      final pathJson = data['path'] as List;

      return pathJson
          .map((item) => Document.fromJson(item as Map<String, dynamic>))
          .toList();
    } catch (e) {
      // Fallback - return just the document itself
      final entity = await _isar.getDocumentById(documentId);
      if (entity != null) {
        return [_entityToDocument(entity)];
      }
      return [];
    }
  }

  @override
  Future<VersionListResult> getDocumentVersions(
    String documentId, {
    int limit = 20,
    int offset = 0,
  }) async {
    try {
      final response = await _apiClient.get(
        '/documents/$documentId/versions',
        queryParams: {'limit': limit, 'offset': offset},
      );

      final data = response.data['data'] as Map<String, dynamic>;
      final versionsJson = data['versions'] as List;
      final total = data['total'] as int;

      final versions = versionsJson.map((v) {
        final json = v as Map<String, dynamic>;
        return DocumentVersion(
          id: json['id'] as String,
          documentId: json['document_id'] as String,
          versionNumber: json['version_number'] as int,
          title: json['title'] as String,
          content: json['content'] as Map<String, dynamic>,
          createdBy: json['created_by'] as String,
          createdAt: _parseDateTime(json['created_at'] as String?),
          changeSummary: json['change_summary'] as String?,
        );
      }).toList();

      return VersionListResult(versions: versions, total: total);
    } catch (e) {
      return VersionListResult(versions: [], total: 0);
    }
  }

  @override
  Future<DocumentVersion> createVersion({
    required String documentId,
    required Map<String, dynamic> content,
    required String title,
    String? changeSummary,
  }) async {
    final response = await _apiClient.post(
      '/documents/$documentId/versions',
      data: {
        'content': content,
        'title': title,
        'change_summary': changeSummary,
      },
    );

    final json = response.data['data']['version'] as Map<String, dynamic>;
    return DocumentVersion(
      id: json['id'] as String,
      documentId: json['document_id'] as String,
      versionNumber: json['version_number'] as int,
      title: json['title'] as String,
      content: json['content'] as Map<String, dynamic>,
      createdBy: json['created_by'] as String,
      createdAt: _parseDateTime(json['created_at'] as String?),
      changeSummary: json['change_summary'] as String?,
    );
  }

  @override
  Future<Document> restoreVersion(String documentId, int versionNumber) async {
    final response = await _apiClient.post(
      '/documents/$documentId/versions/$versionNumber/restore',
    );

    final json = response.data['data']['document'] as Map<String, dynamic>;
    final document = Document.fromJson(json);

    await _saveDocumentLocally(document);

    return document;
  }

  @override
  Future<VersionDiff> getVersionDiff(
    String documentId,
    int fromVersion,
    int toVersion,
  ) async {
    final response = await _apiClient.get(
      '/documents/$documentId/versions/diff',
      queryParams: {'from': fromVersion, 'to': toVersion},
    );

    final json = response.data['data'] as Map<String, dynamic>;
    return VersionDiff(
      fromVersion: json['from_version'] as int,
      toVersion: json['to_version'] as int,
      fromContent: json['from_content'] as Map<String, dynamic>,
      toContent: json['to_content'] as Map<String, dynamic>,
    );
  }

  // Helper methods

  Future<void> _saveDocumentLocally(Document document) async {
    final entity = _documentToEntity(document);
    await _isar.saveDocument(entity);
  }

  DocumentEntity _documentToEntity(Document document) {
    return DocumentEntity()
      ..uuid = document.id
      ..spaceId = document.spaceId
      ..parentId = document.parentId
      ..title = document.title
      ..icon = document.icon
      ..content = document.content
      ..contentSize = document.contentSize
      ..isArchived = document.isArchived
      ..createdBy = document.createdBy
      ..lastEditedBy = document.lastEditedBy
      ..createdAt = document.createdAt
      ..updatedAt = document.updatedAt
      ..isSynced = document.isSynced
      ..isDirty = document.isDirty
      ..lastSyncedAt = document.lastSyncedAt;
  }

  Document _entityToDocument(DocumentEntity entity) {
    return Document(
      id: entity.uuid,
      spaceId: entity.spaceId,
      parentId: entity.parentId,
      title: entity.title,
      icon: entity.icon,
      content: entity.content,
      contentSize: entity.contentSize,
      isArchived: entity.isArchived,
      createdBy: entity.createdBy,
      lastEditedBy: entity.lastEditedBy,
      createdAt: entity.createdAt,
      updatedAt: entity.updatedAt,
      isSynced: entity.isSynced,
      isDirty: entity.isDirty,
      lastSyncedAt: entity.lastSyncedAt,
    );
  }

  DateTime _parseDateTime(String? dateTimeString) {
    if (dateTimeString == null || dateTimeString.isEmpty) {
      return DateTime.now();
    }
    try {
      return DateTime.parse(dateTimeString);
    } catch (e) {
      return DateTime.now();
    }
  }
}

/// Provider for DocumentRepository
final documentRepositoryProvider = Provider<DocumentRepository>((ref) {
  final apiClient = ref.watch(apiClientProvider);
  // final isar = ref.watch(isarProvider);
  final isar = IsarDatabase(null);
  return DocumentRepositoryImpl(apiClient, isar);
});
