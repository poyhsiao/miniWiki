// Flutter search service unit tests
import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/annotations.dart';
import 'package:mockito/mockito.dart';
import 'package:dio/dio.dart';
import 'dart:convert';

import 'package:miniwiki/services/search_service.dart';
import 'package:miniwiki/domain/repositories/search_repository.dart';
import 'package:miniwiki/domain/entities/search_result.dart';
import 'search_service_test.mocks.dart';

@GenerateNiceMocks([MockSpec<SearchRepository>()])
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
        final expectedResults = [
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

        when(mockRepository.search(
          query: 'Rust',
          spaceId: null,
          limit: 20,
          offset: 0,
        )).thenAnswer((_) async => (expectedResults, 2));

        // Act
        final result = await searchService.searchDocuments(
          query: 'Rust',
          spaceId: null,
          limit: 20,
          offset: 0,
        );

        // Assert
        expect(result.isRight, true);
        result.fold(
          (error) => fail('Should not return error: $error'),
          (results) {
            expect(results.length, 2);
            expect(results[0].title, 'Getting Started with Rust');
            expect(results[0].score, 2.5);
          },
        );
      });

      test('returns empty list when no results found', () async {
        // Arrange
        when(mockRepository.search(
          query: 'nonexistent xyz',
          spaceId: null,
          limit: 20,
          offset: 0,
        )).thenAnswer((_) async => ([], 0));

        // Act
        final result = await searchService.searchDocuments(
          query: 'nonexistent xyz',
        );

        // Assert
        expect(result.isRight, true);
        result.fold(
          (error) => fail('Should not return error: $error'),
          (results) => expect(results, isEmpty),
        );
      });

      test('filters by space when spaceId is provided', () async {
        // Arrange
        final spaceId = 'space-filter-123';
        when(mockRepository.search(
          query: 'documentation',
          spaceId: spaceId,
          limit: 10,
          offset: 0,
        )).thenAnswer((_) async => (
              [
                SearchResult(
                  documentId: 'doc-3',
                  spaceId: spaceId,
                  spaceName: 'Filtered Space',
                  title: 'Documentation Guide',
                  snippet: '...documentation for the platform...',
                  score: 1.5,
                ),
              ],
              1,
            ));

        // Act
        final result = await searchService.searchDocuments(
          query: 'documentation',
          spaceId: spaceId,
          limit: 10,
        );

        // Assert
        expect(result.isRight, true);
        result.fold(
          (error) => fail('Should not return error: $error'),
          (results) {
            expect(results.length, 1);
            expect(results[0].spaceId, spaceId);
          },
        );
      });

      test('returns error on API failure', () async {
        // Arrange
        when(mockRepository.search(
          query: 'test',
          spaceId: null,
          limit: 20,
          offset: 0,
        )).thenThrow(DioException(
          requestOptions: RequestOptions(path: '/search'),
          response: Response(
            requestOptions: RequestOptions(path: '/search'),
            statusCode: 500,
          ),
        ));

        // Act
        final result = await searchService.searchDocuments(query: 'test');

        // Assert
        expect(result.isLeft, true);
        result.fold(
          (error) => expect(error, isA<SearchException>()),
          (_) => fail('Should return error'),
        );
      });

      test('uses default pagination values', () async {
        // Arrange
        when(mockRepository.search(
          query: 'flutter',
          spaceId: null,
          limit: 20,
          offset: 0,
        )).thenAnswer((_) async => ([], 0));

        // Act
        await searchService.searchDocuments(query: 'flutter');

        // Assert
        verify(mockRepository.search(
          query: 'flutter',
          spaceId: null,
          limit: 20,
          offset: 0,
        )).called(1);
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
        expect(result, 'This is a test document about **Flutter** and Dart');
      });

      test('handles case insensitive matching', () {
        // Act
        final result = SearchService.highlightQuery(
          'Testing RUST programming in RUST',
          'rust',
        );

        // Assert
        expect(result, 'Testing **RUST** programming in **RUST**');
      });

      test('returns original text when no match', () {
        // Act
        final result = SearchService.highlightQuery(
          'No matching terms here',
          'xyz123',
        );

        // Assert
        expect(result, 'No matching terms here');
      });
    });

    group('debounceSearch', () {
      test('delays search execution', () async {
        // Arrange
        when(mockRepository.search(
          query: 'test',
          spaceId: null,
          limit: 20,
          offset: 0,
        )).thenAnswer((_) async => ([], 0));

        // Act
        final future =
            searchService.debounceSearch('test', Duration(milliseconds: 100));

        // Verify not called immediately
        verifyNever(mockRepository.search(
            query: anyNamed('query'),
            spaceId: anyNamed('spaceId'),
            limit: anyNamed('limit'),
            offset: anyNamed('offset')));

        // Wait for debounce
        await future;

        // Assert - now should be called
        verify(mockRepository.search(
          query: 'test',
          spaceId: null,
          limit: 20,
          offset: 0,
        )).called(1);
      });
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
  });

  group('SearchException', () {
    test('can be created with message', () {
      // Act
      final exception = SearchException('Search failed');

      // Assert
      expect(exception.message, 'Search failed');
    });

    test('can be created with status code', () {
      // Act
      final exception = SearchException.apiError(404, 'Not found');

      // Assert
      expect(exception.message, 'API error 404: Not found');
      expect(exception.statusCode, 404);
    });
  });
}
