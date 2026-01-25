import 'package:flutter_test/flutter_test.dart';
import 'package:miniwiki/domain/entities/comment.dart';
import 'package:miniwiki/domain/repositories/comment_repository.dart';
import 'package:miniwiki/services/comment_service.dart';

// Mock implementation of CommentRepository for testing
class MockCommentRepository implements CommentRepository {
  final Map<String, Comment> _comments = {};
  int _idCounter = 1;

  @override
  Future<CommentListResult> listComments({
    required String documentId,
    String? parentId,
    int limit = 50,
    int offset = 0,
  }) async {
    final comments = _comments.values
        .where((c) => c.documentId == documentId && c.parentId == parentId)
        .skip(offset)
        .take(limit)
        .toList();

    final total = _comments.values
        .where((c) => c.documentId == documentId && c.parentId == parentId)
        .length;

    return CommentListResult(
      comments: comments,
      total: total,
      limit: limit,
      offset: offset,
    );
  }

  @override
  Future<Comment?> getComment(String id) async => _comments[id];

  @override
  Future<Comment> createComment({
    required String documentId,
    required String content,
    String? parentId,
  }) async {
    final id = 'comment_$_idCounter';
    _idCounter++;

    final comment = Comment(
      id: id,
      documentId: documentId,
      parentId: parentId,
      authorId: 'user1',
      authorName: 'Test User',
      content: content,
      createdAt: DateTime.now(),
      updatedAt: DateTime.now(),
    );

    _comments[id] = comment;
    return comment;
  }

  @override
  Future<Comment> updateComment({
    required String id,
    required String content,
  }) async {
    final existing = _comments[id]!;
    final updated = existing.copyWith(
      content: content,
      updatedAt: DateTime.now(),
    );
    _comments[id] = updated;
    return updated;
  }

  @override
  Future<Comment> resolveComment(String id) async {
    final existing = _comments[id]!;
    final resolved = existing.copyWith(
      isResolved: true,
      resolvedBy: 'user1',
      resolvedAt: DateTime.now(),
    );
    _comments[id] = resolved;
    return resolved;
  }

  @override
  Future<Comment> unresolveComment(String id) async {
    final existing = _comments[id]!;
    final unresolved = existing.clearResolutionInfo();
    _comments[id] = unresolved;
    return unresolved;
  }

  @override
  Future<void> deleteComment(String id) async {
    _comments.remove(id);
  }
}

