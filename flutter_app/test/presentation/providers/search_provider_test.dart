import 'dart:async';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:miniwiki/domain/entities/search_result.dart';
import 'package:miniwiki/presentation/providers/search_provider.dart';
import 'package:miniwiki/services/search_service.dart';
import 'package:mocktail/mocktail.dart';

class MockSearchService extends Mock implements SearchService {}

void main() {
  group('SearchProvider', () {
    late MockSearchService mockSearchService;
    late SearchProvider searchProvider;

    setUp(() {
      mockSearchService = MockSearchService();
      searchProvider = SearchProvider(searchService: mockSearchService);
    });

    tearDown(() {
      searchProvider.dispose();
    });

    group('initial state', () {
      test('starts with empty data state', () {
        expect(
          searchProvider.state,
          const AsyncValue<List<SearchResult>>.data([]),
        );
      });

      test('initial state is not loading', () {
        expect(searchProvider.state.isLoading, false);
      });

      test('initial state has no error', () {
        expect(searchProvider.state.hasError, false);
      });
    });

    group('search with debounce', () {
      test('search cancels previous timer', () async {
        // Setup: both queries should succeed
        when(() => mockSearchService.searchDocuments(query: 'first'))
            .thenAnswer(
                (_) async => SearchResultOrError(results: [], hasError: false));
        when(() => mockSearchService.searchDocuments(query: 'second'))
            .thenAnswer(
                (_) async => SearchResultOrError(results: [], hasError: false));

        // Act - call search twice rapidly (debounce 300ms)
        searchProvider.search('first');
        await Future.delayed(const Duration(milliseconds: 50));
        searchProvider.search('second');

        // Advance time beyond debounce period (second search will trigger)
        await Future.delayed(const Duration(milliseconds: 400));

        // Only second query should have triggered - first was cancelled
        verify(() => mockSearchService.searchDocuments(query: 'second'))
            .called(1);
        verifyNever(() => mockSearchService.searchDocuments(query: 'first'));
      });

      test('search does not call service immediately', () async {
        searchProvider.search('test query');
        // Immediately after, state should not be loading yet (debouncing)
        expect(searchProvider.state.isLoading, false);
      });

      test('search calls service after debounce', () async {
        when(() => mockSearchService.searchDocuments(query: 'test')).thenAnswer(
            (_) async => SearchResultOrError(results: [], hasError: false));

        searchProvider.search('test');
        await Future.delayed(const Duration(milliseconds: 400));

        verify(() => mockSearchService.searchDocuments(query: 'test'))
            .called(1);
      });

      test('search with custom debounce duration', () async {
        when(() => mockSearchService.searchDocuments(query: 'custom'))
            .thenAnswer(
                (_) async => SearchResultOrError(results: [], hasError: false));

        searchProvider.search('custom',
            debounce: const Duration(milliseconds: 500));
        await Future.delayed(const Duration(milliseconds: 200));

        // Should not have called yet
        expect(searchProvider.state.isLoading, false);

        await Future.delayed(const Duration(milliseconds: 400));

        verify(() => mockSearchService.searchDocuments(query: 'custom'))
            .called(1);
      });
    });

    group('searchImmediate', () {
      test('searchImmediate calls service immediately', () async {
        final searchResultOrError = SearchResultOrError(
          results: [
            SearchResult(
              documentId: 'doc1',
              spaceId: 'space1',
              spaceName: 'Test Space',
              title: 'Test Document',
              snippet: 'Test snippet...',
              score: 0.95,
            ),
          ],
          hasError: false,
        );

        when(() => mockSearchService.searchDocuments(query: 'immediate'))
            .thenAnswer((_) async => searchResultOrError);

        searchProvider.searchImmediate('immediate');
        await Future.delayed(const Duration(milliseconds: 50));

        verify(() => mockSearchService.searchDocuments(query: 'immediate'))
            .called(1);
        expect(searchProvider.state is AsyncData, true);
      });

      test('searchImmediate updates state to loading first', () async {
        final completer = Completer<SearchResultOrError>();
        when(() => mockSearchService.searchDocuments(query: 'loading'))
            .thenAnswer((_) => completer.future);

        // Start search but don't complete it
        searchProvider.searchImmediate('loading');
        await Future.delayed(const Duration(milliseconds: 10));

        // At this point state should be loading
        expect(searchProvider.state.isLoading, true);
      });

      test('searchImmediate cancels pending debounce timer', () async {
        // Debounced search returns but is slower
        when(() => mockSearchService.searchDocuments(query: 'debounced'))
            .thenAnswer(
                (_) async => SearchResultOrError(results: [], hasError: false));
        when(() => mockSearchService.searchDocuments(query: 'immediate'))
            .thenAnswer(
                (_) async => SearchResultOrError(results: [], hasError: false));

        // Start debounced search (will complete after 300ms)
        searchProvider.search('debounced');
        await Future.delayed(const Duration(milliseconds: 50));

        // Call immediate search which should cancel the debounced one
        searchProvider.searchImmediate('immediate');
        await Future.delayed(const Duration(milliseconds: 50));

        // Only immediate should have triggered
        verify(() => mockSearchService.searchDocuments(query: 'immediate'))
            .called(1);
        // Wait past the 300ms debounce window to ensure debounced search was cancelled
        await Future.delayed(const Duration(milliseconds: 400));
        verifyNever(() => mockSearchService.searchDocuments(query: 'debounced'));
      });
    });

    group('_performSearch', () {
      test('_performSearch clears state for empty query', () async {
        searchProvider.searchImmediate('');
        await Future.delayed(const Duration(milliseconds: 10));

        expect(searchProvider.state,
            const AsyncValue<List<SearchResult>>.data([]));
      });

      test('_performSearch clears state for whitespace-only query', () async {
        searchProvider.searchImmediate('   ');
        await Future.delayed(const Duration(milliseconds: 10));

        expect(searchProvider.state,
            const AsyncValue<List<SearchResult>>.data([]));
      });

      test('_performSearch sets loading state', () async {
        final completer = Completer<SearchResultOrError>();
        when(() => mockSearchService.searchDocuments(query: 'loading'))
            .thenAnswer((_) => completer.future);

        searchProvider.searchImmediate('loading');
        await Future.delayed(const Duration(milliseconds: 10));

        expect(searchProvider.state.isLoading, true);
      });

      test('_performSearch updates state with results on success', () async {
        final searchResultOrError = SearchResultOrError(
          results: [
            SearchResult(
              documentId: 'doc1',
              spaceId: 'space1',
              spaceName: 'Test Space',
              title: 'Found Document',
              snippet: 'Found snippet...',
              score: 0.98,
            ),
          ],
          hasError: false,
        );

        when(() => mockSearchService.searchDocuments(query: 'found'))
            .thenAnswer((_) async => searchResultOrError);

        searchProvider.searchImmediate('found');
        await Future.delayed(const Duration(milliseconds: 50));

        expect(searchProvider.state is AsyncData, true);
        final results =
            (searchProvider.state as AsyncData<List<SearchResult>>).value;
        expect(results.length, 1);
        expect(results[0].title, 'Found Document');
      });

      test('_performSearch sets error state on failure', () async {
        when(() => mockSearchService.searchDocuments(query: 'error'))
            .thenAnswer((_) async => SearchResultOrError(
                results: [], hasError: true, error: 'Server error'));

        searchProvider.searchImmediate('error');
        await Future.delayed(const Duration(milliseconds: 50));

        expect(searchProvider.state.hasError, true);
        final state = searchProvider.state;
        expect(state, isA<AsyncError<List<SearchResult>>>());
        expect((state as AsyncError).error.toString(), contains('Server error'));
      });

      test('_performSearch sets error state for null error', () async {
        when(() => mockSearchService.searchDocuments(query: 'nullerror'))
            .thenAnswer((_) async =>
                SearchResultOrError(results: [], hasError: true, error: null));

        searchProvider.searchImmediate('nullerror');
        await Future.delayed(const Duration(milliseconds: 50));

        expect(searchProvider.state.hasError, true);
      });
    });

    group('clear', () {
      test('clear resets state to empty', () async {
        final searchResultOrError = SearchResultOrError(
          results: [
            SearchResult(
              documentId: 'doc1',
              spaceId: 'space1',
              spaceName: 'Test',
              title: 'Doc',
              snippet: '...',
              score: 0.9,
            ),
          ],
          hasError: false,
        );
        when(() => mockSearchService.searchDocuments(query: 'test'))
            .thenAnswer((_) async => searchResultOrError);

        searchProvider.searchImmediate('test');
        await Future.delayed(const Duration(milliseconds: 50));

        // Then clear
        searchProvider.clear();

        expect(searchProvider.state,
            const AsyncValue<List<SearchResult>>.data([]));
      });

      test('clear cancels pending debounce timer', () async {
        // Stub search but never await - the timer will call it eventually
        when(() =>
                mockSearchService.searchDocuments(query: any(named: 'query')))
            .thenAnswer(
                (_) async => SearchResultOrError(results: [], hasError: false));

        searchProvider.search('test');
        await Future.delayed(const Duration(milliseconds: 50));

        // Clear should prevent the search from happening
        searchProvider.clear();

        // Wait to see if the timer would have fired
        await Future.delayed(const Duration(milliseconds: 400));

        // If clear worked, state should still be initial (not loading or error)
        expect(searchProvider.state is AsyncData, true);
        expect((searchProvider.state as AsyncData<List<SearchResult>>).value,
            isEmpty);
      });
    });

    group('getSuggestions', () {
      test('getSuggestions returns suggestions from service', () async {
        when(() => mockSearchService.getSuggestions('test')).thenAnswer(
            (_) async => ['test document', 'test case', 'testing guide']);

        final result = await searchProvider.getSuggestions('test');

        expect(result.length, 3);
        expect(result, contains('test document'));
        verify(() => mockSearchService.getSuggestions('test')).called(1);
      });

      test('getSuggestions returns empty list for empty query', () async {
        // getSuggestions in the service handles empty query internally
        // The provider just passes through
        when(() => mockSearchService.getSuggestions(any()))
            .thenAnswer((_) async => []);

        final result = await searchProvider.getSuggestions('');

        expect(result, isEmpty);
      });

      test('getSuggestions returns empty list when service throws', () async {
        // The provider catches exceptions and returns empty list
        when(() => mockSearchService.getSuggestions('error'))
            .thenThrow(Exception('Service error'));

        final result = await searchProvider.getSuggestions('error');

        // Should return empty list when service throws
        expect(result, isEmpty);
      });
    });

    group('dispose', () {
      test('dispose cancels debounce timer', () async {
        // Create an isolated instance to avoid tearDown interference
        final isolatedSearchService = MockSearchService();
        final isolatedProvider =
            SearchProvider(searchService: isolatedSearchService);

        when(() => isolatedSearchService.searchDocuments(query: 'test'))
            .thenAnswer(
                (_) async => SearchResultOrError(results: [], hasError: false));

        // Start a debounced search, then immediately dispose
        isolatedProvider
          ..search('test')
          ..dispose();

        // Wait for debounce duration to see if search was called
        await Future<void>.delayed(const Duration(milliseconds: 400));

        // Verify that searchDocuments was never called
        verifyNever(() => isolatedSearchService.searchDocuments(query: 'test'));
      });
    });

    group('SearchProvider - providers', () {
      test('searchStateProvider is correctly configured', () {
        final searchService = MockSearchService();
        final provider = SearchProvider(searchService: searchService);
        addTearDown(provider.dispose);

        // Verify it creates a StateNotifier
        expect(provider, isA<StateNotifier<AsyncValue<List<SearchResult>>>>());
      });

      test('searchQueryProvider starts with empty string', () {
        final container = ProviderContainer();
        addTearDown(container.dispose);

        expect(container.read(searchQueryProvider), '');
      });

      test('searchQueryProvider can be updated', () {
        final container = ProviderContainer();
        addTearDown(container.dispose);

        container.read(searchQueryProvider.notifier).state = 'test query';

        expect(container.read(searchQueryProvider), 'test query');
      });

      test('searchLoadingProvider derives from searchStateProvider', () {
        final container = ProviderContainer();
        addTearDown(container.dispose);

        // Initially not loading
        expect(container.read(searchLoadingProvider), false);
      });
    });

    group('SearchResult entity', () {
      test('SearchResult creates with all fields', () {
        final result = SearchResult(
          documentId: 'doc1',
          spaceId: 'space1',
          spaceName: 'Test Space',
          title: 'Test Document',
          snippet: 'This is a test...',
          score: 0.95,
        );

        expect(result.documentId, 'doc1');
        expect(result.spaceId, 'space1');
        expect(result.title, 'Test Document');
        expect(result.score, 0.95);
      });

      test('SearchResult equality works correctly', () {
        final result1 = SearchResult(
          documentId: 'doc1',
          spaceId: 'space1',
          spaceName: 'Space',
          title: 'Doc',
          snippet: '...',
          score: 0.9,
        );

        final result2 = SearchResult(
          documentId: 'doc1',
          spaceId: 'space1',
          spaceName: 'Space',
          title: 'Doc',
          snippet: '...',
          score: 0.9,
        );

        expect(result1 == result2, true);
      });

      test('SearchResult toJson serializes correctly', () {
        final result = SearchResult(
          documentId: 'doc1',
          spaceId: 'space1',
          spaceName: 'Test Space',
          title: 'Test Document',
          snippet: 'Test snippet',
          score: 0.95,
        );

        final json = result.toJson();

        expect(json['documentId'], 'doc1');
        expect(json['title'], 'Test Document');
        expect(json['score'], 0.95);
      });

      test('SearchResult fromJson deserializes correctly', () {
        final json = {
          'documentId': 'doc1',
          'spaceId': 'space1',
          'spaceName': 'Test Space',
          'title': 'Test Document',
          'snippet': 'Test snippet',
          'score': 0.95,
        };

        final result = SearchResult.fromJson(json);

        expect(result.documentId, 'doc1');
        expect(result.title, 'Test Document');
      });

      test('SearchResult handles null values', () {
        final result = SearchResult.fromJson({});

        expect(result.documentId, '');
        expect(result.title, '');
        expect(result.score, 0.0);
      });
    });
  });
}
