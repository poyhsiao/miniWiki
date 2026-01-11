import 'dart:io';
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

  factory ApiError.fromResponse(Response response) {
    final data = response.data as Map<String, dynamic>;
    return ApiError(
      data['message'] as String? ?? 'Unknown error',
      data['error'] as String? ?? 'API_ERROR',
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
    headers: {
      'Content-Type': 'application/json',
      'Accept': 'application/json',
    },
  ));

  dio.interceptors.add(LogInterceptor(
    request: true,
    responseBody: true,
    error: true,
  ));

  dio.interceptors.add(InterceptorsWrapper(
    onError: (error, handler) async {
      if (error.response?.statusCode == 401) {
        // TODO: Handle token refresh or logout
      }
      return handler.next(error);
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

  Future<Response> get(String path,
      {Map<String, dynamic>? queryParams, Options? options}) async {
    return _dio.get(path, queryParameters: queryParams, options: options);
  }

  Future<Response> post(String path, {dynamic data, Options? options}) async {
    return _dio.post(path, data: data, options: options);
  }

  Future<Response> put(String path, {dynamic data, Options? options}) async {
    return _dio.put(path, data: data, options: options);
  }

  Future<Response> patch(String path, {dynamic data, Options? options}) async {
    return _dio.patch(path, data: data, options: options);
  }

  Future<Response> delete(String path, {dynamic data, Options? options}) async {
    return _dio.delete(path, data: data, options: options);
  }

  void setAuthToken(String token) {
    _dio.options.headers['Authorization'] = 'Bearer $token';
  }

  void clearAuthToken() {
    _dio.options.headers.remove('Authorization');
  }
}
