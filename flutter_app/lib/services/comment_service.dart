import 'package:miniwiki/domain/entities/comment.dart';
import 'package:miniwiki/domain/repositories/comment_repository.dart';

/// Service class for comment-related business logic
///
/// This service wraps the repository and provides higher-level
/// operations for comment management in the miniWiki app.
class CommentService {
  final CommentRepository _repository;

  CommentService(this._repository);

  /// Lists comments for a document
  ///
  /// [documentId] The ID of the document
  /// [parentId] Filter by parent comment ID (null for top-level)
  /// [limit] Maximum number of comments to return
  /// [offset] Number of comments to skip
  ///
  /// Returns a list of comments with pagination info
  Future<CommentListResult> listComments({
    required String documentId,
    String? parentId,
    int limit = 50,
    int offset = 0,
  }) =>
      _repository.listComments(
        documentId: documentId,
        parentId: parentId,
        limit: limit,
        offset: offset,
      );

  /// Gets a comment by ID
  ///
  /// [id] The comment ID
  ///
  /// Returns the comment if found, null otherwise
  Future<Comment?> getComment(String id) => _repository.getComment(id);

  /// Creates a new comment on a document
  ///
  /// [documentId] The ID of the document
  /// [content] The content of the comment
  /// [parentId] Optional parent comment ID for threaded replies
  ///
  /// Returns the created comment
  Future<Comment> createComment({
    required String documentId,
    required String content,
    String? parentId,
  }) =>
      _repository.createComment(
        documentId: documentId,
        content: _sanitizeContent(content),
        parentId: parentId,
      );

  /// Updates a comment's content
  ///
  /// [id] The comment ID
  /// [content] The new content
  ///
  /// Returns the updated comment
  Future<Comment> updateComment({
    required String id,
    required String content,
  }) =>
      _repository.updateComment(
        id: id,
        content: _sanitizeContent(content),
      );

  /// Resolves a comment
  ///
  /// [id] The comment ID
  ///
  /// Returns the resolved comment
  Future<Comment> resolveComment(String id) => _repository.resolveComment(id);

  /// Unresolves a comment
  ///
  /// [id] The comment ID
  ///
  /// Returns the unresolved comment
  Future<Comment> unresolveComment(String id) =>
      _repository.unresolveComment(id);

  /// Deletes a comment
  ///
  /// [id] The comment ID
  ///
  /// This also deletes all child comments (replies)
  Future<void> deleteComment(String id) => _repository.deleteComment(id);

  /// Sanitizes comment content before sending to API
  ///
  /// Trims leading and trailing whitespace
  String _sanitizeContent(String content) => content.trim();
}
