/// Search result entity returned from search queries
class SearchResult {
  final String documentId;
  final String spaceId;
  final String spaceName;
  final String title;
  final String snippet;
  final double score;

  SearchResult({
    required this.documentId,
    required this.spaceId,
    required this.spaceName,
    required this.title,
    required this.snippet,
    required this.score,
  });

  factory SearchResult.fromJson(Map<String, dynamic> json) {
    return SearchResult(
      documentId: json['documentId'] as String? ?? '',
      spaceId: json['spaceId'] as String? ?? '',
      spaceName: json['spaceName'] as String? ?? '',
      title: json['title'] as String? ?? '',
      snippet: json['snippet'] as String? ?? '',
      score: (json['score'] as num?)?.toDouble() ?? 0.0,
    );
  }

  Map<String, dynamic> toJson() {
    return {
      'documentId': documentId,
      'spaceId': spaceId,
      'spaceName': spaceName,
      'title': title,
      'snippet': snippet,
      'score': score,
    };
  }

  @override
  bool operator ==(Object other) {
    if (identical(this, other)) return true;
    return other is SearchResult &&
        other.documentId == documentId &&
        other.spaceId == spaceId &&
        other.spaceName == spaceName &&
        other.title == title &&
        other.snippet == snippet &&
        other.score == score;
  }

  @override
  int get hashCode => Object.hash(
        documentId,
        spaceId,
        spaceName,
        title,
        snippet,
        score,
      );
}

/// Search response containing results and metadata
class SearchResponse {
  final List<SearchResult> results;
  final int total;
  final int took; // Time in milliseconds

  SearchResponse({
    required this.results,
    required this.total,
    required this.took,
  });

  factory SearchResponse.fromJson(Map<String, dynamic> json) {
    return SearchResponse(
      results: (json['results'] as List<dynamic>?)
              ?.map((r) => SearchResult.fromJson(r as Map<String, dynamic>))
              .toList() ??
          [],
      total: json['total'] as int? ?? 0,
      took: json['took'] as int? ?? 0,
    );
  }

  Map<String, dynamic> toJson() {
    return {
      'results': results.map((r) => r.toJson()).toList(),
      'total': total,
      'took': took,
    };
  }
}

/// Search parameters for querying
class SearchParams {
  final String query;
  final String? spaceId;
  final int limit;
  final int offset;

  SearchParams({
    required this.query,
    this.spaceId,
    this.limit = 20,
    this.offset = 0,
  });

  Map<String, dynamic> toQueryParams() {
    final params = <String, dynamic>{
      'q': query,
      'limit': limit,
      'offset': offset,
    };
    if (spaceId != null) {
      params['spaceId'] = spaceId!;
    }
    return params;
  }
}
