import 'dart:async';

import 'package:miniwiki/core/network/api_client.dart';
import 'package:miniwiki/domain/entities/document.dart';
import 'package:miniwiki/domain/repositories/document_repository.dart';
import 'package:riverpod/riverpod.dart';

/// Implementation of DocumentRepository that handles API operations
/// Offline storage is handled separately by the sync service
class DocumentRepositoryImpl implements DocumentRepository {
  final ApiClient _apiClient;

  DocumentRepositoryImpl(this._apiClient);

  @override
  Future<Document> createDocument({
    required String spaceId,
    required String? parentId,
    required String title,
    required String? icon,
    required Map<String, dynamic> content,
  }) async {
    final response = await _apiClient.post('/spaces/$spaceId/documents', data: {
      'title': title,
      'icon': icon,
      'parent_id': parentId,
      'content': content,
    });

    final doc = response.data['data']['document'] as Map<String, dynamic>;
    return Document.fromJson(doc);
  }

  @override
  Future<Document> getDocument(String id) async {
    final response = await _apiClient.get('/documents/$id');
    final doc = response.data['data'] as Map<String, dynamic>;
    return Document.fromJson(doc);
  }

  @override
  Future<Document> updateDocument({
    required String id,
    String? title,
    String? icon,
    Map<String, dynamic>? content,
  }) async {
    final requestData = <String, dynamic>{};
    if (title != null) requestData['title'] = title;
    if (icon != null) requestData['icon'] = icon;
    if (content != null) requestData['content'] = content;

    final response =
        await _apiClient.patch('/documents/$id', data: requestData);
    final doc = response.data['data']['document'] as Map<String, dynamic>;
    return Document.fromJson(doc);
  }

  @override
  Future<void> deleteDocument(String id) async {
    await _apiClient.delete('/documents/$id');
  }

  @override
  Future<DocumentListResult> listDocuments({
    required String spaceId,
    String? parentId,
    int limit = 20,
    int offset = 0,
  }) async {
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

    return DocumentListResult(
      documents: documents,
      total: total,
      limit: limit,
      offset: offset,
    );
  }

  @override
  Future<DocumentListResult> getDocumentChildren(String parentId) async {
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
  }

  @override
  Future<List<Document>> getDocumentPath(String documentId) async {
    final response = await _apiClient.get('/documents/$documentId/path');
    final data = response.data['data'] as Map<String, dynamic>;
    final pathJson = data['path'] as List;

    final result = <Document>[];
    for (final item in pathJson) {
      final jsonMap = <String, dynamic>{};
      (item as Map).forEach((key, value) {
        if (value is Map) {
          jsonMap[key.toString()] = Map<String, dynamic>.from(value);
        } else {
          jsonMap[key.toString()] = value;
        }
      });
      result.add(Document.fromJson(jsonMap));
    }
    return result;
  }

  @override
  Future<VersionListResult> getDocumentVersions(
    String documentId, {
    int limit = 20,
    int offset = 0,
  }) async {
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
    return Document.fromJson(json);
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
  return DocumentRepositoryImpl(apiClient);
});
