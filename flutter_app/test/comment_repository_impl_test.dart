import 'package:dio/dio.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:miniwiki/core/network/api_client.dart';
import 'package:miniwiki/data/repositories/comment_repository_impl.dart';
import 'package:mocktail/mocktail.dart';

class MockApiClient extends Mock implements ApiClient {}

void main() {
  group('CommentRepositoryImpl', () {
    late CommentRepositoryImpl repository;
    late MockApiClient mockApiClient;

    setUp(() {
      mockApiClient = MockApiClient();
      repository = CommentRepositoryImpl(mockApiClient);
    });

    group('listComments', () {
      test('should return comments list successfully', () async {
        // Arrange
        final response = Response<dynamic>(
          requestOptions: RequestOptions(path: '/documents/doc1/comments'),
          data: {
            'data': {
              'comments': [
                {
                  'id': 'comment1',
                  'document_id': 'doc1',
                  'content': 'Test comment',
                  'author_id': 'user1',
                  'author_name': 'Test User',
                  'created_at': '2025-01-01T00:00:00Z',
                  'updated_at': '2025-01-01T00:00:00Z',
                }
              ],
              'total': 1
            }
          },
          statusCode: 200,
        );
        when(() => mockApiClient.get(
          '/documents/doc1/comments',
          queryParams: any(named: 'queryParams'),
        )).thenAnswer((_) async => response);

        // Act
        final result = await repository.listComments(documentId: 'doc1');

        // Assert
        expect(result.comments.length, 1);
        expect(result.total, 1);
        expect(result.comments.first.content, 'Test comment');
      });

      test('should handle empty comments list', () async {
        // Arrange
        final response = Response<dynamic>(
          requestOptions: RequestOptions(path: '/documents/doc1/comments'),
          data: {
            'data': {'comments': <dynamic>[], 'total': 0}
          },
          statusCode: 200,
        );
        when(() => mockApiClient.get(
          '/documents/doc1/comments',
          queryParams: any(named: 'queryParams'),
        )).thenAnswer((_) async => response);

        // Act
        final result = await repository.listComments(documentId: 'doc1');

        // Assert
        expect(result.comments.isEmpty, true);
        expect(result.total, 0);
      });

      test('should include parent_id in query when provided', () async {
        // Arrange
        final response = Response<dynamic>(
          requestOptions: RequestOptions(path: '/documents/doc1/comments'),
          data: {
            'data': {'comments': <dynamic>[], 'total': 0}
          },
          statusCode: 200,
        );
        when(() => mockApiClient.get(
          '/documents/doc1/comments',
          queryParams: any(named: 'queryParams'),
        )).thenAnswer((_) async => response);

        // Act
        await repository.listComments(documentId: 'doc1', parentId: 'parent1');

        // Assert
        final captured = verify(() => mockApiClient.get(
          '/documents/doc1/comments',
          queryParams: captureAny(named: 'queryParams'),
        )).captured.single as Map<String, dynamic>;
        expect(captured['parent_id'], 'parent1');
      });

      test('should apply limit and offset correctly', () async {
        // Arrange
        final response = Response<dynamic>(
          requestOptions: RequestOptions(path: '/documents/doc1/comments'),
          data: {
            'data': {'comments': <dynamic>[], 'total': 0}
          },
          statusCode: 200,
        );
        when(() => mockApiClient.get(
          '/documents/doc1/comments',
          queryParams: any(named: 'queryParams'),
        )).thenAnswer((_) async => response);

        // Act
        await repository.listComments(
          documentId: 'doc1',
          limit: 100,
          offset: 50,
        );

        // Assert
        final captured = verify(() => mockApiClient.get(
          '/documents/doc1/comments',
          queryParams: captureAny(named: 'queryParams'),
        )).captured.single as Map<String, dynamic>;
        expect(captured['limit'], 100);
        expect(captured['offset'], 50);
      });
    });

    group('getComment', () {
      test('should return comment when found', () async {
        // Arrange
        final response = Response<dynamic>(
          requestOptions: RequestOptions(path: '/comments/comment1'),
          data: {
            'data': {
              'id': 'comment1',
              'document_id': 'doc1',
              'content': 'Test comment',
              'author_id': 'user1',
              'author_name': 'Test User',
              'created_at': '2025-01-01T00:00:00Z',
              'updated_at': '2025-01-01T00:00:00Z',
            }
          },
          statusCode: 200,
        );
        when(() => mockApiClient.get('/comments/comment1'))
            .thenAnswer((_) async => response);

        // Act
        final result = await repository.getComment('comment1');

        // Assert
        expect(result, isNotNull);
        expect(result?.id, 'comment1');
        expect(result?.content, 'Test comment');
      });

      test('should return null when comment not found', () async {
        // Arrange
        when(() => mockApiClient.get('/comments/nonexistent'))
            .thenThrow(NotFoundError('Comment not found'));

        // Act
        final result = await repository.getComment('nonexistent');

        // Assert
        expect(result, isNull);
      });

      test('should return null on generic error', () async {
        // Arrange
        when(() => mockApiClient.get('/comments/comment1'))
            .thenThrow(Exception('Generic error'));

        // Act
        final result = await repository.getComment('comment1');

        // Assert
        expect(result, isNull);
      });
    });

    group('createComment', () {
      test('should create comment successfully', () async {
        // Arrange
        final response = Response<dynamic>(
          requestOptions: RequestOptions(path: '/documents/doc1/comments'),
          data: {
            'data': {
              'comment': {
                'id': 'comment1',
                'document_id': 'doc1',
                'content': 'New comment',
                'author_id': 'user1',
                'author_name': 'Test User',
                'created_at': '2025-01-01T00:00:00Z',
                'updated_at': '2025-01-01T00:00:00Z',
              }
            }
          },
          statusCode: 201,
        );
        when(() => mockApiClient.post(
          '/documents/doc1/comments',
          data: any(named: 'data'),
        )).thenAnswer((_) async => response);

        // Act
        final result = await repository.createComment(
          documentId: 'doc1',
          content: 'New comment',
        );

        // Assert
        expect(result.id, 'comment1');
        expect(result.content, 'New comment');
        expect(result.documentId, 'doc1');
      });

      test('should create reply comment when parent_id provided', () async {
        // Arrange
        final response = Response<dynamic>(
          requestOptions: RequestOptions(path: '/documents/doc1/comments'),
          data: {
            'data': {
              'comment': {
                'id': 'reply1',
                'document_id': 'doc1',
                'parent_id': 'comment1',
                'content': 'Reply comment',
                'author_id': 'user1',
                'author_name': 'Test User',
                'created_at': '2025-01-01T00:00:00Z',
                'updated_at': '2025-01-01T00:00:00Z',
              }
            }
          },
          statusCode: 201,
        );
        when(() => mockApiClient.post(
          '/documents/doc1/comments',
          data: any(named: 'data'),
        )).thenAnswer((_) async => response);

        // Act
        final result = await repository.createComment(
          documentId: 'doc1',
          content: 'Reply comment',
          parentId: 'comment1',
        );

        // Assert
        expect(result.id, 'reply1');
        expect(result.parentId, 'comment1');
      });
    });

    group('updateComment', () {
      test('should update comment successfully', () async {
        // Arrange
        final response = Response<dynamic>(
          requestOptions: RequestOptions(path: '/comments/comment1'),
          data: {
            'data': {
              'id': 'comment1',
              'document_id': 'doc1',
              'content': 'Updated comment',
              'author_id': 'user1',
              'author_name': 'Test User',
              'created_at': '2025-01-01T00:00:00Z',
              'updated_at': '2025-01-01T01:00:00Z',
            }
          },
          statusCode: 200,
        );
        when(() => mockApiClient.patch(
          '/comments/comment1',
          data: any(named: 'data'),
        )).thenAnswer((_) async => response);

        // Act
        final result = await repository.updateComment(
          id: 'comment1',
          content: 'Updated comment',
        );

        // Assert
        expect(result.id, 'comment1');
        expect(result.content, 'Updated comment');
      });
    });

    group('resolveComment', () {
      test('should resolve comment successfully', () async {
        // Arrange
        final response = Response<dynamic>(
          requestOptions: RequestOptions(path: '/comments/comment1/resolve'),
          data: {
            'data': {
              'id': 'comment1',
              'document_id': 'doc1',
              'content': 'Test comment',
              'is_resolved': true,
              'resolved_by': 'user1',
              'resolved_at': '2025-01-01T01:00:00Z',
              'author_id': 'user1',
              'author_name': 'Test User',
              'created_at': '2025-01-01T00:00:00Z',
              'updated_at': '2025-01-01T01:00:00Z',
            }
          },
          statusCode: 200,
        );
        when(() => mockApiClient.post('/comments/comment1/resolve'))
            .thenAnswer((_) async => response);

        // Act
        final result = await repository.resolveComment('comment1');

        // Assert
        expect(result.isResolved, true);
        expect(result.resolvedBy, 'user1');
      });
    });

    group('unresolveComment', () {
      test('should unresolve comment successfully', () async {
        // Arrange
        final response = Response<dynamic>(
          requestOptions: RequestOptions(path: '/comments/comment1/unresolve'),
          data: {
            'data': {
              'id': 'comment1',
              'document_id': 'doc1',
              'content': 'Test comment',
              'is_resolved': false,
              'author_id': 'user1',
              'author_name': 'Test User',
              'created_at': '2025-01-01T00:00:00Z',
              'updated_at': '2025-01-01T01:00:00Z',
            }
          },
          statusCode: 200,
        );
        when(() => mockApiClient.post('/comments/comment1/unresolve'))
            .thenAnswer((_) async => response);

        // Act
        final result = await repository.unresolveComment('comment1');

        // Assert
        expect(result.isResolved, false);
      });
    });

    group('deleteComment', () {
      test('should delete comment successfully', () async {
        // Arrange
        final response = Response<dynamic>(
          requestOptions: RequestOptions(path: '/comments/comment1'),
          statusCode: 204,
        );
        when(() => mockApiClient.delete('/comments/comment1'))
            .thenAnswer((_) async => response);

        // Act & Assert
        await repository.deleteComment('comment1');
        verify(() => mockApiClient.delete('/comments/comment1')).called(1);
      });
    });
  });
}
