import 'dart:async';

import 'package:miniwiki/core/network/api_client.dart';
import 'package:miniwiki/domain/entities/comment.dart';
import 'package:miniwiki/domain/repositories/comment_repository.dart';
import 'package:riverpod/riverpod.dart';

/// Implementation of CommentRepository that handles API operations
/// Offline storage is handled separately by the sync service
class CommentRepositoryImpl implements CommentRepository {
  final ApiClient _apiClient;

  CommentRepositoryImpl(this._apiClient);

  @override
  Future<CommentListResult> listComments({
    required String documentId,
    String? parentId,
    int limit = 50,
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
      '/documents/$documentId/comments',
      queryParams: queryParams,
    );

    final data = response.data['data'] as Map<String, dynamic>;
    final commentsJson = data['comments'] as List;
    final total = data['total'] as int;

    final comments = commentsJson
        .map((comment) => Comment.fromJson(comment as Map<String, dynamic>))
        .toList();

    return CommentListResult(
      comments: comments,
      total: total,
      limit: limit,
      offset: offset,
    );
  }

  @override
  Future<Comment?> getComment(String id) async {
    try {
      final response = await _apiClient.get('/comments/$id');
      final data = response.data['data'] as Map<String, dynamic>;
      return Comment.fromJson(data);
    } on NotFoundError catch (_) {
      return null;
    } catch (e) {
      return null;
    }
  }

  @override
  Future<Comment> createComment({
    required String documentId,
    required String content,
    String? parentId,
  }) async {
    final requestData = <String, dynamic>{
      'content': content,
    };
    if (parentId != null) {
      requestData['parent_id'] = parentId;
    }

    final response = await _apiClient.post(
      '/documents/$documentId/comments',
      data: requestData,
    );

    final data = response.data['data']['comment'] as Map<String, dynamic>;
    return Comment.fromJson(data);
  }

  @override
  Future<Comment> updateComment({
    required String id,
    required String content,
  }) async {
    final response = await _apiClient.patch('/comments/$id', data: {
      'content': content,
    });
    final data = response.data['data'] as Map<String, dynamic>;
    return Comment.fromJson(data);
  }

  @override
  Future<Comment> resolveComment(String id) async {
    final response = await _apiClient.post('/comments/$id/resolve');
    final data = response.data['data'] as Map<String, dynamic>;
    return Comment.fromJson(data);
  }

  @override
  Future<Comment> unresolveComment(String id) async {
    final response = await _apiClient.post('/comments/$id/unresolve');
    final data = response.data['data'] as Map<String, dynamic>;
    return Comment.fromJson(data);
  }

  @override
  Future<void> deleteComment(String id) async {
    await _apiClient.delete('/comments/$id');
  }
}

/// Provider for CommentRepositoryImpl
final commentRepositoryProvider = Provider<CommentRepository>((ref) {
  final apiClient = ref.watch(apiClientProvider);
  return CommentRepositoryImpl(apiClient);
});
