import 'dart:async';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:miniwiki/domain/entities/search_result.dart';
import 'package:miniwiki/services/search_service.dart';

/// Search provider using Riverpod StateNotifier
class SearchProvider extends StateNotifier<AsyncValue<List<SearchResult>>> {
  final SearchService searchService;
  Timer? _debounceTimer;

  SearchProvider({required this.searchService})
      : super(const AsyncValue.data([]));

  /// Search with debounce
  void search(String query,
      {Duration debounce = const Duration(milliseconds: 300)}) {
    _debounceTimer?.cancel();
    _debounceTimer = Timer(debounce, () {
      _performSearch(query);
    });
  }

  /// Perform search immediately without debounce
  void searchImmediate(String query) {
    _debounceTimer?.cancel();
    _performSearch(query);
  }

  Future<void> _performSearch(String query) async {
    if (query.trim().isEmpty) {
      state = const AsyncValue.data([]);
      return;
    }

    state = const AsyncValue.loading();

    final result = await searchService.searchDocuments(query: query.trim());

    if (result.hasError) {
      state = AsyncValue.error(
        result.error ?? 'Unknown error',
        StackTrace.current,
      );
    } else {
      state = AsyncValue.data(result.results);
    }
  }

  /// Clear search results
  void clear() {
    _debounceTimer?.cancel();
    state = const AsyncValue.data([]);
  }

  /// Get suggestions for autocomplete
  Future<List<String>> getSuggestions(String query) async => searchService.getSuggestions(query);

  @override
  void dispose() {
    _debounceTimer?.cancel();
    super.dispose();
  }
}

/// Async search state provider
final searchStateProvider =
    StateNotifierProvider<SearchProvider, AsyncValue<List<SearchResult>>>(
        (ref) {
  final searchService = ref.watch(searchServiceProvider);
  return SearchProvider(searchService: searchService);
});

/// Simple search query provider
final searchQueryProvider = StateProvider<String>((ref) => '');

/// Derived loading state from the main search provider
final searchLoadingProvider = Provider<bool>((ref) => ref.watch(searchStateProvider).isLoading);
