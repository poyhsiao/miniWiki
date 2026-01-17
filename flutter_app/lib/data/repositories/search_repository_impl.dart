import 'dart:async';
import 'package:dio/dio.dart';
import 'package:miniwiki/core/network/api_client.dart';
import 'package:miniwiki/domain/entities/search_result.dart';
import 'package:miniwiki/domain/repositories/search_repository.dart';

/// Implementation of SearchRepository using the API
class SearchRepositoryImpl implements SearchRepository {
  final ApiClient apiClient;

  SearchRepositoryImpl({required this.apiClient});

  @override
  Future<(List<SearchResult>, int)> search({
    required String query,
    String? spaceId,
    int limit = 20,
    int offset = 0,
  }) async {
    try {
      final params = <String, dynamic>{
        'q': query,
        'limit': limit,
        'offset': offset,
      };
      if (spaceId != null) {
        params['spaceId'] = spaceId;
      }

      final response = await apiClient.get(
        '/search',
        queryParams: params,
      );

      final searchResponse = SearchResponse.fromJson(
        response.data as Map<String, dynamic>,
      );

      return (
        searchResponse.results,
        searchResponse.total,
      );
    } on DioException catch (e) {
      if (e.response == null) {
        throw ApiError(e.message ?? 'Network error', 'NETWORK_ERROR');
      }
      throw ApiError.fromResponse(e.response!);
    } catch (e) {
      throw ApiError(e.toString(), 'UNKNOWN_ERROR');
    }
  }

  @override
  Future<List<String>> getSuggestions(String query) async {
    try {
      final response = await apiClient.get(
        '/search/suggestions',
        queryParams: {'q': query},
      );

      return (response.data['suggestions'] as List<dynamic>)
          .map((s) => s.toString())
          .toList();
    } on DioException catch (e) {
      if (e.response == null) {
        throw ApiError(e.message ?? 'Network error', 'NETWORK_ERROR');
      }
      throw ApiError.fromResponse(e.response!);
    } catch (e) {
      throw ApiError(e.toString(), 'UNKNOWN_ERROR');
    }
  }
}
