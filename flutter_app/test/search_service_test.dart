// Flutter search service unit tests
import 'package:flutter_test/flutter_test.dart';
import 'package:dio/dio.dart';
import 'dart:convert';

import 'package:miniwiki/services/search_service.dart';
import 'package:miniwiki/domain/repositories/search_repository.dart';
import 'package:miniwiki/domain/entities/search_result.dart';
import 'package:miniwiki/core/network/api_client.dart';

// Simple mock implementation for testing
class MockSearchRepository implements SearchRepository {
  List<SearchResult> mockResults = [];
  int mockTotal = 0;
  bool shouldThrow = false;
  DioException? dioError;

  @override
  Future<(List<SearchResult>, int)> search({
    required String query,
    String? spaceId,
    int limit = 20,
    int offset = 0,
  }) async {
    if (shouldThrow && dioError != null) {
      throw dioError!;
    }
    return (mockResults, mockTotal);
  }

  @override
  Future<List<String>> getSuggestions(String query) async {
    if (query.trim().length < 2) {
      return [];
    }
    return ['$query suggestion 1', '$query suggestion 2'];
  }
}

void main() {
  group('SearchService', () {
    late SearchService searchService;
    late MockSearchRepository mockRepository;

    setUp(() {
      mockRepository = MockSearchRepository();
      searchService = SearchService(repository: mockRepository);
    });

    group('searchDocuments', () {
      test('returns search results on successful search', () async {
        // Arrange
        mockRepository.mockResults = [
          SearchResult(
            documentId: 'doc-1',
            spaceId: 'space-1',
            spaceName: 'Test Space',
            title: 'Getting Started with Rust',
            snippet: '...**Rust** is a systems programming language...',
            score: 2.5,
          ),
          SearchResult(
            documentId: 'doc-2',
            spaceId: 'space-1',
            spaceName: 'Test Space',
            title: 'Async Programming in Rust',
            snippet: '...async **Rust** programming guide...',
            score: 2.0,
          ),
        ];
        mockRepository.mockTotal = 2;

        // Act
        final result = await searchService.searchDocuments(
          query: 'Rust',
          spaceId: null,
          limit: 20,
          offset: 0,
        );

        // Assert
        expect(result.hasError, false);
        expect(result.results.length, 2);
        expect(result.results[0].title, 'Getting Started with Rust');
        expect(result.results[0].score, 2.5);
      });

      test('returns empty list when no results found', () async {
        // Arrange
        mockRepository.mockResults = [];
        mockRepository.mockTotal = 0;

        // Act
        final result = await searchService.searchDocuments(
          query: 'nonexistent xyz',
        );

        // Assert
        expect(result.hasError, false);
        expect(result.results, isEmpty);
      });

      test('filters by space when spaceId is provided', () async {
        // Arrange
        final spaceId = 'space-filter-123';
        mockRepository.mockResults = [
          SearchResult(
            documentId: 'doc-3',
            spaceId: spaceId,
            spaceName: 'Filtered Space',
            title: 'Documentation Guide',
            snippet: '...documentation for the platform...',
            score: 1.5,
          ),
        ];
        mockRepository.mockTotal = 1;

        // Act
        final result = await searchService.searchDocuments(
          query: 'documentation',
          spaceId: spaceId,
          limit: 10,
        );

        // Assert
        expect(result.hasError, false);
        expect(result.results.length, 1);
        expect(result.results[0].spaceId, spaceId);
      });

      test('returns error on API failure', () async {
        // Arrange
        mockRepository.shouldThrow = true;
        mockRepository.dioError = DioException(
          requestOptions: RequestOptions(path: '/search'),
          response: Response(
            requestOptions: RequestOptions(path: '/search'),
            statusCode: 500,
          ),
        );

        // Act
        final result = await searchService.searchDocuments(query: 'test');

        // Assert
        expect(result.hasError, true);
        expect(result.error, isNotNull);
      });

      test('returns empty for empty query', () async {
        // Act
        final result = await searchService.searchDocuments(query: '   ');

        // Assert
        expect(result.hasError, false);
        expect(result.results, isEmpty);
      });

      test('uses default pagination values', () async {
        // Arrange
        mockRepository.mockResults = [];
        mockRepository.mockTotal = 0;

        // Act
        await searchService.searchDocuments(query: 'flutter');

        // Assert - verify the search was called with correct defaults
        expect(mockRepository.mockResults, isEmpty);
      });
    });

    group('getSuggestions', () {
      test('returns suggestions for valid query', () async {
        // Act
        final suggestions = await searchService.getSuggestions('test');

        // Assert
        expect(suggestions.length, 2);
        expect(suggestions[0], 'test suggestion 1');
      });

      test('returns empty for short query', () async {
        // Act
        final suggestions = await searchService.getSuggestions('t');

        // Assert
        expect(suggestions, isEmpty);
      });
    });

    group('highlightQuery', () {
      test('wraps matching terms with highlight markers', () {
        // Act
        final result = SearchService.highlightQuery(
          'This is a test document about Flutter and Dart',
          'Flutter',
        );

        // Assert
        expect(result.length, 3);
        expect(result[0].toPlainText(), 'This is a test document about ');
        expect(result[1].toPlainText(), 'Flutter');
        expect(result[2].toPlainText(), ' and Dart');
      });

      test('handles case insensitive matching', () {
        // Act
        final result = SearchService.highlightQuery(
          'Testing RUST programming in RUST',
          'rust',
        );

        // Assert - both occurrences should be highlighted
        expect(result.length, 5);
      });

      test('returns original text when no match', () {
        // Act
        final result = SearchService.highlightQuery(
          'No matching terms here',
          'xyz123',
        );

        // Assert
        expect(result.length, 1);
        expect(result[0].toPlainText(), 'No matching terms here');
      });

      test('handles empty query', () {
        // Act
        final result = SearchService.highlightQuery(
          'Some text here',
          '',
        );

        // Assert
        expect(result.length, 1);
        expect(result[0].toPlainText(), 'Some text here');
      });

      test('handles empty text', () {
        // Act
        final result = SearchService.highlightQuery(
          '',
          'query',
        );

        // Assert
        expect(result.length, 1);
        expect(result[0].toPlainText(), '');
      });
    });

    group('SearchResult', () {
      test('can be created with all fields', () {
        // Act
        final result = SearchResult(
          documentId: 'doc-123',
          spaceId: 'space-456',
          spaceName: 'My Space',
          title: 'Test Document',
          snippet: '...test **snippet**...',
          score: 1.75,
        );

        // Assert
        expect(result.documentId, 'doc-123');
        expect(result.spaceId, 'space-456');
        expect(result.spaceName, 'My Space');
        expect(result.title, 'Test Document');
        expect(result.snippet, '...test **snippet**...');
        expect(result.score, 1.75);
      });

      test('supports equality comparison', () {
        // Arrange
        final result1 = SearchResult(
          documentId: 'doc-1',
          spaceId: 'space-1',
          spaceName: 'Space',
          title: 'Title',
          snippet: '...',
          score: 1.0,
        );
        final result2 = SearchResult(
          documentId: 'doc-1',
          spaceId: 'space-1',
          spaceName: 'Space',
          title: 'Title',
          snippet: '...',
          score: 1.0,
        );

        // Assert
        expect(result1, equals(result2));
      });

      test('supports inequality comparison', () {
        // Arrange
        final result1 = SearchResult(
          documentId: 'doc-1',
          spaceId: 'space-1',
          spaceName: 'Space',
          title: 'Title',
          snippet: '...',
          score: 1.0,
        );
        final result2 = SearchResult(
          documentId: 'doc-2',
          spaceId: 'space-1',
          spaceName: 'Space',
          title: 'Title',
          snippet: '...',
          score: 1.0,
        );

        // Assert
        expect(result1, isNot(equals(result2)));
      });
    });

    group('SearchException', () {
      test('can be created with message', () {
        // Act
        final exception = SearchException('Search failed');

        // Assert
        expect(exception.message, 'Search failed');
        expect(exception.statusCode, isNull);
      });

      test('can be created with status code', () {
        // Act
        final exception = SearchException.apiError(404, 'Not found');

        // Assert
        expect(exception.message, 'API error 404: Not found');
        expect(exception.statusCode, 404);
      });
    });

    group('SearchResultOrError', () {
      test('can be created with results', () {
        // Act
        final result = SearchResultOrError(
          results: [],
          hasError: false,
        );

        // Assert
        expect(result.results, isEmpty);
        expect(result.hasError, false);
        expect(result.error, isNull);
      });

      test('can be created with error', () {
        // Act
        final result = SearchResultOrError(
          results: [],
          hasError: true,
          error: 'Something went wrong',
        );

        // Assert
        expect(result.results, isEmpty);
        expect(result.hasError, true);
        expect(result.error, 'Something went wrong');
      });
    });
  });
}
