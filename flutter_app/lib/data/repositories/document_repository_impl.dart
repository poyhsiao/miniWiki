import 'package:riverpod/riverpod.dart';
import 'package:miniwiki/core/network/api_client.dart';
import 'package:miniwiki/data/models/document_entity.dart';
import 'package:miniwiki/data/datasources/isar_datasource.dart';

abstract class DocumentRepository {
  Future<List<DocumentEntity>> getDocuments(String spaceId);
  Future<DocumentEntity?> getDocument(String id);
  Future<DocumentEntity> createDocument(DocumentEntity document);
  Future<DocumentEntity> updateDocument(DocumentEntity document);
  Future<void> deleteDocument(String id);
  Future<List<DocumentEntity>> getDirtyDocuments();
  Future<void> markDocumentSynced(String id);
}

class DocumentRepositoryImpl implements DocumentRepository {
  final ApiClient _apiClient;
  final IsarDatabase _isar;

  DocumentRepositoryImpl(this._apiClient, this._isar);

  @override
  Future<List<DocumentEntity>> getDocuments(String spaceId) async {
    try {
      final response = await _apiClient.get('/spaces/$spaceId/documents');
      final data = response.data as List;

      final documents = data.map((doc) {
        final entity = DocumentEntity()
          ..uuid = doc['id'] as String
          ..spaceId = doc['space_id'] as String
          ..parentId = doc['parent_id'] as String?
          ..title = doc['title'] as String
          ..icon = doc['icon'] as String?
          ..content = doc['content'] as Map<String, dynamic>
          ..contentSize = doc['content_size'] as int
          ..isArchived = doc['is_archived'] as bool
          ..createdBy = doc['created_by'] as String
          ..lastEditedBy = doc['last_edited_by'] as String
          ..createdAt = DateTime.parse(doc['created_at'] as String)
          ..updatedAt = DateTime.parse(doc['updated_at'] as String)
          ..isSynced = true
          ..isDirty = false;

        return entity;
      }).toList();

      // Save to local storage
      for (final doc in documents) {
        await _isar.saveDocument(doc);
      }

      return documents;
    } catch (e) {
      // Fallback to local storage if offline
      return _isar.getDocumentsBySpace(spaceId);
    }
  }

  @override
  Future<DocumentEntity?> getDocument(String id) async {
    try {
      final response = await _apiClient.get('/documents/$id');
      final doc = response.data as Map<String, dynamic>;

      final entity = DocumentEntity()
        ..uuid = doc['id'] as String
        ..spaceId = doc['space_id'] as String
        ..parentId = doc['parent_id'] as String?
        ..title = doc['title'] as String
        ..icon = doc['icon'] as String?
        ..content = doc['content'] as Map<String, dynamic>
        ..contentSize = doc['content_size'] as int
        ..isArchived = doc['is_archived'] as bool
        ..createdBy = doc['created_by'] as String
        ..lastEditedBy = doc['last_edited_by'] as String
        ..createdAt = DateTime.parse(doc['created_at'] as String)
        ..updatedAt = DateTime.parse(doc['updated_at'] as String)
        ..isSynced = true
        ..isDirty = false;

      await _isar.saveDocument(entity);

      return entity;
    } catch (e) {
      return _isar.getDocumentById(id);
    }
  }

  @override
  Future<DocumentEntity> createDocument(DocumentEntity document) async {
    try {
      final response =
          await _apiClient.post('/spaces/${document.spaceId}/documents', data: {
        'title': document.title,
        'icon': document.icon,
        'parent_id': document.parentId,
        'content': document.content,
      });

      final doc = response.data as Map<String, dynamic>;

      final entity = DocumentEntity()
        ..uuid = doc['id'] as String
        ..spaceId = doc['space_id'] as String
        ..parentId = doc['parent_id'] as String?
        ..title = doc['title'] as String
        ..icon = doc['icon'] as String?
        ..content = doc['content'] as Map<String, dynamic>
        ..contentSize = doc['content_size'] as int
        ..isArchived = doc['is_archived'] as bool
        ..createdBy = doc['created_by'] as String
        ..lastEditedBy = doc['last_edited_by'] as String
        ..createdAt = DateTime.parse(doc['created_at'] as String)
        ..updatedAt = DateTime.parse(doc['updated_at'] as String)
        ..isSynced = true
        ..isDirty = false;

      await _isar.saveDocument(entity);

      return entity;
    } catch (e) {
      // Offline create - mark as dirty
      document.isDirty = true;
      document.isSynced = false;
      await _isar.saveDocument(document);
      return document;
    }
  }

  @override
  Future<DocumentEntity> updateDocument(DocumentEntity document) async {
    try {
      final response =
          await _apiClient.patch('/documents/${document.uuid}', data: {
        'title': document.title,
        'icon': document.icon,
        'parent_id': document.parentId,
        'content': document.content,
      });

      final doc = response.data as Map<String, dynamic>;

      final entity = DocumentEntity()
        ..uuid = doc['id'] as String
        ..spaceId = doc['space_id'] as String
        ..parentId = doc['parent_id'] as String?
        ..title = doc['title'] as String
        ..icon = doc['icon'] as String?
        ..content = doc['content'] as Map<String, dynamic>
        ..contentSize = doc['content_size'] as int
        ..isArchived = doc['is_archived'] as bool
        ..createdBy = doc['created_by'] as String
        ..lastEditedBy = doc['last_edited_by'] as String
        ..createdAt = DateTime.parse(doc['created_at'] as String)
        ..updatedAt = DateTime.parse(doc['updated_at'] as String)
        ..isSynced = true
        ..isDirty = false;

      await _isar.saveDocument(entity);

      return entity;
    } catch (e) {
      // Offline update - mark as dirty
      document.isDirty = true;
      document.isSynced = false;
      await _isar.saveDocument(document);
      return document;
    }
  }

  @override
  Future<void> deleteDocument(String id) async {
    try {
      await _apiClient.delete('/documents/$id');
    } catch (e) {
      // Offline delete - mark as dirty with delete operation
    }
  }

  @override
  Future<List<DocumentEntity>> getDirtyDocuments() async {
    return _isar.getDirtyDocuments();
  }

  @override
  Future<void> markDocumentSynced(String id) async {
    await _isar.markDocumentSynced(id);
  }
}

final documentRepositoryProvider = Provider<DocumentRepository>((ref) {
  final apiClient = ref.watch(apiClientProvider);
  // final isar = ref.watch(isarProvider);
  final isar = IsarDatabase(null);
  return DocumentRepositoryImpl(apiClient, isar);
});
