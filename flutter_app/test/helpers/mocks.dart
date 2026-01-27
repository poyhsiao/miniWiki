import 'package:mocktail/mocktail.dart';
import 'package:dio/dio.dart';
import 'package:miniwiki/core/network/api_client.dart';
import 'package:miniwiki/data/datasources/local_storage.dart';
import 'package:miniwiki/services/crdt_service.dart';
import 'package:miniwiki/services/offline_service.dart';
import 'package:miniwiki/services/websocket_service.dart';

/// Mock ApiClient
class MockApiClient extends Mock implements ApiClient {}

/// Mock LocalStorage
class MockLocalStorage extends Mock implements LocalStorage {}

/// Mock CRDTService
class MockCRDTService extends Mock implements CRDTService {}

/// Mock OfflineService
class MockOfflineService extends Mock implements OfflineService {}

/// Mock WebSocketService
class MockWebSocketService extends Mock implements WebSocketService {}

/// Mock Dio
class MockDio extends Mock implements Dio {}

/// Mock Response
class MockResponse extends Mock implements Response {}

/// 幫助創建 Mock Response 的工廠方法
class MockResponseFactory {
  static Response createSuccessResponse(dynamic data) {
    final response = MockResponse();
    when(() => response.data).thenReturn(data);
    when(() => response.statusCode).thenReturn(200);
    return response;
  }

  static Response createErrorResponse(int statusCode, String message) {
    final response = MockResponse();
    when(() => response.statusCode).thenReturn(statusCode);
    when(() => response.statusMessage).thenReturn(message);
    when(() => response.data).thenReturn({'error': message});
    return response;
  }

  static Response createCreatedResponse(dynamic data) {
    final response = MockResponse();
    when(() => response.data).thenReturn(data);
    when(() => response.statusCode).thenReturn(201);
    return response;
  }

  static Response createNotFoundResponse() {
    return createErrorResponse(404, 'Not Found');
  }

  static Response createUnauthorizedResponse() {
    return createErrorResponse(401, 'Unauthorized');
  }

  static Response createInternalServerError() {
    return createErrorResponse(500, 'Internal Server Error');
  }
}

/// 幫助設置 Mock ApiClient 的行為
class MockApiClientHelper {
  static void setupSuccessCall(
    MockApiClient mockClient,
    dynamic data, {
    int statusCode = 200,
  }) {
    when(() => mockClient.get(
      any(),
      queryParams: any(named: 'queryParams'),
    )).thenAnswer((_) async {
      return MockResponseFactory.createSuccessResponse({
        'data': data,
        'status': 'success',
      });
    });

    when(() => mockClient.post(
      any(),
      data: any(named: 'data'),
    )).thenAnswer((_) async {
      return MockResponseFactory.createSuccessResponse({
        'data': data,
        'status': 'success',
      });
    });

    when(() => mockClient.put(
      any(),
      data: any(named: 'data'),
    )).thenAnswer((_) async {
      return MockResponseFactory.createSuccessResponse({
        'data': data,
        'status': 'success',
      });
    });

    when(() => mockClient.delete(any())).thenAnswer((_) async {
      return MockResponseFactory.createSuccessResponse({
        'data': data,
        'status': 'success',
      });
    });
  }

  static void setupErrorCall(
    MockApiClient mockClient,
    String path, {
    int statusCode = 500,
    String message = 'Internal Server Error',
  }) {
    when(() => mockClient.get(
      any(),
      queryParams: any(named: 'queryParams'),
    )).thenThrow(Exception('Failed to fetch $path: status $statusCode - $message'));

    when(() => mockClient.post(
      any(),
      data: any(named: 'data'),
    )).thenThrow(Exception('Failed to create $path: status $statusCode - $message'));

    when(() => mockClient.put(
      any(),
      data: any(named: 'data'),
    )).thenThrow(Exception('Failed to update $path: status $statusCode - $message'));

    when(() => mockClient.delete(any())).thenThrow(
        Exception('Failed to delete $path: status $statusCode - $message'));
  }
}
