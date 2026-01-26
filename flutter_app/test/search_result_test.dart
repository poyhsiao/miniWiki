import 'package:flutter_test/flutter_test.dart';
import 'package:miniwiki/domain/entities/search_result.dart';

void main() {
  group('SearchResult Entity Tests', () {
    test('SearchResult can be created with all fields', () {
      // Arrange & Act
      final result = SearchResult(
        documentId: 'doc1',
        spaceId: 'space1',
        spaceName: 'Test Space',
        title: 'Test Document',
        snippet: 'This is a test snippet...',
        score: 0.95,
      );

      // Assert
      expect(result.documentId, 'doc1');
      expect(result.spaceId, 'space1');
      expect(result.spaceName, 'Test Space');
      expect(result.title, 'Test Document');
      expect(result.snippet, 'This is a test snippet...');
      expect(result.score, 0.95);
    });

    test('SearchResult with zero score', () {
      // Arrange & Act
      final result = SearchResult(
        documentId: 'doc1',
        spaceId: 'space1',
        spaceName: 'Test Space',
        title: 'Test Document',
        snippet: 'Low relevance',
        score: 0.0,
      );

      // Assert
      expect(result.score, 0.0);
    });

    test('SearchResult with perfect score', () {
      // Arrange & Act
      final result = SearchResult(
        documentId: 'doc1',
        spaceId: 'space1',
        spaceName: 'Test Space',
        title: 'Test Document',
        snippet: 'Perfect match',
        score: 1.0,
      );

      // Assert
      expect(result.score, 1.0);
    });

    test('SearchResult toJson creates correct JSON', () {
      // Arrange
      final result = SearchResult(
        documentId: 'doc1',
        spaceId: 'space1',
        spaceName: 'Test Space',
        title: 'Test Document',
        snippet: 'Test snippet',
        score: 0.85,
      );

      // Act
      final json = result.toJson();

      // Assert
      expect(json['documentId'], 'doc1');
      expect(json['spaceId'], 'space1');
      expect(json['spaceName'], 'Test Space');
      expect(json['title'], 'Test Document');
      expect(json['snippet'], 'Test snippet');
      expect(json['score'], 0.85);
    });

    test('SearchResult fromJson creates instance correctly', () {
      // Arrange
      final json = {
        'documentId': 'doc1',
        'spaceId': 'space1',
        'spaceName': 'Test Space',
        'title': 'Test Document',
        'snippet': 'Test snippet',
        'score': 0.85,
      };

      // Act
      final result = SearchResult.fromJson(json);

      // Assert
      expect(result.documentId, 'doc1');
      expect(result.spaceId, 'space1');
      expect(result.spaceName, 'Test Space');
      expect(result.title, 'Test Document');
      expect(result.snippet, 'Test snippet');
      expect(result.score, 0.85);
    });

    test('SearchResult fromJson handles null values with defaults', () {
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

    test('SearchResult fromJson handles missing fields', () {
      // Arrange
      final json = <String, dynamic>{};

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

    test('SearchResult equality works correctly', () {
      // Arrange
      final result1 = SearchResult(
        documentId: 'doc1',
        spaceId: 'space1',
        spaceName: 'Test Space',
        title: 'Test Document',
        snippet: 'Test snippet',
        score: 0.85,
      );

      final result2 = SearchResult(
        documentId: 'doc1',
        spaceId: 'space1',
        spaceName: 'Test Space',
        title: 'Test Document',
        snippet: 'Test snippet',
        score: 0.85,
      );

      final result3 = SearchResult(
        documentId: 'doc2',
        spaceId: 'space1',
        spaceName: 'Test Space',
        title: 'Different Document',
        snippet: 'Different snippet',
        score: 0.75,
      );

      // Assert
      expect(result1, equals(result2));
      expect(result1, isNot(equals(result3)));
      expect(result1.hashCode, equals(result2.hashCode));
    });

    test('SearchResult with integer score in JSON', () {
      // Arrange
      final json = {
        'documentId': 'doc1',
        'spaceId': 'space1',
        'spaceName': 'Test Space',
        'title': 'Test Document',
        'snippet': 'Test snippet',
        'score': 1, // Integer instead of double
      };

      // Act
      final result = SearchResult.fromJson(json);

      // Assert
      expect(result.score, 1.0);
    });
  });

  group('SearchResponse Tests', () {
    test('SearchResponse can be created with all fields', () {
      // Arrange & Act
      final results = [
        SearchResult(
          documentId: 'doc1',
          spaceId: 'space1',
          spaceName: 'Test Space',
          title: 'Document 1',
          snippet: 'Snippet 1',
          score: 0.95,
        ),
        SearchResult(
          documentId: 'doc2',
          spaceId: 'space1',
          spaceName: 'Test Space',
          title: 'Document 2',
          snippet: 'Snippet 2',
          score: 0.85,
        ),
      ];

      final response = SearchResponse(
        results: results,
        total: 10,
        took: 50,
      );

      // Assert
      expect(response.results.length, 2);
      expect(response.total, 10);
      expect(response.took, 50);
    });

    test('SearchResponse toJson creates correct JSON', () {
      // Arrange
      final results = [
        SearchResult(
          documentId: 'doc1',
          spaceId: 'space1',
          spaceName: 'Test Space',
          title: 'Document 1',
          snippet: 'Snippet 1',
          score: 0.95,
        ),
      ];

      final response = SearchResponse(
        results: results,
        total: 5,
        took: 25,
      );

      // Act
      final json = response.toJson();

      // Assert
      expect(json['results'], isList);
      expect(json['results'].length, 1);
      expect(json['total'], 5);
      expect(json['took'], 25);
      expect(json['results'][0]['documentId'], 'doc1');
    });

    test('SearchResponse fromJson creates instance correctly', () {
      // Arrange
      final json = {
        'results': [
          {
            'documentId': 'doc1',
            'spaceId': 'space1',
            'spaceName': 'Test Space',
            'title': 'Document 1',
            'snippet': 'Snippet 1',
            'score': 0.95,
          },
        ],
        'total': 5,
        'took': 25,
      };

      // Act
      final response = SearchResponse.fromJson(json);

      // Assert
      expect(response.results.length, 1);
      expect(response.total, 5);
      expect(response.took, 25);
      expect(response.results[0].documentId, 'doc1');
    });

    test('SearchResponse fromJson handles empty results', () {
      // Arrange
      final json = {
        'results': <dynamic>[],
        'total': 0,
        'took': 10,
      };

      // Act
      final response = SearchResponse.fromJson(json);

      // Assert
      expect(response.results.isEmpty, true);
      expect(response.total, 0);
      expect(response.took, 10);
    });

    test('SearchResponse fromJson handles null values with defaults', () {
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

    test('SearchResponse with no results found', () {
      // Arrange & Act
      final response = SearchResponse(
        results: [],
        total: 0,
        took: 15,
      );

      // Assert
      expect(response.results.isEmpty, true);
      expect(response.total, 0);
      expect(response.took, 15);
    });
  });

  group('SearchParams Tests', () {
    test('SearchParams can be created with all fields', () {
      // Arrange & Act
      final params = SearchParams(
        query: 'test query',
        spaceId: 'space1',
        limit: 10,
        offset: 20,
      );

      // Assert
      expect(params.query, 'test query');
      expect(params.spaceId, 'space1');
      expect(params.limit, 10);
      expect(params.offset, 20);
    });

    test('SearchParams with default values', () {
      // Arrange & Act
      final params = SearchParams(
        query: 'test query',
      );

      // Assert
      expect(params.query, 'test query');
      expect(params.spaceId, isNull);
      expect(params.limit, 20);
      expect(params.offset, 0);
    });

    test('SearchParams toQueryParams creates correct map', () {
      // Arrange
      final params = SearchParams(
        query: 'test query',
        spaceId: 'space1',
        limit: 10,
        offset: 20,
      );

      // Act
      final queryParams = params.toQueryParams();

      // Assert
      expect(queryParams['q'], 'test query');
      expect(queryParams['spaceId'], 'space1');
      expect(queryParams['limit'], 10);
      expect(queryParams['offset'], 20);
    });

    test('SearchParams toQueryParams excludes spaceId when null', () {
      // Arrange
      final params = SearchParams(
        query: 'test query',
      );

      // Act
      final queryParams = params.toQueryParams();

      // Assert
      expect(queryParams['q'], 'test query');
      expect(queryParams.containsKey('spaceId'), false);
      expect(queryParams['limit'], 20);
      expect(queryParams['offset'], 0);
    });

    test('SearchParams with pagination', () {
      // Arrange & Act
      final page1Params = SearchParams(
        query: 'test',
        limit: 10,
      );

      final page2Params = SearchParams(
        query: 'test',
        limit: 10,
        offset: 10,
      );

      // Assert
      expect(page1Params.offset, 0);
      expect(page2Params.offset, 10);
    });

    test('SearchParams with large limit', () {
      // Arrange & Act
      final params = SearchParams(
        query: 'test',
        limit: 100,
      );

      // Assert
      expect(params.limit, 100);
    });

    test('SearchParams with custom offset only', () {
      // Arrange & Act
      final params = SearchParams(
        query: 'test',
        offset: 50,
      );

      // Assert
      expect(params.offset, 50);
      expect(params.limit, 20); // Default
    });
  });
}
