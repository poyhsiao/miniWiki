import '../entities/search_result.dart';

/// Repository interface for search operations
abstract class SearchRepository {
  /// Search for documents matching the query
  Future<(List<SearchResult>, int)> search({
    required String query,
    String? spaceId,
    int limit = 20,
    int offset = 0,
  });

  /// Get search suggestions for autocomplete
  Future<List<String>> getSuggestions(String query);
}
