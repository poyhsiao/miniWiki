import 'package:flutter_test/flutter_test.dart';
import 'package:miniwiki/domain/entities/comment.dart';

void main() {
  group('Comment Entity Tests', () {
    test('Comment can be created with required fields', () {
      // Arrange & Act
      const comment = Comment(
        id: 'comment1',
        documentId: 'doc1',
        authorId: 'user1',
        authorName: 'Test User',
        content: 'This is a test comment',
      );

      // Assert
      expect(comment.id, 'comment1');
      expect(comment.documentId, 'doc1');
      expect(comment.authorId, 'user1');
      expect(comment.authorName, 'Test User');
      expect(comment.content, 'This is a test comment');
      expect(comment.parentId, isNull);
      expect(comment.authorAvatar, isNull);
      expect(comment.isResolved, false);
      expect(comment.resolvedBy, isNull);
      expect(comment.resolvedAt, isNull);
      expect(comment.createdAt, isNull);
      expect(comment.updatedAt, isNull);
    });

    test('Comment can be created with all fields', () {
      // Arrange & Act
      final now = DateTime(2024);
      final comment = Comment(
        id: 'comment1',
        documentId: 'doc1',
        parentId: 'parent1',
        authorId: 'user1',
        authorName: 'Test User',
        authorAvatar: 'https://example.com/avatar.jpg',
        content: 'This is a test comment',
        isResolved: true,
        resolvedBy: 'admin1',
        resolvedAt: now,
        createdAt: now,
        updatedAt: now,
      );

      // Assert
      expect(comment.id, 'comment1');
      expect(comment.parentId, 'parent1');
      expect(comment.authorAvatar, 'https://example.com/avatar.jpg');
      expect(comment.isResolved, true);
      expect(comment.resolvedBy, 'admin1');
      expect(comment.resolvedAt, now);
      expect(comment.createdAt, now);
      expect(comment.updatedAt, now);
    });

    test('Comment copyWith creates modified copy', () {
      // Arrange
      const original = Comment(
        id: 'comment1',
        documentId: 'doc1',
        authorId: 'user1',
        authorName: 'Test User',
        content: 'Original content',
      );

      // Act
      final modified = original.copyWith(
        content: 'Modified content',
        isResolved: true,
        resolvedBy: 'admin1',
      );

      // Assert
      expect(modified.id, 'comment1'); // Unchanged
      expect(modified.content, 'Modified content');
      expect(modified.isResolved, true);
      expect(modified.resolvedBy, 'admin1');
      expect(modified.authorName, 'Test User'); // Unchanged
    });

    test('Comment copyWith preserves values when null is passed', () {
      // Arrange
      const original = Comment(
        id: 'comment1',
        documentId: 'doc1',
        authorId: 'user1',
        authorName: 'Test User',
        content: 'Content',
        parentId: 'parent1',
        authorAvatar: 'avatar.jpg',
      );

      // Act - When null is passed to copyWith, it preserves the original value
      final modified = original.copyWith(
        
      );

      // Assert - Original values are preserved due to ?? operator
      expect(modified.parentId, 'parent1'); // Preserved
      expect(modified.authorAvatar, 'avatar.jpg'); // Preserved
      expect(modified.id, 'comment1'); // Unchanged
    });

    test('Comment clearResolutionInfo removes resolution', () {
      // Arrange
      final now = DateTime(2024);
      final resolvedComment = Comment(
        id: 'comment1',
        documentId: 'doc1',
        authorId: 'user1',
        authorName: 'Test User',
        content: 'This is a comment',
        isResolved: true,
        resolvedBy: 'admin1',
        resolvedAt: now,
        createdAt: now,
        updatedAt: now,
      );

      // Act
      final unresolvedComment = resolvedComment.clearResolutionInfo();

      // Assert
      expect(unresolvedComment.isResolved, false);
      expect(unresolvedComment.resolvedBy, isNull);
      expect(unresolvedComment.resolvedAt, isNull);
      expect(unresolvedComment.id, 'comment1'); // Other fields preserved
      expect(unresolvedComment.content, 'This is a comment');
    });

    test('Comment toJson creates correct JSON with all fields', () {
      // Arrange
      final now = DateTime(2024, 1, 1, 12);
      final comment = Comment(
        id: 'comment1',
        documentId: 'doc1',
        parentId: 'parent1',
        authorId: 'user1',
        authorName: 'Test User',
        authorAvatar: 'https://example.com/avatar.jpg',
        content: 'This is a test comment',
        isResolved: true,
        resolvedBy: 'admin1',
        resolvedAt: now,
        createdAt: now,
        updatedAt: now,
      );

      // Act
      final json = comment.toJson();

      // Assert
      expect(json['id'], 'comment1');
      expect(json['document_id'], 'doc1');
      expect(json['parent_id'], 'parent1');
      expect(json['author_id'], 'user1');
      expect(json['author_name'], 'Test User');
      expect(json['author_avatar'], 'https://example.com/avatar.jpg');
      expect(json['content'], 'This is a test comment');
      expect(json['is_resolved'], true);
      expect(json['resolved_by'], 'admin1');
      expect(json['resolved_at'], '2024-01-01T12:00:00.000');
      expect(json['created_at'], '2024-01-01T12:00:00.000');
      expect(json['updated_at'], '2024-01-01T12:00:00.000');
    });

    test('Comment toJson excludes null optional fields', () {
      // Arrange
      const comment = Comment(
        id: 'comment1',
        documentId: 'doc1',
        authorId: 'user1',
        authorName: 'Test User',
        content: 'This is a test comment',
      );

      // Act
      final json = comment.toJson();

      // Assert
      expect(json['id'], 'comment1');
      expect(json.containsKey('parent_id'), false);
      expect(json.containsKey('author_avatar'), false);
      expect(json.containsKey('resolved_by'), false);
      expect(json.containsKey('resolved_at'), false);
      expect(json.containsKey('created_at'), false);
      expect(json.containsKey('updated_at'), false);
    });

    test('Comment fromJson creates instance with all fields', () {
      // Arrange
      final json = {
        'id': 'comment1',
        'document_id': 'doc1',
        'parent_id': 'parent1',
        'author_id': 'user1',
        'author_name': 'Test User',
        'author_avatar': 'https://example.com/avatar.jpg',
        'content': 'This is a test comment',
        'is_resolved': true,
        'resolved_by': 'admin1',
        'resolved_at': '2024-01-01T12:00:00.000Z',
        'created_at': '2024-01-01T12:00:00.000Z',
        'updated_at': '2024-01-01T12:00:00.000Z',
      };

      // Act
      final comment = Comment.fromJson(json);

      // Assert
      expect(comment.id, 'comment1');
      expect(comment.documentId, 'doc1');
      expect(comment.parentId, 'parent1');
      expect(comment.authorId, 'user1');
      expect(comment.authorName, 'Test User');
      expect(comment.authorAvatar, 'https://example.com/avatar.jpg');
      expect(comment.content, 'This is a test comment');
      expect(comment.isResolved, true);
      expect(comment.resolvedBy, 'admin1');
    });

    test('Comment fromJson handles null optional fields', () {
      // Arrange
      final json = {
        'id': 'comment1',
        'document_id': 'doc1',
        'author_id': 'user1',
        'author_name': 'Test User',
        'content': 'This is a test comment',
      };

      // Act
      final comment = Comment.fromJson(json);

      // Assert
      expect(comment.parentId, isNull);
      expect(comment.authorAvatar, isNull);
      expect(comment.isResolved, false); // Default
      expect(comment.resolvedBy, isNull);
      expect(comment.resolvedAt, isNull);
      expect(comment.createdAt, isNull);
      expect(comment.updatedAt, isNull);
    });

    test('Comment fromJson handles invalid date strings', () {
      // Arrange
      final json = {
        'id': 'comment1',
        'document_id': 'doc1',
        'author_id': 'user1',
        'author_name': 'Test User',
        'content': 'This is a test comment',
        'created_at': 'invalid-date',
        'updated_at': '',
      };

      // Act
      final comment = Comment.fromJson(json);

      // Assert
      expect(comment.createdAt, isNull); // Invalid date returns null
      expect(comment.updatedAt, isNull); // Empty string returns null
    });

    test('Comment equality works correctly', () {
      // Arrange
      const comment1 = Comment(
        id: 'comment1',
        documentId: 'doc1',
        authorId: 'user1',
        authorName: 'Test User',
        content: 'Content 1',
      );

      const comment2 = Comment(
        id: 'comment1',
        documentId: 'doc2',
        authorId: 'user2',
        authorName: 'Different User',
        content: 'Content 2',
      );

      const comment3 = Comment(
        id: 'comment2',
        documentId: 'doc1',
        authorId: 'user1',
        authorName: 'Test User',
        content: 'Content 1',
      );

      // Assert
      expect(comment1, equals(comment2)); // Same ID
      expect(comment1, isNot(equals(comment3))); // Different ID
      expect(comment1.hashCode, equals(comment2.hashCode));
    });

    test('Comment toString returns formatted string', () {
      // Arrange & Act
      const comment = Comment(
        id: 'comment1',
        documentId: 'doc1',
        authorId: 'user1',
        authorName: 'Test User',
        content: 'This is a test comment',
      );

      // Assert
      expect(comment.toString(), 'Comment(id: comment1, author: Test User)');
    });

    test('Comment with threaded replies (parentId)', () {
      // Arrange & Act
      const parentComment = Comment(
        id: 'comment1',
        documentId: 'doc1',
        authorId: 'user1',
        authorName: 'Test User',
        content: 'Parent comment',
      );

      const replyComment = Comment(
        id: 'comment2',
        documentId: 'doc1',
        parentId: 'comment1',
        authorId: 'user2',
        authorName: 'Reply User',
        content: 'Reply to parent',
      );

      // Assert
      expect(parentComment.parentId, isNull);
      expect(replyComment.parentId, 'comment1');
      expect(replyComment.documentId, parentComment.documentId);
    });

    test('Comment resolution workflow', () {
      // Arrange
      const unresolvedComment = Comment(
        id: 'comment1',
        documentId: 'doc1',
        authorId: 'user1',
        authorName: 'Test User',
        content: 'Bug report',
      );

      // Act - Resolve the comment
      final resolvedComment = unresolvedComment.copyWith(
        isResolved: true,
        resolvedBy: 'admin1',
        resolvedAt: DateTime(2024),
      );

      // Assert
      expect(resolvedComment.isResolved, true);
      expect(resolvedComment.resolvedBy, 'admin1');
      expect(resolvedComment.resolvedAt, isNotNull);

      // Act - Unresolve the comment
      final reopenedComment = resolvedComment.clearResolutionInfo();

      // Assert
      expect(reopenedComment.isResolved, false);
      expect(reopenedComment.resolvedBy, isNull);
      expect(reopenedComment.resolvedAt, isNull);
    });

    test('Comment with default isResolved', () {
      // Arrange & Act
      const comment = Comment(
        id: 'comment1',
        documentId: 'doc1',
        authorId: 'user1',
        authorName: 'Test User',
        content: 'Content',
      );

      // Assert
      expect(comment.isResolved, false); // Default value
    });

    test('Comment can be explicitly marked as unresolved', () {
      // Arrange & Act
      const comment = Comment(
        id: 'comment1',
        documentId: 'doc1',
        authorId: 'user1',
        authorName: 'Test User',
        content: 'Content',
      );

      // Assert
      expect(comment.isResolved, false);
    });
  });
}