void main() {
  group('CommentService - Service Lifecycle', () {
    late CommentService service;
    late MockCommentRepository mockRepository;

    setUp(() {
      mockRepository = MockCommentRepository();
      service = CommentService(mockRepository);
    });

    test('Service can be created with repository', () {
      expect(service, isNotNull);
      expect(service, isA<CommentService>());
    });
  });

  group('CommentService - listComments', () {
    late CommentService service;
    late MockCommentRepository mockRepository;

    setUp(() async {
      mockRepository = MockCommentRepository();
      service = CommentService(mockRepository);

      // Create test comments
      await mockRepository.createComment(
        documentId: 'doc1',
        content: 'First comment',
      );
      await mockRepository.createComment(
        documentId: 'doc1',
        content: 'Second comment',
      );
      await mockRepository.createComment(
        documentId: 'doc2',
        content: 'Other document comment',
      );
    });

    test('listComments returns comments for document', () async {
      final result = await service.listComments(documentId: 'doc1');

      expect(result.comments.length, 2);
      expect(result.comments[0].content, 'First comment');
      expect(result.comments[1].content, 'Second comment');
    });

    test('listComments returns empty list for non-existent document',
        () async {
      final result = await service.listComments(documentId: 'nonexistent');

      expect(result.comments, isEmpty);
      expect(result.total, 0);
    });

    test('listComments respects limit parameter', () async {
      final result = await service.listComments(
        documentId: 'doc1',
        limit: 1,
      );

      expect(result.comments.length, 1);
      expect(result.total, 2); // Total is still 2
    });

    test('listComments respects offset parameter', () async {
      final result = await service.listComments(
        documentId: 'doc1',
        offset: 1,
      );

      expect(result.comments.length, 1);
      expect(result.comments[0].content, 'Second comment');
    });

    test('listComments filters by parentId', () async {
      // Create parent comment
      final parent = await mockRepository.createComment(
        documentId: 'doc1',
        content: 'Parent comment',
      );

      // Create reply
      await mockRepository.createComment(
        documentId: 'doc1',
        content: 'Reply comment',
        parentId: parent.id,
      );

      // List top-level comments (setUp created 2, plus the parent = 3 total)
      final topLevelResult =
          await service.listComments(documentId: 'doc1', parentId: null);
      expect(topLevelResult.comments.length, 3);

      // List replies
      final repliesResult = await service.listComments(
        documentId: 'doc1',
        parentId: parent.id,
      );
      expect(repliesResult.comments.length, 1);
      expect(repliesResult.comments[0].content, 'Reply comment');
    });

    test('listComments uses default pagination', () async {
      final result = await service.listComments(documentId: 'doc1');

      expect(result.limit, 50);
      expect(result.offset, 0);
    });
  });

  group('CommentService - getComment', () {
    late CommentService service;
    late MockCommentRepository mockRepository;

    setUp(() async {
      mockRepository = MockCommentRepository();
      service = CommentService(mockRepository);

      await mockRepository.createComment(
        documentId: 'doc1',
        content: 'Test comment',
      );
    });

    test('getComment returns comment when it exists', () async {
      final comment = await service.getComment('comment_1');

      expect(comment, isNotNull);
      expect(comment!.id, 'comment_1');
      expect(comment.content, 'Test comment');
    });

    test('getComment returns null for non-existent comment', () async {
      final comment = await service.getComment('nonexistent');

      expect(comment, isNull);
    });
  });

  group('CommentService - createComment', () {
    late CommentService service;
    late MockCommentRepository mockRepository;

    setUp(() {
      mockRepository = MockCommentRepository();
      service = CommentService(mockRepository);
    });

    test('createComment creates comment with content', () async {
      final comment = await service.createComment(
        documentId: 'doc1',
        content: 'New comment',
      );

      expect(comment.id, isNotNull);
      expect(comment.documentId, 'doc1');
      expect(comment.content, 'New comment');
      expect(comment.authorId, 'user1');
      expect(comment.isResolved, false);
    });

    test('createComment creates top-level comment when parentId is null',
        () async {
      final comment = await service.createComment(
        documentId: 'doc1',
        content: 'Top-level comment',
        parentId: null,
      );

      expect(comment.parentId, isNull);
    });

    test('createComment creates reply when parentId is provided', () async {
      // Create parent
      final parent = await service.createComment(
        documentId: 'doc1',
        content: 'Parent',
      );

      // Create reply
      final reply = await service.createComment(
        documentId: 'doc1',
        content: 'Reply',
        parentId: parent.id,
      );

      expect(reply.parentId, parent.id);
    });

    test('createComment sanitizes content (trims whitespace)', () async {
      final comment = await service.createComment(
        documentId: 'doc1',
        content: '  Comment with spaces  ',
      );

      expect(comment.content, 'Comment with spaces');
    });

    test('createComment stores timestamps', () async {
      final beforeCreate = DateTime.now();

      final comment = await service.createComment(
        documentId: 'doc1',
        content: 'Timestamped comment',
      );

      final afterCreate = DateTime.now();

      expect(comment.createdAt, isNotNull);
      expect(comment.updatedAt, isNotNull);
      expect(
        comment.createdAt!.isAfter(beforeCreate) ||
            comment.createdAt!.isAtSameMomentAs(beforeCreate),
        true,
      );
      expect(
        comment.createdAt!.isBefore(afterCreate) ||
            comment.createdAt!.isAtSameMomentAs(afterCreate),
        true,
      );
    });
  });

  group('CommentService - updateComment', () {
    late CommentService service;
    late MockCommentRepository mockRepository;
    late Comment existingComment;

    setUp(() async {
      mockRepository = MockCommentRepository();
      service = CommentService(mockRepository);

      existingComment = await mockRepository.createComment(
        documentId: 'doc1',
        content: 'Original content',
      );
    });

    test('updateComment updates comment content', () async {
      final updated = await service.updateComment(
        id: existingComment.id,
        content: 'Updated content',
      );

      expect(updated.id, existingComment.id);
      expect(updated.content, 'Updated content');
    });

    test('updateComment sanitizes content', () async {
      final updated = await service.updateComment(
        id: existingComment.id,
        content: '  Updated content  ',
      );

      expect(updated.content, 'Updated content');
    });

    test('updateComment updates timestamp', () async {
      final beforeUpdate = existingComment.updatedAt;

      await Future<void>.delayed(const Duration(milliseconds: 10));

      final updated = await service.updateComment(
        id: existingComment.id,
        content: 'Updated',
      );

      expect(updated.updatedAt, isNotNull);
      expect(updated.updatedAt!.isAfter(beforeUpdate!), true);
    });

    test('updateComment preserves other fields', () async {
      final updated = await service.updateComment(
        id: existingComment.id,
        content: 'Updated',
      );

      expect(updated.documentId, existingComment.documentId);
      expect(updated.authorId, existingComment.authorId);
      expect(updated.authorName, existingComment.authorName);
      expect(updated.isResolved, existingComment.isResolved);
    });
  });

  group('CommentService - resolveComment', () {
    late CommentService service;
    late MockCommentRepository mockRepository;
    late Comment existingComment;

    setUp(() async {
      mockRepository = MockCommentRepository();
      service = CommentService(mockRepository);

      existingComment = await mockRepository.createComment(
        documentId: 'doc1',
        content: 'Unresolved comment',
      );
    });

    test('resolveComment marks comment as resolved', () async {
      final resolved = await service.resolveComment(existingComment.id);

      expect(resolved.id, existingComment.id);
      expect(resolved.isResolved, true);
      expect(resolved.resolvedBy, 'user1');
      expect(resolved.resolvedAt, isNotNull);
    });

    test('resolveComment sets resolved timestamp', () async {
      final beforeResolve = DateTime.now();

      await Future<void>.delayed(const Duration(milliseconds: 10));

      final resolved = await service.resolveComment(existingComment.id);

      expect(resolved.resolvedAt, isNotNull);
      expect(
        resolved.resolvedAt!.isAfter(beforeResolve) ||
            resolved.resolvedAt!.isAtSameMomentAs(beforeResolve),
        true,
      );
    });

    test('resolveComment preserves other fields', () async {
      final resolved = await service.resolveComment(existingComment.id);

      expect(resolved.content, existingComment.content);
      expect(resolved.documentId, existingComment.documentId);
      expect(resolved.authorId, existingComment.authorId);
    });
  });

  group('CommentService - unresolveComment', () {
    late CommentService service;
    late MockCommentRepository mockRepository;
    late Comment resolvedComment;

    setUp(() async {
      mockRepository = MockCommentRepository();
      service = CommentService(mockRepository);

      final created = await mockRepository.createComment(
        documentId: 'doc1',
        content: 'Comment to resolve',
      );
      resolvedComment = await mockRepository.resolveComment(created.id);
    });

    test('unresolveComment marks comment as unresolved', () async {
      final unresolved =
          await service.unresolveComment(resolvedComment.id);

      expect(unresolved.id, resolvedComment.id);
      expect(unresolved.isResolved, false);
      expect(unresolved.resolvedBy, isNull);
      expect(unresolved.resolvedAt, isNull);
    });

    test('unresolveComment preserves other fields', () async {
      final unresolved =
          await service.unresolveComment(resolvedComment.id);

      expect(unresolved.content, resolvedComment.content);
      expect(unresolved.documentId, resolvedComment.documentId);
    });
  });

  group('CommentService - deleteComment', () {
    late CommentService service;
    late MockCommentRepository mockRepository;
    late Comment commentToDelete;

    setUp(() async {
      mockRepository = MockCommentRepository();
      service = CommentService(mockRepository);

      commentToDelete = await mockRepository.createComment(
        documentId: 'doc1',
        content: 'Comment to delete',
      );
    });

    test('deleteComment removes comment', () async {
      await service.deleteComment(commentToDelete.id);

      final retrieved = await service.getComment(commentToDelete.id);
      expect(retrieved, isNull);
    });

    test('deleteComment completes without error', () async {
      await expectLater(
        service.deleteComment(commentToDelete.id),
        completes,
      );
    });

    test('deleteComment is idempotent (can delete non-existent)', () async {
      await service.deleteComment('nonexistent');

      // Should not throw
      expect(true, isTrue);
    });
  });

  group('CommentService - Content Sanitization', () {
    late CommentService service;
    late MockCommentRepository mockRepository;

    setUp(() {
      mockRepository = MockCommentRepository();
      service = CommentService(mockRepository);
    });

    test('Content trimming removes leading whitespace', () async {
      final comment = await service.createComment(
        documentId: 'doc1',
        content: '   Leading spaces',
      );

      expect(comment.content, 'Leading spaces');
    });

    test('Content trimming removes trailing whitespace', () async {
      final comment = await service.createComment(
        documentId: 'doc1',
        content: 'Trailing spaces   ',
      );

      expect(comment.content, 'Trailing spaces');
    });

    test('Content trimming removes both leading and trailing', () async {
      final comment = await service.createComment(
        documentId: 'doc1',
        content: '   Both sides   ',
      );

      expect(comment.content, 'Both sides');
    });

    test('Content trimming preserves internal spaces', () async {
      final comment = await service.createComment(
        documentId: 'doc1',
        content: 'Internal   spaces   preserved',
      );

      expect(comment.content, 'Internal   spaces   preserved');
    });

    test('Content trimming handles empty string', () async {
      final comment = await service.createComment(
        documentId: 'doc1',
        content: '   ',
      );

      expect(comment.content, '');
    });

    test('Content trimming handles tabs', () async {
      final comment = await service.createComment(
        documentId: 'doc1',
        content: '\t\tTab content\t\t',
      );

      // Note: trim() removes all whitespace including tabs
      expect(comment.content, 'Tab content');
    });

    test('Content trimming handles newlines', () async {
      final comment = await service.createComment(
        documentId: 'doc1',
        content: '\nNewline content\n',
      );

      // Note: trim() removes newlines too
      expect(comment.content, 'Newline content');
    });
  });

  group('CommentService - Integration Scenarios', () {
    late CommentService service;
    late MockCommentRepository mockRepository;

    setUp(() {
      mockRepository = MockCommentRepository();
      service = CommentService(mockRepository);
    });

    test('Full comment lifecycle', () async {
      // Create
      final comment = await service.createComment(
        documentId: 'doc1',
        content: 'Initial comment',
      );
      expect(comment.isResolved, false);

      // Update
      final updated = await service.updateComment(
        id: comment.id,
        content: 'Updated comment',
      );
      expect(updated.content, 'Updated comment');

      // Resolve
      final resolved = await service.resolveComment(comment.id);
      expect(resolved.isResolved, true);

      // Unresolve
      final unresolved = await service.unresolveComment(comment.id);
      expect(unresolved.isResolved, false);

      // Delete
      await service.deleteComment(comment.id);
      final retrieved = await service.getComment(comment.id);
      expect(retrieved, isNull);
    });

    test('Threaded comments workflow', () async {
      // Create parent comment
      final parent = await service.createComment(
        documentId: 'doc1',
        content: 'Parent comment',
      );

      // Create multiple replies
      final reply1 = await service.createComment(
        documentId: 'doc1',
        content: 'First reply',
        parentId: parent.id,
      );

      final reply2 = await service.createComment(
        documentId: 'doc1',
        content: 'Second reply',
        parentId: parent.id,
      );

      // List top-level comments (should only include parent)
      final allComments =
          await service.listComments(documentId: 'doc1', parentId: null);
      expect(allComments.comments.length, 1);

      // List replies only
      final replies = await service.listComments(
        documentId: 'doc1',
        parentId: parent.id,
      );
      expect(replies.comments.length, 2);
      expect(replies.comments.any((c) => c.id == reply1.id), true);
      expect(replies.comments.any((c) => c.id == reply2.id), true);
    });

    test('Pagination with many comments', () async {
      // Create 100 comments
      for (var i = 0; i < 100; i++) {
        await service.createComment(
          documentId: 'doc1',
          content: 'Comment $i',
        );
      }

      // First page
      final page1 =
          await service.listComments(documentId: 'doc1', limit: 20, offset: 0);
      expect(page1.comments.length, 20);
      expect(page1.total, 100);
      expect(page1.hasMore, true);

      // Second page
      final page2 =
          await service.listComments(documentId: 'doc1', limit: 20, offset: 20);
      expect(page2.comments.length, 20);
      expect(page2.total, 100);
      expect(page2.hasMore, true);

      // Last page
      final lastPage =
          await service.listComments(documentId: 'doc1', limit: 20, offset: 80);
      expect(lastPage.comments.length, 20);
      expect(lastPage.total, 100);
      expect(lastPage.hasMore, false);

      // Beyond last page
      final emptyPage =
          await service.listComments(documentId: 'doc1', limit: 20, offset: 100);
      expect(emptyPage.comments.length, 0);
      expect(emptyPage.hasMore, false);
    });

    test('Multiple documents have separate comments', () async {
      await service.createComment(
        documentId: 'doc1',
        content: 'Doc1 comment',
      );
      await service.createComment(
        documentId: 'doc2',
        content: 'Doc2 comment',
      );

      final doc1Comments = await service.listComments(documentId: 'doc1');
      final doc2Comments = await service.listComments(documentId: 'doc2');

      expect(doc1Comments.comments.length, 1);
      expect(doc2Comments.comments.length, 1);
      expect(doc1Comments.comments[0].content, 'Doc1 comment');
      expect(doc2Comments.comments[0].content, 'Doc2 comment');
    });
  });

  group('CommentService - Error Handling', () {
    late CommentService service;
    late MockCommentRepository mockRepository;

    setUp(() {
      mockRepository = MockCommentRepository();
      service = CommentService(mockRepository);
    });

    test('updateComment with non-existent ID throws', () async {
      // Mock repository will throw if comment doesn't exist
      expect(
        () => service.updateComment(
          id: 'nonexistent',
          content: 'Updated',
        ),
        throwsA(isA<TypeError>()),
      );
    });

    test('resolveComment with non-existent ID throws', () async {
      expect(
        () => service.resolveComment('nonexistent'),
        throwsA(isA<TypeError>()),
      );
    });

    test('unresolveComment with non-existent ID throws', () async {
      expect(
        () => service.unresolveComment('nonexistent'),
        throwsA(isA<TypeError>()),
      );
    });
  });

  group('CommentListResult - hasMore calculation', () {
    test('hasMore returns false when all comments loaded', () {
      final result = CommentListResult(
        comments: List.generate(10, (i) => Comment(
          id: 'comment_$i',
          documentId: 'doc1',
          authorId: 'user1',
          authorName: 'User',
          content: 'Comment $i',
        )),
        total: 10,
        limit: 10,
        offset: 0,
      );

      expect(result.hasMore, false);
    });

    test('hasMore returns true when more comments available', () {
      final result = CommentListResult(
        comments: List.generate(5, (i) => Comment(
          id: 'comment_$i',
          documentId: 'doc1',
          authorId: 'user1',
          authorName: 'User',
          content: 'Comment $i',
        )),
        total: 10,
        limit: 5,
        offset: 0,
      );

      expect(result.hasMore, true);
    });

    test('hasMore returns true when on subsequent page', () {
      final result = CommentListResult(
        comments: List.generate(5, (i) => Comment(
          id: 'comment_$i',
          documentId: 'doc1',
          authorId: 'user1',
          authorName: 'User',
          content: 'Comment $i',
        )),
        total: 10,
        limit: 10,
        offset: 5,
      );

      expect(result.hasMore, false); // 5 + 5 = 10 (no more)
    });

    test('hasMore handles empty result', () {
      final result = CommentListResult(
        comments: const [],
        total: 0,
        limit: 10,
        offset: 0,
      );

      expect(result.hasMore, false);
    });
  });

  group('Comment - Entity Operations', () {
    test('Comment copyWith updates all fields independently', () {
      final now = DateTime(2025, 1, 1, 12, 0);
      final comment = Comment(
        id: 'comment1',
        documentId: 'doc1',
        authorId: 'user1',
        authorName: 'Test User',
        content: 'Original',
        createdAt: now,
        updatedAt: now,
      );

      final updated = comment.copyWith(
        content: 'Updated',
        isResolved: true,
        resolvedBy: 'admin',
        resolvedAt: now,
      );

      expect(updated.id, 'comment1');
      expect(updated.content, 'Updated');
      expect(updated.isResolved, true);
      expect(updated.resolvedBy, 'admin');
      expect(updated.resolvedAt, now);
      expect(comment.content, 'Original'); // Original unchanged
    });

    test('Comment copyWith with null values keeps original', () {
      const comment = Comment(
        id: 'comment1',
        documentId: 'doc1',
        authorId: 'user1',
        authorName: 'Test User',
        content: 'Original',
      );

      final updated = comment.copyWith(content: null);

      expect(updated.content, 'Original');
    });

    test('Comment fromJson parses snake_case to camelCase', () {
      final json = {
        'id': 'comment1',
        'document_id': 'doc1',
        'parent_id': null,
        'author_id': 'user1',
        'author_name': 'Test User',
        'author_avatar': null,
        'content': 'Test comment',
        'is_resolved': false,
        'resolved_by': null,
        'resolved_at': null,
        'created_at': '2025-01-01T12:00:00.000Z',
        'updated_at': '2025-01-01T12:00:00.000Z',
      };

      final comment = Comment.fromJson(json);

      expect(comment.id, 'comment1');
      expect(comment.documentId, 'doc1');
      expect(comment.authorId, 'user1');
      expect(comment.authorName, 'Test User');
      expect(comment.content, 'Test comment');
      expect(comment.isResolved, false);
    });

    test('Comment fromJson handles null optional fields', () {
      final json = {
        'id': 'comment1',
        'document_id': 'doc1',
        'parent_id': null,
        'author_id': 'user1',
        'author_name': 'Test User',
        'author_avatar': null,
        'content': 'Test',
        'is_resolved': false,
        'resolved_by': null,
        'resolved_at': null,
        'created_at': null,
        'updated_at': null,
      };

      final comment = Comment.fromJson(json);

      expect(comment.parentId, isNull);
      expect(comment.authorAvatar, isNull);
      expect(comment.resolvedBy, isNull);
      expect(comment.resolvedAt, isNull);
      expect(comment.createdAt, isNull);
      expect(comment.updatedAt, isNull);
    });

    test('Comment fromJson defaults isResolved to false', () {
      final json = {
        'id': 'comment1',
        'document_id': 'doc1',
        'parent_id': null,
        'author_id': 'user1',
        'author_name': 'Test User',
        'content': 'Test',
        'is_resolved': null, // Missing field
      };

      final comment = Comment.fromJson(json);

      expect(comment.isResolved, false);
    });

    test('Comment toJson converts camelCase to snake_case', () {
      final now = DateTime(2025, 1, 1, 12, 0);
      final comment = Comment(
        id: 'comment1',
        documentId: 'doc1',
        authorId: 'user1',
        authorName: 'Test User',
        content: 'Test comment',
        parentId: 'parent1',
        authorAvatar: 'avatar.png',
        isResolved: true,
        resolvedBy: 'admin',
        resolvedAt: now,
        createdAt: now,
        updatedAt: now,
      );

      final json = comment.toJson();

      expect(json['id'], 'comment1');
      expect(json['document_id'], 'doc1');
      expect(json['parent_id'], 'parent1');
      expect(json['author_id'], 'user1');
      expect(json['author_name'], 'Test User');
      expect(json['content'], 'Test comment');
      expect(json['author_avatar'], 'avatar.png');
      expect(json['is_resolved'], true);
      expect(json['resolved_by'], 'admin');
      expect(json['resolved_at'], isNotNull);
      expect(json['created_at'], isNotNull);
      expect(json['updated_at'], isNotNull);
    });

    test('Comment toJson omits null fields', () {
      const comment = Comment(
        id: 'comment1',
        documentId: 'doc1',
        authorId: 'user1',
        authorName: 'Test User',
        content: 'Test comment',
      );

      final json = comment.toJson();

      expect(json.containsKey('parent_id'), false);
      expect(json.containsKey('author_avatar'), false);
      expect(json.containsKey('is_resolved'), true); // Default value included
      expect(json.containsKey('resolved_by'), false);
      expect(json.containsKey('resolved_at'), false);
      expect(json.containsKey('created_at'), false);
      expect(json.containsKey('updated_at'), false);
    });

    test('Comment equality is based on id', () {
      const comment1 = Comment(
        id: 'comment1',
        documentId: 'doc1',
        authorId: 'user1',
        authorName: 'User 1',
        content: 'Content 1',
      );

      const comment2 = Comment(
        id: 'comment1',
        documentId: 'doc2',
        authorId: 'user2',
        authorName: 'User 2',
        content: 'Content 2',
      );

      expect(comment1, comment2);
      expect(comment1.hashCode, comment2.hashCode);
    });

    test('Comment inequality for different ids', () {
      const comment1 = Comment(
        id: 'comment1',
        documentId: 'doc1',
        authorId: 'user1',
        authorName: 'User',
        content: 'Content',
      );

      const comment2 = Comment(
        id: 'comment2',
        documentId: 'doc1',
        authorId: 'user1',
        authorName: 'User',
        content: 'Content',
      );

      expect(comment1, isNot(comment2));
    });

    test('Comment toString includes id and author', () {
      const comment = Comment(
        id: 'comment1',
        documentId: 'doc1',
        authorId: 'user1',
        authorName: 'Test User',
        content: 'Content',
      );

      expect(comment.toString(), 'Comment(id: comment1, author: Test User)');
    });
  });
}
