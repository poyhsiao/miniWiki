import 'package:dio/dio.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:miniwiki/core/network/api_client.dart';
import 'package:miniwiki/data/repositories/search_repository_impl.dart';
import 'package:miniwiki/domain/entities/search_result.dart';
import 'package:mocktail/mocktail.dart';

class MockApiClient extends Mock implements ApiClient {}

class MockResponse extends Mock implements Response<dynamic> {}

void main() {
  group('SearchRepositoryImpl', () {
    late SearchRepositoryImpl repository;
    late MockApiClient mockApiClient;

    setUp(() {
      mockApiClient = MockApiClient();
      repository = SearchRepositoryImpl(apiClient: mockApiClient);
    });

    group('search', () {
      test('should return search results successfully', () async {
        // Arrange
        final response = MockResponse();
        when(() => response.statusCode).thenReturn(200);
        when(() => response.data).thenReturn({
          'results': [
            {
              'documentId': 'doc1',
              'spaceId': 'space1',
              'spaceName': 'My Space',
              'title': 'Test Document',
              'snippet': 'This is a test document...',
              'score': 0.95,
            },
            {
              'documentId': 'doc2',
              'spaceId': 'space1',
              'spaceName': 'My Space',
              'title': 'Another Document',
              'snippet': 'Another test document...',
              'score': 0.85,
            },
          ],
          'total': 2,
          'took': 15,
        });

        when(() => mockApiClient.get(
          '/search',
          queryParams: any(named: 'queryParams'),
        )).thenAnswer((_) async => response);

        // Act
        final result = await repository.search(query: 'test');

        // Assert
        expect(result.$1.length, 2);
        expect(result.$2, 2);
        expect(result.$1[0].title, 'Test Document');
        expect(result.$1[0].score, 0.95);
        expect(result.$1[1].documentId, 'doc2');
        verify(() => mockApiClient.get(
          '/search',
          queryParams: {
            'q': 'test',
            'limit': 20,
            'offset': 0,
          },
        )).called(1);
      });

      test('should include spaceId in query params when provided', () async {
        // Arrange
        final response = MockResponse();
        when(() => response.statusCode).thenReturn(200);
        when(() => response.data).thenReturn({
          'results': [
            {
              'documentId': 'doc1',
              'spaceId': 'space1',
              'spaceName': 'My Space',
              'title': 'Test Document',
              'snippet': 'Snippet...',
              'score': 0.9,
            },
          ],
          'total': 1,
          'took': 10,
        });

        when(() => mockApiClient.get(
          '/search',
          queryParams: any(named: 'queryParams'),
        )).thenAnswer((_) async => response);

        // Act
        await repository.search(query: 'test', spaceId: 'space1');

        // Assert
        final captured = verify(() => mockApiClient.get(
          '/search',
          queryParams: captureAny(named: 'queryParams'),
        )).captured.single as Map<String, dynamic>;

        expect(captured['q'], 'test');
        expect(captured['spaceId'], 'space1');
        expect(captured['limit'], 20);
        expect(captured['offset'], 0);
      });

      test('should apply custom limit and offset', () async {
        // Arrange
        final response = MockResponse();
        when(() => response.statusCode).thenReturn(200);
        when(() => response.data).thenReturn({
          'results': <dynamic>[],
          'total': 0,
          'took': 5,
        });

        when(() => mockApiClient.get(
          '/search',
          queryParams: any(named: 'queryParams'),
        )).thenAnswer((_) async => response);

        // Act
        await repository.search(
          query: 'test',
          limit: 50,
          offset: 100,
        );

        // Assert
        final captured = verify(() => mockApiClient.get(
          '/search',
          queryParams: captureAny(named: 'queryParams'),
        )).captured.single as Map<String, dynamic>;

        expect(captured['limit'], 50);
        expect(captured['offset'], 100);
      });

      test('should return empty results when no matches found', () async {
        // Arrange
        final response = MockResponse();
        when(() => response.statusCode).thenReturn(200);
        when(() => response.data).thenReturn({
          'results': <dynamic>[],
          'total': 0,
          'took': 5,
        });

        when(() => mockApiClient.get(
          '/search',
          queryParams: any(named: 'queryParams'),
        )).thenAnswer((_) async => response);

        // Act
        final result = await repository.search(query: 'nonexistent');

        // Assert
        expect(result.$1.isEmpty, true);
        expect(result.$2, 0);
      });

      test('should handle all search result fields correctly', () async {
        // Arrange
        final response = MockResponse();
        when(() => response.statusCode).thenReturn(200);
        when(() => response.data).thenReturn({
          'results': [
            {
              'documentId': 'doc123',
              'spaceId': 'space456',
              'spaceName': 'Engineering Team',
              'title': 'API Documentation',
              'snippet': 'This document contains API endpoints...',
              'score': 0.98,
            },
          ],
          'total': 1,
          'took': 8,
        });

        when(() => mockApiClient.get(
          '/search',
          queryParams: any(named: 'queryParams'),
        )).thenAnswer((_) async => response);

        // Act
        final result = await repository.search(query: 'API');

        // Assert
        expect(result.$1.length, 1);
        final searchResult = result.$1[0];
        expect(searchResult.documentId, 'doc123');
        expect(searchResult.spaceId, 'space456');
        expect(searchResult.spaceName, 'Engineering Team');
        expect(searchResult.title, 'API Documentation');
        expect(searchResult.snippet, contains('API endpoints'));
        expect(searchResult.score, 0.98);
      });

      test('should throw ApiError on DioException with null response',
          () async {
        // Arrange
        final dioException = DioException(
          requestOptions: RequestOptions(path: '/search'),
          message: 'Network error',
        );

        when(() => mockApiClient.get(
          '/search',
          queryParams: any(named: 'queryParams'),
        )).thenThrow(dioException);

        // Act & Assert
        await expectLater(
          repository.search(query: 'test'),
          throwsA(isA<ApiError>()),
        );
      });

      test('should throw ApiError from DioException response', () async {
        // Arrange
        final errorResponse = Response<dynamic>(
          data: {
            'error': 'VALIDATION_ERROR',
            'message': 'Invalid query parameter',
          },
          statusCode: 400,
          requestOptions: RequestOptions(path: '/search'),
        );

        final dioException = DioException(
          requestOptions: RequestOptions(path: '/search'),
          response: errorResponse,
        );

        when(() => mockApiClient.get(
          '/search',
          queryParams: any(named: 'queryParams'),
        )).thenThrow(dioException);

        // Act & Assert
        await expectLater(
          repository.search(query: 'test'),
          throwsA(isA<ApiError>()),
        );
      });

      test('should throw ApiError on generic exception', () async {
        // Arrange
        when(() => mockApiClient.get(
          '/search',
          queryParams: any(named: 'queryParams'),
        )).thenThrow(Exception('Unexpected error'));

        // Act & Assert
        await expectLater(
          repository.search(query: 'test'),
          throwsA(isA<ApiError>()),
        );
      });

      test('should handle special characters in query', () async {
        // Arrange
        final response = MockResponse();
        when(() => response.statusCode).thenReturn(200);
        when(() => response.data).thenReturn({
          'results': <dynamic>[],
          'total': 0,
          'took': 3,
        });

        when(() => mockApiClient.get(
          '/search',
          queryParams: any(named: 'queryParams'),
        )).thenAnswer((_) async => response);

        // Act
        await repository.search(query: 'C++ programming');

        // Assert
        final captured = verify(() => mockApiClient.get(
          '/search',
          queryParams: captureAny(named: 'queryParams'),
        )).captured.single as Map<String, dynamic>;

        expect(captured['q'], 'C++ programming');
      });
    });

    group('getSuggestions', () {
      test('should return suggestions successfully', () async {
        // Arrange
        final response = MockResponse();
        when(() => response.statusCode).thenReturn(200);
        when(() => response.data).thenReturn({
          'suggestions': ['test document', 'test case', 'testing guide'],
        });

        when(() => mockApiClient.get(
          '/search/suggestions',
          queryParams: any(named: 'queryParams'),
        )).thenAnswer((_) async => response);

        // Act
        final result = await repository.getSuggestions('test');

        // Assert
        expect(result.length, 3);
        expect(result, contains('test document'));
        expect(result, contains('test case'));
        expect(result, contains('testing guide'));
        verify(() => mockApiClient.get(
          '/search/suggestions',
          queryParams: {'q': 'test'},
        )).called(1);
      });

      test('should return empty list when no suggestions', () async {
        // Arrange
        final response = MockResponse();
        when(() => response.statusCode).thenReturn(200);
        when(() => response.data).thenReturn({
          'suggestions': <dynamic>[],
        });

        when(() => mockApiClient.get(
          '/search/suggestions',
          queryParams: any(named: 'queryParams'),
        )).thenAnswer((_) async => response);

        // Act
        final result = await repository.getSuggestions('query');

        // Assert
        expect(result.isEmpty, true);
      });

      test('should throw ApiError on DioException with null response',
          () async {
        // Arrange
        final dioException = DioException(
          requestOptions: RequestOptions(path: '/search/suggestions'),
          message: 'Network error',
        );

        when(() => mockApiClient.get(
          '/search/suggestions',
          queryParams: any(named: 'queryParams'),
        )).thenThrow(dioException);

        // Act & Assert
        await expectLater(
          repository.getSuggestions('test'),
          throwsA(isA<ApiError>()),
        );
      });

      test('should throw ApiError from DioException response', () async {
        // Arrange
        final errorResponse = Response<dynamic>(
          data: {
            'error': 'RATE_LIMIT_EXCEEDED',
            'message': 'Too many requests',
          },
          statusCode: 429,
          requestOptions: RequestOptions(path: '/search/suggestions'),
        );

        final dioException = DioException(
          requestOptions: RequestOptions(path: '/search/suggestions'),
          response: errorResponse,
        );

        when(() => mockApiClient.get(
          '/search/suggestions',
          queryParams: any(named: 'queryParams'),
        )).thenThrow(dioException);

        // Act & Assert
        await expectLater(
          repository.getSuggestions('test'),
          throwsA(isA<ApiError>()),
        );
      });

      test('should throw ApiError on generic exception', () async {
        // Arrange
        when(() => mockApiClient.get(
          '/search/suggestions',
          queryParams: any(named: 'queryParams'),
        )).thenThrow(Exception('Server error'));

        // Act & Assert
        await expectLater(
          repository.getSuggestions('test'),
          throwsA(isA<ApiError>()),
        );
      });

      test('should handle partial suggestions', () async {
        // Arrange
        final response = MockResponse();
        when(() => response.statusCode).thenReturn(200);
        when(() => response.data).thenReturn({
          'suggestions': ['flutter widgets', 'flutter state management'],
        });

        when(() => mockApiClient.get(
          '/search/suggestions',
          queryParams: any(named: 'queryParams'),
        )).thenAnswer((_) async => response);

        // Act
        final result = await repository.getSuggestions('flutt');

        // Assert
        expect(result.length, 2);
        expect(result.every((s) => s.contains('flutter')), true);
      });

      test('should pass query parameter correctly', () async {
        // Arrange
        final response = MockResponse();
        when(() => response.statusCode).thenReturn(200);
        when(() => response.data).thenReturn({
          'suggestions': <dynamic>[],
        });

        when(() => mockApiClient.get(
          '/search/suggestions',
          queryParams: any(named: 'queryParams'),
        )).thenAnswer((_) async => response);

        // Act
        await repository.getSuggestions('search query');

        // Assert
        final captured = verify(() => mockApiClient.get(
          '/search/suggestions',
          queryParams: captureAny(named: 'queryParams'),
        )).captured.single as Map<String, dynamic>;

        expect(captured['q'], 'search query');
      });
    });

    group('SearchResult entity', () {
      test('should create SearchResult from JSON correctly', () {
        // Arrange
        final json = {
          'documentId': 'doc1',
          'spaceId': 'space1',
          'spaceName': 'Test Space',
          'title': 'Test Document',
          'snippet': 'Test snippet...',
          'score': 0.95,
        };

        // Act
        final result = SearchResult.fromJson(json);

        // Assert
        expect(result.documentId, 'doc1');
        expect(result.spaceId, 'space1');
        expect(result.spaceName, 'Test Space');
        expect(result.title, 'Test Document');
        expect(result.snippet, 'Test snippet...');
        expect(result.score, 0.95);
      });

      test('should handle null values in JSON', () {
        // Arrange
        final json = {
          'documentId': null,
          'spaceId': null,
          'spaceName': null,
          'title': null,
          'snippet': null,
          'score': null,
        };

        // Act
        final result = SearchResult.fromJson(json);

        // Assert
        expect(result.documentId, '');
        expect(result.spaceId, '');
        expect(result.spaceName, '');
        expect(result.title, '');
        expect(result.snippet, '');
        expect(result.score, 0.0);
      });

      test('should convert SearchResult to JSON correctly', () {
        // Arrange
        final searchResult = SearchResult(
          documentId: 'doc1',
          spaceId: 'space1',
          spaceName: 'Test Space',
          title: 'Test Document',
          snippet: 'Test snippet...',
          score: 0.95,
        );

        // Act
        final json = searchResult.toJson();

        // Assert
        expect(json['documentId'], 'doc1');
        expect(json['spaceId'], 'space1');
        expect(json['spaceName'], 'Test Space');
        expect(json['title'], 'Test Document');
        expect(json['snippet'], 'Test snippet...');
        expect(json['score'], 0.95);
      });

      test('should implement equality correctly', () {
        // Arrange
        final result1 = SearchResult(
          documentId: 'doc1',
          spaceId: 'space1',
          spaceName: 'Space',
          title: 'Doc',
          snippet: 'Snippet',
          score: 0.9,
        );

        final result2 = SearchResult(
          documentId: 'doc1',
          spaceId: 'space1',
          spaceName: 'Space',
          title: 'Doc',
          snippet: 'Snippet',
          score: 0.9,
        );

        final result3 = SearchResult(
          documentId: 'doc2',
          spaceId: 'space1',
          spaceName: 'Space',
          title: 'Doc',
          snippet: 'Snippet',
          score: 0.9,
        );

        // Assert
        // SearchResult equality checks ALL fields, not just documentId
        expect(result1 == result2, true); // All fields identical
        expect(result1 == result3, false); // Different documentId
      });
    });

    group('SearchResponse entity', () {
      test('should create SearchResponse from JSON correctly', () {
        // Arrange
        final json = {
          'results': [
            {
              'documentId': 'doc1',
              'spaceId': 'space1',
              'spaceName': 'Space',
              'title': 'Doc',
              'snippet': 'Snippet',
              'score': 0.9,
            },
          ],
          'total': 1,
          'took': 10,
        };

        // Act
        final response = SearchResponse.fromJson(json);

        // Assert
        expect(response.results.length, 1);
        expect(response.total, 1);
        expect(response.took, 10);
      });

      test('should handle null values in SearchResponse JSON', () {
        // Arrange
        final json = {
          'results': null,
          'total': null,
          'took': null,
        };

        // Act
        final response = SearchResponse.fromJson(json);

        // Assert
        expect(response.results.isEmpty, true);
        expect(response.total, 0);
        expect(response.took, 0);
      });

      test('should convert SearchResponse to JSON correctly', () {
        // Arrange
        final searchResponse = SearchResponse(
          results: [
            SearchResult(
              documentId: 'doc1',
              spaceId: 'space1',
              spaceName: 'Space',
              title: 'Doc',
              snippet: 'Snippet',
              score: 0.9,
            ),
          ],
          total: 1,
          took: 10,
        );

        // Act
        final json = searchResponse.toJson();

        // Assert
        expect(json['results'].length, 1);
        expect(json['total'], 1);
        expect(json['took'], 10);
      });
    });
  });
}
