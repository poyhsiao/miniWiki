class NetworkError implements Exception {
  final String message;
  final int? statusCode;

  NetworkError(this.message, [this.statusCode]);

  @override
  String toString() => 'NetworkError: $message${statusCode != null ? ' (Status: $statusCode)' : ''}';

  factory NetworkError.fromJson(Map<String, dynamic> json, [int? statusCode]) {
    final message = json['message'] as String? ?? json['error'] as String? ?? 'Request failed';
    return NetworkError(
      message,
      statusCode,
    );
  }
}
