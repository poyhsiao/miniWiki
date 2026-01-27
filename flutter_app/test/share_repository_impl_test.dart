import 'package:dio/dio.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:miniwiki/core/network/api_client.dart';
import 'package:miniwiki/core/network/network_error.dart' as ne;
import 'package:miniwiki/data/repositories/share_repository_impl.dart';
import 'package:miniwiki/domain/entities/share_link.dart';
import 'package:mocktail/mocktail.dart';

class MockApiClient extends Mock implements ApiClient {}

class MockDio extends Mock implements Dio {}

void main() {
  // Register fallback values for mocktail
  registerFallbackValue(RequestOptions(path: ''));

  group('ShareRepositoryImpl', () {
    late ShareRepositoryImpl repository;
    late MockDio mockDio;

    setUp(() {
      mockDio = MockDio();
      final mockApiClient = MockApiClient();
      when(() => mockApiClient.dio).thenReturn(mockDio);
      repository = ShareRepositoryImpl(
        apiClient: mockApiClient,
        baseUrl: 'http://localhost:3000',
      );
    });

    group('createShareLink', () {
      test('should create share link successfully', () async {
        // Arrange
        final requestData = CreateShareLinkRequest(
          documentId: 'doc1',
          permission: 'view',
          accessCode: '1234',
        );

        final responseData = {
          'id': 'share1',
          'document_id': 'doc1',
          'document_title': 'Test Document',
          'token': 'abc123',
          'requires_access_code': true,
          'permission': 'view',
          'is_active': true,
          'created_at': '2025-01-01T00:00:00Z',
          'access_count': 0,
          'created_by': 'user1',
          'access_code': '1234',
        };

        final response = Response<Map<String, dynamic>>(
          data: responseData,
          statusCode: 201,
          requestOptions: RequestOptions(path: '/api/v1/documents/doc1/share'),
        );

        when(() => mockDio.post<Map<String, dynamic>>(
          any(),
          data: any(named: 'data'),
        )).thenAnswer((_) async => response);

        // Act
        final result = await repository.createShareLink(requestData);

        // Assert
        expect(result.id, 'share1');
        expect(result.documentId, 'doc1');
        expect(result.token, 'abc123');
        expect(result.requiresAccessCode, true);
        verify(() => mockDio.post<Map<String, dynamic>>(
          any(),
          data: any(named: 'data'),
        )).called(1);
      });

      test('should throw NetworkError when creation fails', () async {
        // Arrange
        final requestData = CreateShareLinkRequest(
          documentId: 'doc1',
          permission: 'view',
        );

        final response = Response<Map<String, dynamic>>(
          data: null,
          statusCode: 400,
          statusMessage: 'Bad Request',
          requestOptions: RequestOptions(path: '/api/v1/documents/doc1/share'),
        );

        when(() => mockDio.post<Map<String, dynamic>>(
          any(),
          data: any(named: 'data'),
        )).thenAnswer((_) async => response);

        // Act & Assert
        await expectLater(
          repository.createShareLink(requestData),
          throwsA(isA<ne.NetworkError>()),
        );
      });

      test('should handle creation with expiration date', () async {
        // Arrange
        final expirationDate = DateTime.parse('2025-12-31T23:59:59Z');
        final requestData = CreateShareLinkRequest(
          documentId: 'doc1',
          permission: 'comment',
          expiresAt: expirationDate,
        );

        final responseData = {
          'id': 'share2',
          'document_id': 'doc1',
          'document_title': 'Test Document',
          'token': 'xyz789',
          'requires_access_code': false,
          'expires_at': '2025-12-31T23:59:59Z',
          'permission': 'comment',
          'is_active': true,
          'created_at': '2025-01-01T00:00:00Z',
          'access_count': 0,
          'created_by': 'user1',
        };

        final response = Response<Map<String, dynamic>>(
          data: responseData,
          statusCode: 201,
          requestOptions: RequestOptions(path: '/api/v1/documents/doc1/share'),
        );

        when(() => mockDio.post<Map<String, dynamic>>(
          any(),
          data: any(named: 'data'),
        )).thenAnswer((_) async => response);

        // Act
        final result = await repository.createShareLink(requestData);

        // Assert
        expect(result.expiresAt, expirationDate);
        expect(result.permission, 'comment');
      });
    });

    group('getShareLinks', () {
      test('should return list of share links', () async {
        // Arrange
        final responseData = [
          {
            'id': 'share1',
            'document_id': 'doc1',
            'document_title': 'Test Document',
            'token': 'abc123',
            'requires_access_code': false,
            'permission': 'view',
            'is_active': true,
            'created_at': '2025-01-01T00:00:00Z',
            'access_count': 5,
            'created_by': 'user1',
          },
          {
            'id': 'share2',
            'document_id': 'doc1',
            'document_title': 'Test Document',
            'token': 'def456',
            'requires_access_code': true,
            'permission': 'comment',
            'is_active': true,
            'created_at': '2025-01-02T00:00:00Z',
            'access_count': 2,
            'created_by': 'user2',
          },
        ];

        final response = Response<List<dynamic>>(
          data: responseData,
          statusCode: 200,
          requestOptions: RequestOptions(path: '/api/v1/documents/doc1/share'),
        );

        when(() => mockDio.get<List<dynamic>>(any())).thenAnswer((_) async => response);

        // Act
        final result = await repository.getShareLinks('doc1');

        // Assert
        expect(result.length, 2);
        expect(result[0].id, 'share1');
        expect(result[1].id, 'share2');
        expect(result[0].token, 'abc123');
        expect(result[1].requiresAccessCode, true);
        verify(() => mockDio.get<List<dynamic>>(any())).called(1);
      });

      test('should return empty list when no shares exist', () async {
        // Arrange
        final response = Response<List<dynamic>>(
          data: [],
          statusCode: 200,
          requestOptions: RequestOptions(path: '/api/v1/documents/doc1/share'),
        );

        when(() => mockDio.get<List<dynamic>>(any())).thenAnswer((_) async => response);

        // Act
        final result = await repository.getShareLinks('doc1');

        // Assert
        expect(result.isEmpty, true);
      });

      test('should throw NetworkError on API error', () async {
        // Arrange
        final response = Response<List<dynamic>>(
          data: null,
          statusCode: 500,
          statusMessage: 'Internal Server Error',
          requestOptions: RequestOptions(path: '/api/v1/documents/doc1/share'),
        );

        when(() => mockDio.get<List<dynamic>>(any())).thenAnswer((_) async => response);

        // Act & Assert
        await expectLater(
          repository.getShareLinks('doc1'),
          throwsA(isA<ne.NetworkError>()),
        );
      });
    });

    group('getShareLinkByToken', () {
      test('should return share link when found', () async {
        // Arrange
        final responseData = {
          'id': 'share1',
          'document_id': 'doc1',
          'document_title': 'Test Document',
          'token': 'abc123',
          'requires_access_code': false,
          'permission': 'view',
          'is_active': true,
          'created_at': '2025-01-01T00:00:00Z',
          'access_count': 10,
          'created_by': 'user1',
        };

        final response = Response<Map<String, dynamic>>(
          data: responseData,
          statusCode: 200,
          requestOptions: RequestOptions(path: '/api/v1/share/abc123'),
        );

        when(() => mockDio.get<Map<String, dynamic>>(any()))
            .thenAnswer((_) async => response);

        // Act
        final result = await repository.getShareLinkByToken('abc123');

        // Assert
        expect(result, isNotNull);
        expect(result!.id, 'share1');
        expect(result.token, 'abc123');
        expect(result.requiresAccessCode, false);
      });

      test('should return null when share link not found (404)', () async {
        // Arrange
        final response = Response<Map<String, dynamic>>(
          data: null,
          statusCode: 404,
          requestOptions: RequestOptions(path: '/api/v1/share/nonexistent'),
        );

        when(() => mockDio.get<Map<String, dynamic>>(any()))
            .thenAnswer((_) async => response);

        // Act
        final result = await repository.getShareLinkByToken('nonexistent');

        // Assert
        expect(result, isNull);
      });

      test('should return partial share link when access code required', () async {
        // Arrange
        final responseData = {
          'id': 'share1',
          'document_id': 'doc1',
          'document_title': 'Protected Document',
          'token': 'abc123',
          'requires_access_code': true,
          'permission': 'view',
          'is_active': true,
          'created_at': '2025-01-01T00:00:00Z',
          'access_count': 0,
          'created_by': 'user1',
        };

        final response = Response<Map<String, dynamic>>(
          data: responseData,
          statusCode: 200,
          requestOptions: RequestOptions(path: '/api/v1/share/abc123'),
        );

        when(() => mockDio.get<Map<String, dynamic>>(any()))
            .thenAnswer((_) async => response);

        // Act
        final result = await repository.getShareLinkByToken('abc123');

        // Assert
        expect(result, isNotNull);
        expect(result!.requiresAccessCode, true);
        expect(result.accessCount, 0);
      });

      test('should throw NetworkError on other API errors', () async {
        // Arrange
        final response = Response<Map<String, dynamic>>(
          data: null,
          statusCode: 500,
          statusMessage: 'Server Error',
          requestOptions: RequestOptions(path: '/api/v1/share/abc123'),
        );

        when(() => mockDio.get<Map<String, dynamic>>(any()))
            .thenAnswer((_) async => response);

        // Act & Assert
        await expectLater(
          repository.getShareLinkByToken('abc123'),
          throwsA(isA<ne.NetworkError>()),
        );
      });
    });

    group('verifyAccessCode', () {
      test('should verify access code successfully', () async {
        // Arrange
        final responseData = {
          'id': 'share1',
          'document_id': 'doc1',
          'document_title': 'Protected Document',
          'document_content': {
            'type': 'Y.Doc',
            'content': 'Sample content',
          },
          'permission': 'view',
          'verified': true,
        };

        final response = Response<Map<String, dynamic>>(
          data: responseData,
          statusCode: 200,
          requestOptions: RequestOptions(path: '/api/v1/share/abc123/verify'),
        );

        when(() => mockDio.post<Map<String, dynamic>>(
          any(),
          data: any(named: 'data'),
        )).thenAnswer((_) async => response);

        // Act
        final result = await repository.verifyAccessCode('abc123', '1234');

        // Assert
        expect(result.verified, true);
        expect(result.id, 'share1');
        expect(result.documentId, 'doc1');
        expect(result.permission, 'view');
        verify(() => mockDio.post<Map<String, dynamic>>(
          any(),
          data: any(named: 'data'),
        )).called(1);
      });

      test('should throw NetworkError when access code is invalid', () async {
        // Arrange
        final response = Response<Map<String, dynamic>>(
          data: null,
          statusCode: 401,
          statusMessage: 'Unauthorized',
          requestOptions: RequestOptions(path: '/api/v1/share/abc123/verify'),
        );

        when(() => mockDio.post<Map<String, dynamic>>(
          any(),
          data: any(named: 'data'),
        )).thenAnswer((_) async => response);

        // Act & Assert
        await expectLater(
          repository.verifyAccessCode('abc123', 'wrong'),
          throwsA(isA<ne.NetworkError>()),
        );
      });

      test('should include expiration date in verification response', () async {
        // Arrange
        final expirationDate = '2025-12-31T23:59:59Z';
        final responseData = {
          'id': 'share1',
          'document_id': 'doc1',
          'document_title': 'Expiring Document',
          'document_content': {
            'type': 'Y.Doc',
          },
          'permission': 'view',
          'expires_at': expirationDate,
          'verified': true,
        };

        final response = Response<Map<String, dynamic>>(
          data: responseData,
          statusCode: 200,
          requestOptions: RequestOptions(path: '/api/v1/share/abc123/verify'),
        );

        when(() => mockDio.post<Map<String, dynamic>>(
          any(),
          data: any(named: 'data'),
        )).thenAnswer((_) async => response);

        // Act
        final result = await repository.verifyAccessCode('abc123', '1234');

        // Assert
        expect(result.expiresAt, DateTime.parse(expirationDate));
      });
    });

    group('deleteShareLink', () {
      test('should delete share link successfully', () async {
        // Arrange
        final response = Response<dynamic>(
          data: null,
          statusCode: 204,
          requestOptions: RequestOptions(
            path: '/api/v1/documents/doc1/share/abc123',
          ),
        );

        when(() => mockDio.delete<dynamic>(any())).thenAnswer((_) async => response);

        // Act & Assert
        await expectLater(
          repository.deleteShareLink('doc1', 'abc123'),
          completes,
        );
        verify(() => mockDio.delete<dynamic>(any())).called(1);
      });

      test('should throw NetworkError when deletion fails', () async {
        // Arrange
        final response = Response<dynamic>(
          data: null,
          statusCode: 404,
          statusMessage: 'Not Found',
          requestOptions: RequestOptions(
            path: '/api/v1/documents/doc1/share/abc123',
          ),
        );

        when(() => mockDio.delete<dynamic>(any())).thenAnswer((_) async => response);

        // Act & Assert
        await expectLater(
          repository.deleteShareLink('doc1', 'abc123'),
          throwsA(isA<ne.NetworkError>()),
        );
      });

      test('should handle server error during deletion', () async {
        // Arrange
        final response = Response<dynamic>(
          data: null,
          statusCode: 500,
          statusMessage: 'Internal Server Error',
          requestOptions: RequestOptions(
            path: '/api/v1/documents/doc1/share/abc123',
          ),
        );

        when(() => mockDio.delete<dynamic>(any())).thenAnswer((_) async => response);

        // Act & Assert
        await expectLater(
          repository.deleteShareLink('doc1', 'abc123'),
          throwsA(isA<ne.NetworkError>()),
        );
      });
    });

    group('copyShareLinkToClipboard', () {
      test('should copy share link to clipboard successfully', () async {
        // Arrange
        TestWidgetsFlutterBinding.ensureInitialized();

        final shareLink = ShareLink(
          id: 'share1',
          documentId: 'doc1',
          documentTitle: 'Test Document',
          token: 'abc123',
          requiresAccessCode: false,
          permission: 'view',
          isActive: true,
          createdAt: DateTime.parse('2025-01-01T00:00:00Z'),
          accessCount: 0,
          createdBy: 'user1',
        );

        // Act
        final result = await repository.copyShareLinkToClipboard(
          shareLink,
          'http://localhost:3000',
        );

        // Assert
        expect(result, true);
      });

      test('should generate correct share URL', () {
        // Arrange
        final shareLink = ShareLink(
          id: 'share1',
          documentId: 'doc1',
          documentTitle: 'Test Document',
          token: 'xyz789',
          requiresAccessCode: false,
          permission: 'comment',
          isActive: true,
          createdAt: DateTime.parse('2025-01-01T00:00:00Z'),
          accessCount: 5,
          createdBy: 'user2',
        );

        const baseUrl = 'https://example.com';
        const expectedUrl = 'https://example.com/share/xyz789';

        // Act
        final actualUrl = shareLink.getShareUrl(baseUrl);

        // Assert
        expect(actualUrl, expectedUrl);
      });
    });

    group('ShareLink entity methods', () {
      test('isExpired should return false when no expiration', () {
        final shareLink = ShareLink(
          id: 'share1',
          documentId: 'doc1',
          documentTitle: 'Test',
          token: 'abc123',
          requiresAccessCode: false,
          permission: 'view',
          isActive: true,
          createdAt: DateTime.now(),
          accessCount: 0,
          createdBy: 'user1',
        );

        expect(shareLink.isExpired, false);
      });

      test('isExpired should return true when expired', () {
        final shareLink = ShareLink(
          id: 'share1',
          documentId: 'doc1',
          documentTitle: 'Test',
          token: 'abc123',
          requiresAccessCode: false,
          permission: 'view',
          isActive: true,
          createdAt: DateTime.now(),
          expiresAt: DateTime.now().subtract(const Duration(days: 1)),
          accessCount: 0,
          createdBy: 'user1',
        );

        expect(shareLink.isExpired, true);
      });

      test('hasReachedMaxAccess should return false when no max set', () {
        final shareLink = ShareLink(
          id: 'share1',
          documentId: 'doc1',
          documentTitle: 'Test',
          token: 'abc123',
          requiresAccessCode: false,
          permission: 'view',
          isActive: true,
          createdAt: DateTime.now(),
          accessCount: 100,
          createdBy: 'user1',
        );

        expect(shareLink.hasReachedMaxAccess, false);
      });

      test('hasReachedMaxAccess should return true when limit reached', () {
        final shareLink = ShareLink(
          id: 'share1',
          documentId: 'doc1',
          documentTitle: 'Test',
          token: 'abc123',
          requiresAccessCode: false,
          permission: 'view',
          isActive: true,
          createdAt: DateTime.now(),
          accessCount: 10,
          maxAccessCount: 10,
          createdBy: 'user1',
        );

        expect(shareLink.hasReachedMaxAccess, true);
      });

      test('isUsable should check all conditions', () {
        final usableLink = ShareLink(
          id: 'share1',
          documentId: 'doc1',
          documentTitle: 'Test',
          token: 'abc123',
          requiresAccessCode: false,
          permission: 'view',
          isActive: true,
          createdAt: DateTime.now(),
          accessCount: 5,
          createdBy: 'user1',
        );

        expect(usableLink.isUsable, true);

        final expiredLink = ShareLink(
          id: 'share2',
          documentId: 'doc1',
          documentTitle: 'Test',
          token: 'def456',
          requiresAccessCode: false,
          permission: 'view',
          isActive: true,
          createdAt: DateTime.now(),
          expiresAt: DateTime.now().subtract(const Duration(days: 1)),
          accessCount: 0,
          createdBy: 'user1',
        );

        expect(expiredLink.isUsable, false);

        final inactiveLink = ShareLink(
          id: 'share3',
          documentId: 'doc1',
          documentTitle: 'Test',
          token: 'ghi789',
          requiresAccessCode: false,
          permission: 'view',
          isActive: false,
          createdAt: DateTime.now(),
          accessCount: 0,
          createdBy: 'user1',
        );

        expect(inactiveLink.isUsable, false);
      });
    });
  });
}
