import 'dart:async';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:miniwiki/core/network/api_client.dart';
import '../../data/repositories/search_repository_impl.dart';
import '../../domain/entities/search_result.dart';
import '../../domain/repositories/search_repository.dart';

/// Search service exception
class SearchException implements Exception {
  final String message;
  final int? statusCode;

  SearchException(this.message, [this.statusCode]);

  factory SearchException.apiError(int statusCode, String message) {
    return SearchException('API error $statusCode: $message', statusCode);
  }

  @override
  String toString() => message;
}

/// Search result wrapper with error handling
class SearchResultOrError {
  final List<SearchResult> results;
  final bool hasError;
  final String? error;

  SearchResultOrError({
    required this.results,
    required this.hasError,
    this.error,
  });
}

/// Search service for document search operations
class SearchService {
  final SearchRepository repository;

  SearchService({required this.repository});

  /// Search for documents
  Future<SearchResultOrError> searchDocuments({
    required String query,
    String? spaceId,
    int limit = 20,
    int offset = 0,
  }) async {
    if (query.trim().isEmpty) {
      return SearchResultOrError(results: [], hasError: false);
    }

    try {
      final (results, total) = await repository.search(
        query: query.trim(),
        spaceId: spaceId,
        limit: limit,
        offset: offset,
      );
      return SearchResultOrError(results: results, hasError: false);
    } on ApiError catch (e) {
      return SearchResultOrError(
        results: [],
        hasError: true,
        error: 'API error ${e.statusCode ?? 500}: ${e.message}',
      );
    } catch (e) {
      return SearchResultOrError(
          results: [], hasError: true, error: 'Search failed: $e');
    }
  }

  /// Get search suggestions for autocomplete
  Future<List<String>> getSuggestions(String query) async {
    if (query.trim().length < 2) {
      return [];
    }

    try {
      final suggestions = await repository.getSuggestions(query.trim());
      return suggestions;
    } catch (e) {
      return [];
    }
  }

  /// Highlight matching terms in text, returning TextSpans for rich text rendering
  static List<TextSpan> highlightQuery(String text, String query,
      {TextStyle? baseStyle, TextStyle? highlightStyle}) {
    if (query.isEmpty || text.isEmpty) {
      return [TextSpan(text: text, style: baseStyle)];
    }

    final regex = RegExp(
      RegExp.escape(query),
      caseSensitive: false,
    );

    final spans = <TextSpan>[];
    int lastEnd = 0;

    for (final match in regex.allMatches(text)) {
      // Add text before the match
      if (match.start > lastEnd) {
        spans.add(TextSpan(
          text: text.substring(lastEnd, match.start),
          style: baseStyle,
        ));
      }

      // Add highlighted match
      spans.add(TextSpan(
        text: match.group(0),
        style: highlightStyle ??
            TextStyle(
              fontWeight: FontWeight.bold,
              backgroundColor: Colors.yellow.withValues(alpha: 0.3),
            ),
      ));

      lastEnd = match.end;
    }

    // Add remaining text after last match
    if (lastEnd < text.length) {
      spans.add(TextSpan(
        text: text.substring(lastEnd),
        style: baseStyle,
      ));
    }

    return spans.isNotEmpty ? spans : [TextSpan(text: text, style: baseStyle)];
  }
}

/// Riverpod provider for SearchService
final searchServiceProvider = Provider<SearchService>((ref) {
  final repository = ref.watch(searchRepositoryProvider);
  return SearchService(repository: repository);
});

/// Search repository provider
final searchRepositoryProvider = Provider<SearchRepository>((ref) {
  final apiClient = ref.watch(apiClientProvider);
  return SearchRepositoryImpl(apiClient: apiClient);
});
