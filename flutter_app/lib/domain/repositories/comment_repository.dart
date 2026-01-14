import 'package:miniwiki/domain/entities/comment.dart';

/// Repository interface for comment operations
///
/// This interface defines the contract for comment data access,
/// supporting both online (API) and offline (local database) operations.
abstract class CommentRepository {
  /// List comments for a document
  ///
  /// [documentId] The ID of the document
  /// [parentId] Filter by parent comment ID (null for top-level comments)
  /// [limit] Maximum number of comments to return (default 50)
  /// [offset] Number of comments to skip (for pagination)
  ///
  /// Returns a list of comments with pagination info
  Future<CommentListResult> listComments({
    required String documentId,
    String? parentId,
    int limit = 50,
    int offset = 0,
  });

  /// Get a comment by ID
  ///
  /// [id] The ID of the comment to retrieve
  ///
  /// Returns the comment if found, null otherwise
  Future<Comment?> getComment(String id);

  /// Create a new comment
  ///
  /// [documentId] The ID of the document to add the comment to
  /// [content] The content of the comment
  /// [parentId] The ID of the parent comment (optional, for threaded replies)
  ///
  /// Returns the created comment
  Future<Comment> createComment({
    required String documentId,
    required String content,
    String? parentId,
  });

  /// Update a comment
  ///
  /// [id] The ID of the comment to update
  /// [content] The new content
  ///
  /// Returns the updated comment
  Future<Comment> updateComment({
    required String id,
    required String content,
  });

  /// Resolve a comment
  ///
  /// [id] The ID of the comment to resolve
  ///
  /// Returns the resolved comment
  Future<Comment> resolveComment(String id);

  /// Unresolve a comment
  ///
  /// [id] The ID of the comment to unresolve
  ///
  /// Returns the unresolved comment
  Future<Comment> unresolveComment(String id);

  /// Delete a comment
  ///
  /// [id] The ID of the comment to delete
  ///
  /// This also deletes all child comments (replies)
  Future<void> deleteComment(String id);
}

/// Result of listing comments
class CommentListResult {
  /// List of comments in this result
  final List<Comment> comments;

  /// Total number of comments matching the query
  final int total;

  /// Maximum number of comments requested
  final int limit;

  /// Number of comments skipped
  final int offset;

  const CommentListResult({
    required this.comments,
    required this.total,
    required this.limit,
    required this.offset,
  });

  /// Whether there are more comments to load
  bool get hasMore => offset + comments.length < total;
}
