import 'dart:convert';

import 'package:dio/dio.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:miniwiki/core/config/app_config_provider.dart';

abstract class NetworkError implements Exception {
  final String message;
  final int? statusCode;

  const NetworkError(this.message, [this.statusCode]);
}

class ApiError extends NetworkError {
  final String code;

  ApiError(String message, this.code, [int? statusCode])
      : super(message, statusCode);

  factory ApiError.fromResponse(Response<dynamic> response) {
    // Defensively handle different response.data types
    String? message;
    String? code;

    if (response.data == null) {
      message = 'Unknown error';
      code = 'API_ERROR';
    } else if (response.data is Map<String, dynamic>) {
      final data = response.data as Map<String, dynamic>;
      message = data['message'] as String? ?? 'Unknown error';
      code = data['error'] as String? ?? 'API_ERROR';
    } else if (response.data is String) {
      // Try to parse as JSON string, otherwise use the string as message
      try {
        final decoded = jsonDecode(response.data as String);
        if (decoded is Map<String, dynamic>) {
          message = decoded['message'] as String? ?? response.data as String;
          code = decoded['error'] as String? ?? 'API_ERROR';
        } else {
          // Decoded JSON is not a Map, treat as plain string message
          message = response.data as String;
          code = 'API_ERROR';
        }
      } catch (_) {
        message = response.data as String;
        code = 'API_ERROR';
      }
    } else {
      // Fallback to statusMessage or toString()
      message = response.statusMessage ??
          response.data?.toString() ??
          'Unknown error';
      code = 'API_ERROR';
    }

    return ApiError(
      message,
      code,
      response.statusCode,
    );
  }
}

class UnauthorizedError extends ApiError {
  UnauthorizedError([String message = 'Unauthorized'])
      : super(message, 'UNAUTHORIZED', 401);
}

class NotFoundError extends ApiError {
  NotFoundError([String message = 'Not found'])
      : super(message, 'NOT_FOUND', 404);
}

class ValidationError extends ApiError {
  ValidationError([String message = 'Validation error'])
      : super(message, 'VALIDATION_ERROR', 400);
}

final dioProvider = Provider<Dio>((ref) {
  final config = ref.watch(appConfigProvider);

  final dio = Dio(BaseOptions(
    baseUrl: config,
    connectTimeout: const Duration(seconds: 30),
    receiveTimeout: const Duration(seconds: 30),
    validateStatus: (status) => true,
    headers: {
      'Content-Type': 'application/json',
      'Accept': 'application/json',
    },
  ));

  dio.interceptors.add(LogInterceptor(
    responseBody: true,
  ));

  dio.interceptors.add(InterceptorsWrapper(
    onResponse: (response, handler) {
      if (response.statusCode == 401) {
        // TODO: Handle token refresh or logout
      }
      return handler.next(response);
    },
  ));

  return dio;
});

final apiClientProvider = Provider<ApiClient>((ref) {
  final dio = ref.watch(dioProvider);
  return ApiClient(dio);
});

class ApiClient {
  final Dio _dio;

  ApiClient(this._dio);

  /// Expose the Dio instance for advanced usage
  Dio get dio => _dio;

  factory ApiClient.defaultInstance({String? baseUrl}) {
    const envBaseUrl = String.fromEnvironment('API_BASE_URL',
        defaultValue: 'http://localhost:8080/api/v1');
    final dio = Dio(BaseOptions(
      baseUrl: baseUrl ?? envBaseUrl,
      connectTimeout: const Duration(seconds: 30),
      receiveTimeout: const Duration(seconds: 30),
      validateStatus: (status) => true,
      headers: {
        'Content-Type': 'application/json',
        'Accept': 'application/json',
      },
    ));

    dio.interceptors.add(LogInterceptor(
      responseBody: true,
    ));

    dio.interceptors.add(InterceptorsWrapper(
      onResponse: (response, handler) {
        if (response.statusCode == 401) {
          // TODO: Handle token refresh or logout
        }
        return handler.next(response);
      },
    ));

    return ApiClient(dio);
  }

  Future<Response<dynamic>> get(
    String path, {
    Map<String, dynamic>? queryParams,
    Options? options,
  }) async =>
      _dio.get<dynamic>(
        path,
        queryParameters: queryParams,
        options: options,
      );

  Future<Response<dynamic>> post(
    String path, {
    Object? data,
    Options? options,
  }) async =>
      _dio.post<dynamic>(path, data: data, options: options);

  Future<Response<dynamic>> put(
    String path, {
    Object? data,
    Options? options,
  }) async =>
      _dio.put<dynamic>(path, data: data, options: options);

  Future<Response<dynamic>> patch(
    String path, {
    Object? data,
    Options? options,
  }) async =>
      _dio.patch<dynamic>(path, data: data, options: options);

  Future<Response<dynamic>> delete(
    String path, {
    Object? data,
    Options? options,
  }) async =>
      _dio.delete<dynamic>(path, data: data, options: options);

  void setAuthToken(String token) {
    _dio.options.headers['Authorization'] = 'Bearer $token';
  }

  void clearAuthToken() {
    _dio.options.headers.remove('Authorization');
  }
}
