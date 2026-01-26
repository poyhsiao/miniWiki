import 'package:flutter_test/flutter_test.dart';
import 'package:miniwiki/domain/entities/comment.dart';
import 'package:miniwiki/domain/repositories/comment_repository.dart';
import 'package:miniwiki/presentation/providers/comment_provider.dart';
import 'package:miniwiki/services/comment_service.dart';
import 'package:mocktail/mocktail.dart';

// Mock CommentService
class MockCommentService extends Mock implements CommentService {}

void main() {
  group('CommentListState Tests', () {
    test('CommentListState can be created with default values', () {
      // Arrange & Act
      const state = CommentListState(
        documentId: 'doc1',
      );

      // Assert
      expect(state.documentId, 'doc1');
      expect(state.comments, isEmpty);
      expect(state.total, 0);
      expect(state.isLoading, false);
      expect(state.error, isNull);
      expect(state.parentId, isNull);
    });

    test('CommentListState can be created with all fields', () {
      // Arrange & Act
      final now = DateTime(2024);
      final comments = [
        Comment(
          id: 'comment1',
          documentId: 'doc1',
          authorId: 'user1',
          authorName: 'User 1',
          content: 'Comment 1',
          createdAt: now,
          updatedAt: now,
        ),
        Comment(
          id: 'comment2',
          documentId: 'doc1',
          authorId: 'user2',
          authorName: 'User 2',
          content: 'Comment 2',
          createdAt: now,
          updatedAt: now,
        ),
      ];

      final state = CommentListState(
        documentId: 'doc1',
        comments: comments,
        total: 10,
        error: 'Test error',
        parentId: 'parent1',
      );

      // Assert
      expect(state.comments.length, 2);
      expect(state.total, 10);
      expect(state.error, 'Test error');
      expect(state.parentId, 'parent1');
    });

    test('CommentListState hasMore returns true when there are more comments', () {
      // Arrange & Act
      final state = CommentListState(
        documentId: 'doc1',
        comments: [
          Comment(
            id: 'c1',
            documentId: 'doc1',
            authorId: 'u1',
            authorName: 'User',
            content: 'Comment',
            createdAt: DateTime(2024),
            updatedAt: DateTime(2024),
          ),
        ],
        total: 5,
      );

      // Assert
      expect(state.hasMore, true);
    });

    test('CommentListState hasMore returns false when all comments loaded', () {
      // Arrange & Act
      final state = CommentListState(
        documentId: 'doc1',
        comments: [
          Comment(
            id: 'c1',
            documentId: 'doc1',
            authorId: 'u1',
            authorName: 'User',
            content: 'Comment',
            createdAt: DateTime(2024),
            updatedAt: DateTime(2024),
          ),
        ],
        total: 1,
      );

      // Assert
      expect(state.hasMore, false);
    });

    test('CommentListState topLevelComments filters correctly', () {
      // Arrange
      final now = DateTime(2024);
      final state = CommentListState(
        documentId: 'doc1',
        comments: [
          Comment(
            id: 'c1',
            documentId: 'doc1',
            authorId: 'u1',
            authorName: 'User 1',
            content: 'Top-level comment',
            createdAt: now,
            updatedAt: now,
          ),
          Comment(
            id: 'c2',
            documentId: 'doc1',
            parentId: 'c1', // Reply
            authorId: 'u2',
            authorName: 'User 2',
            content: 'Reply',
            createdAt: now,
            updatedAt: now,
          ),
        ],
      );

      // Act
      final topLevel = state.topLevelComments;

      // Assert
      expect(topLevel.length, 1);
      expect(topLevel.first.id, 'c1');
    });

    test('CommentListState getReplies returns child comments', () {
      // Arrange
      final now = DateTime(2024);
      final state = CommentListState(
        documentId: 'doc1',
        comments: [
          Comment(
            id: 'c1',
            documentId: 'doc1',
            authorId: 'u1',
            authorName: 'User 1',
            content: 'Parent',
            createdAt: now,
            updatedAt: now,
          ),
          Comment(
            id: 'c2',
            documentId: 'doc1',
            parentId: 'c1',
            authorId: 'u2',
            authorName: 'User 2',
            content: 'Reply 1',
            createdAt: now,
            updatedAt: now,
          ),
          Comment(
            id: 'c3',
            documentId: 'doc1',
            parentId: 'c1',
            authorId: 'u3',
            authorName: 'User 3',
            content: 'Reply 2',
            createdAt: now,
            updatedAt: now,
          ),
        ],
      );

      // Act
      final replies = state.getReplies('c1');

      // Assert
      expect(replies.length, 2);
      expect(replies[0].id, 'c2');
      expect(replies[1].id, 'c3');
    });

    test('CommentListState copyWith creates modified copy', () {
      // Arrange
      const original = CommentListState(
        documentId: 'doc1',
        total: 5,
      );

      // Act
      final modified = original.copyWith(
        isLoading: true,
        total: 10,
      );

      // Assert
      expect(modified.documentId, 'doc1'); // Unchanged
      expect(modified.isLoading, true);
      expect(modified.total, 10);
    });
  });

  group('CommentEditState Tests', () {
    test('CommentEditState can be created with default values', () {
      // Arrange & Act
      const state = CommentEditState();

      // Assert
      expect(state.content, '');
      expect(state.isSubmitting, false);
      expect(state.error, isNull);
    });

    test('CommentEditState can be created with custom values', () {
      // Arrange & Act
      const state = CommentEditState(
        content: 'Test comment',
        isSubmitting: true,
        error: 'Submit failed',
      );

      // Assert
      expect(state.content, 'Test comment');
      expect(state.isSubmitting, true);
      expect(state.error, 'Submit failed');
    });

    test('CommentEditState isValid returns false for empty content', () {
      // Arrange & Act
      const state = CommentEditState();

      // Assert
      expect(state.isValid, false);
    });

    test('CommentEditState isValid returns false for whitespace only', () {
      // Arrange & Act
      const state = CommentEditState(content: '   ');

      // Assert
      expect(state.isValid, false);
    });

    test('CommentEditState isValid returns true for non-empty content', () {
      // Arrange & Act
      const state = CommentEditState(content: 'Valid comment');

      // Assert
      expect(state.isValid, true);
    });

    test('CommentEditState copyWith creates modified copy', () {
      // Arrange
      const original = CommentEditState(
        content: 'Original',
      );

      // Act
      final modified = original.copyWith(
        content: 'Modified',
        isSubmitting: true,
      );

      // Assert
      expect(modified.content, 'Modified');
      expect(modified.isSubmitting, true);
      expect(modified.error, isNull); // Unchanged
    });
  });

  group('CommentListNotifier Tests', () {
    late CommentListNotifier notifier;
    late MockCommentService mockService;

    setUp(() {
      mockService = MockCommentService();
      notifier = CommentListNotifier(mockService, 'doc1');
    });

    test('CommentListNotifier initial state is correct', () {
      // Assert
      expect(notifier.state.documentId, 'doc1');
      expect(notifier.state.comments, isEmpty);
      expect(notifier.state.isLoading, false);
      expect(notifier.state.total, 0);
    });

    test('CommentListNotifier loadComments updates state on success', () async {
      // Arrange
      final now = DateTime(2024);
      final comments = [
        Comment(
          id: 'c1',
          documentId: 'doc1',
          authorId: 'u1',
          authorName: 'User 1',
          content: 'Comment 1',
          createdAt: now,
          updatedAt: now,
        ),
      ];
      when(() => mockService.listComments(
        documentId: any(named: 'documentId'),
      )).thenAnswer((_) async => CommentListResult(
        comments: comments,
        total: 1,
        limit: 50,
        offset: 0,
      ));

      // Act
      await notifier.loadComments();

      // Assert
      expect(notifier.state.comments.length, 1);
      expect(notifier.state.total, 1);
      expect(notifier.state.isLoading, false);
      expect(notifier.state.error, isNull);
      verify(() => mockService.listComments(
        documentId: any(named: 'documentId'),
      )).called(1);
    });

    test('CommentListNotifier loadComments handles parentId parameter', () async {
      // Arrange
      when(() => mockService.listComments(
        documentId: any(named: 'documentId'),
        parentId: 'parent1',
      )).thenAnswer((_) async => const CommentListResult(
        comments: [],
        total: 0,
        limit: 50,
        offset: 0,
      ));

      // Act
      await notifier.loadComments(parentId: 'parent1');

      // Assert
      expect(notifier.state.parentId, 'parent1');
      verify(() => mockService.listComments(
        documentId: any(named: 'documentId'),
        parentId: 'parent1',
      )).called(1);
    });

    test('CommentListNotifier loadComments handles custom limit', () async {
      // Arrange
      when(() => mockService.listComments(
        documentId: any(named: 'documentId'),
        limit: 100,
      )).thenAnswer((_) async => const CommentListResult(
        comments: [],
        total: 0,
        limit: 50,
        offset: 0,
      ));

      // Act
      await notifier.loadComments(limit: 100);

      // Assert
      verify(() => mockService.listComments(
        documentId: any(named: 'documentId'),
        limit: 100,
      )).called(1);
    });

    test('CommentListNotifier loadComments sets error on failure', () async {
      // Arrange
      when(() => mockService.listComments(
        documentId: any(named: 'documentId'),
        limit: any(named: 'limit'),
      )).thenThrow(Exception('Network error'));

      // Act
      await notifier.loadComments();

      // Assert
      expect(notifier.state.isLoading, false);
      expect(notifier.state.error, 'Exception: Network error');
    });

    test('CommentListNotifier loadAllComments calls loadComments', () async {
      // Arrange
      when(() => mockService.listComments(
        documentId: any(named: 'documentId'),
        limit: 100,
      )).thenAnswer((_) async => const CommentListResult(
        comments: [],
        total: 0,
        limit: 50,
        offset: 0,
      ));

      // Act
      await notifier.loadAllComments();

      // Assert
      verify(() => mockService.listComments(
        documentId: any(named: 'documentId'),
        limit: 100,
      )).called(1);
    });

    test('CommentListNotifier addComment adds comment to state', () {
      // Arrange
      final now = DateTime(2024);
      final comment = Comment(
        id: 'c1',
        documentId: 'doc1',
        authorId: 'u1',
        authorName: 'User 1',
        content: 'New comment',
        createdAt: now,
        updatedAt: now,
      );

      // Act
      notifier.addComment(comment);

      // Assert
      expect(notifier.state.comments.length, 1);
      expect(notifier.state.comments.first.id, 'c1');
      expect(notifier.state.total, 1);
    });

    test('CommentListNotifier updateComment replaces existing comment', () {
      // Arrange
      final now = DateTime(2024);
      final originalComment = Comment(
        id: 'c1',
        documentId: 'doc1',
        authorId: 'u1',
        authorName: 'User 1',
        content: 'Original content',
        createdAt: now,
        updatedAt: now,
      );

      notifier.addComment(originalComment);

      final updatedComment = originalComment.copyWith(
        content: 'Updated content',
        updatedAt: now.add(const Duration(days: 1)),
      );

      // Act
      notifier.updateComment(updatedComment);

      // Assert
      expect(notifier.state.comments.length, 1);
      expect(notifier.state.comments.first.content, 'Updated content');
      expect(notifier.state.comments.first.updatedAt,
          isNot(equals(now)));
    });

    test('CommentListNotifier removeComment removes comment and replies', () {
      // Arrange
      final now = DateTime(2024);
      final parentComment = Comment(
        id: 'c1',
        documentId: 'doc1',
        authorId: 'u1',
        authorName: 'User 1',
        content: 'Parent',
        createdAt: now,
        updatedAt: now,
      );

      final replyComment = Comment(
        id: 'c2',
        documentId: 'doc1',
        parentId: 'c1',
        authorId: 'u2',
        authorName: 'User 2',
        content: 'Reply',
        createdAt: now,
        updatedAt: now,
      );

      notifier.addComment(parentComment);
      notifier.addComment(replyComment);

      // Act
      notifier.removeComment('c1');

      // Assert
      expect(notifier.state.comments, isEmpty);
      expect(notifier.state.total, 0);
    });

    test('CommentListNotifier clearComments resets state', () {
      // Arrange
      final now = DateTime(2024);
      notifier.addComment(Comment(
        id: 'c1',
        documentId: 'doc1',
        authorId: 'u1',
        authorName: 'User 1',
        content: 'Test',
        createdAt: now,
        updatedAt: now,
      ));

      // Act
      notifier.clearComments();

      // Assert
      expect(notifier.state.comments, isEmpty);
      expect(notifier.state.total, 0);
      expect(notifier.state.documentId, 'doc1');
    });
  });

  group('CommentEditNotifier Tests', () {
    test('CommentEditNotifier initial state is correct', () {
      // Arrange & Act
      final notifier = CommentEditNotifier();

      // Assert
      expect(notifier.state.content, '');
      expect(notifier.state.isSubmitting, false);
      expect(notifier.state.error, isNull);
    });

    test('CommentEditNotifier setContent updates state', () {
      // Arrange
      final notifier = CommentEditNotifier();

      // Act
      notifier.setContent('New content');

      // Assert
      expect(notifier.state.content, 'New content');
      expect(notifier.state.error, isNull);
    });

    test('CommentEditNotifier setContent clears previous error', () {
      // Arrange
      final notifier = CommentEditNotifier();
      // First set an error state through the notifier's behavior
      const stateWithError = CommentEditState(
        content: 'Old',
        error: 'Previous error',
      );
      // Use copyWith to update state preserving error
      notifier.state = stateWithError;

      // Act - setContent should clear error by passing explicit null
      notifier.setContent('New content');

      // Assert - Error should be cleared
      expect(notifier.state.content, 'New content');
      expect(notifier.state.error, isNull);
    });

    test('CommentEditNotifier clear resets state', () {
      // Arrange
      final notifier = CommentEditNotifier();
      notifier.setContent('Some content');

      // Act
      notifier.clear();

      // Assert
      expect(notifier.state.content, '');
      expect(notifier.state.isSubmitting, false);
      expect(notifier.state.error, isNull);
    });
  });

  group('CommentListState Edge Cases', () {
    test('CommentListState with empty comments', () {
      // Arrange & Act
      const state = CommentListState(
        documentId: 'doc1',
      );

      // Assert
      expect(state.comments, isEmpty);
      expect(state.hasMore, false);
      expect(state.topLevelComments, isEmpty);
    });

    test('CommentListState with nested comments at multiple levels', () {
      // Arrange
      final now = DateTime(2024);
      final state = CommentListState(
        documentId: 'doc1',
        comments: [
          Comment(
            id: 'c1',
            documentId: 'doc1',
            authorId: 'u1',
            authorName: 'User 1',
            content: 'Level 1',
            createdAt: now,
            updatedAt: now,
          ),
          Comment(
            id: 'c2',
            documentId: 'doc1',
            parentId: 'c1',
            authorId: 'u2',
            authorName: 'User 2',
            content: 'Level 2',
            createdAt: now,
            updatedAt: now,
          ),
          Comment(
            id: 'c3',
            documentId: 'doc1',
            parentId: 'c2',
            authorId: 'u3',
            authorName: 'User 3',
            content: 'Level 3',
            createdAt: now,
            updatedAt: now,
          ),
        ],
      );

      // Assert
      expect(state.topLevelComments.length, 1);
      expect(state.getReplies('c1').length, 1);
      expect(state.getReplies('c2').length, 1);
    });

    test('CommentListState copyWith preserves all fields when null', () {
      // Arrange
      const original = CommentListState(
        documentId: 'doc1',
        total: 5,
        isLoading: true,
        error: 'Error',
        parentId: 'parent1',
      );

      // Act
      final modified = original.copyWith();

      // Assert
      expect(modified.documentId, 'doc1');
      expect(modified.comments, isEmpty);
      expect(modified.total, 5);
      expect(modified.isLoading, true);
      expect(modified.error, 'Error');
      expect(modified.parentId, 'parent1');
    });

    test('CommentListState with very large total', () {
      // Arrange & Act
      const state = CommentListState(
        documentId: 'doc1',
        total: 999999,
      );

      // Assert
      expect(state.total, 999999);
      expect(state.hasMore, true);
    });
  });

  group('CommentEditState Edge Cases', () {
    test('CommentEditState with special characters in content', () {
      // Arrange & Act
      const state = CommentEditState(
        content: r'Test with emoji 🎉 and symbols @#$%',
      );

      // Assert
      expect(state.isValid, true);
      expect(state.content, contains('emoji'));
    });

    test('CommentEditState with very long content', () {
      // Arrange & Act
      final longContent = 'A' * 5000;
      final state = CommentEditState(content: longContent);

      // Assert
      expect(state.isValid, true);
      expect(state.content.length, 5000);
    });

    test('CommentEditState copyWith with null values', () {
      // Arrange
      const original = CommentEditState(
        content: 'Test',
        isSubmitting: true,
        error: 'Error',
      );

      // Act
      final modified = original.copyWith();

      // Assert
      expect(modified.content, 'Test');
      expect(modified.isSubmitting, true);
      expect(modified.error, 'Error');
    });

    test('CommentEditState copyWith with selective updates', () {
      // Arrange
      const original = CommentEditState(
        content: 'Original',
      );

      // Act
      final modified = original.copyWith(
        isSubmitting: true,
      );

      // Assert
      expect(modified.content, 'Original'); // Preserved
      expect(modified.isSubmitting, true);
      expect(modified.error, isNull); // Preserved
    });
  });
}
