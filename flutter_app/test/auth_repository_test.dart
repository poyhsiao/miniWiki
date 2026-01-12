import 'package:flutter_test/flutter_test.dart';
import 'package:mocktail/mocktail.dart';
import 'package:dio/dio.dart';
import 'package:miniwiki/domain/repositories/auth_repository.dart';
import 'package:miniwiki/data/repositories/auth_repository_impl.dart';
import 'package:miniwiki/core/network/api_client.dart';

class MockApiClient extends Mock implements ApiClient {}

class MockResponse extends Mock implements Response {}

void main() {
  group('AuthRepository Tests', () {
    late ApiClient apiClient;
    late AuthRepository authRepository;

    setUp(() {
      apiClient = MockApiClient();
      authRepository = AuthRepositoryImpl(apiClient);
    });

    group('login', () {
      test('login with valid credentials returns auth tokens', () async {
        final response = MockResponse();
        when(() => response.statusCode).thenReturn(200);
        when(() => response.data).thenReturn({
          'user': {
            'id': 'test-uuid',
            'email': 'test@example.com',
            'display_name': 'Test User',
            'avatar_url': null,
            'is_email_verified': true,
          },
          'access_token': 'test-access-token',
          'refresh_token': 'test-refresh-token',
          'expires_in': 3600,
        });
        when(() => apiClient.post('/auth/login', data: any(named: 'data')))
            .thenAnswer((_) async => response);

        final result = await authRepository.login(
          email: 'test@example.com',
          password: 'password123',
        );

        expect(result.user.email, 'test@example.com');
        expect(result.user.displayName, 'Test User');
        expect(result.accessToken, 'test-access-token');
        expect(result.refreshToken, 'test-refresh-token');
        expect(result.expiresIn, 3600);
      });

      test('login with invalid credentials throws error', () async {
        final response = MockResponse();
        when(() => response.statusCode).thenReturn(401);
        when(() => response.data).thenReturn({
          'error': 'AUTH_INVALID_CREDENTIALS',
          'message': 'Invalid email or password',
        });
        when(() => apiClient.post('/auth/login', data: any(named: 'data')))
            .thenAnswer((_) async => response);

        expect(
          () => authRepository.login(
            email: 'test@example.com',
            password: 'wrongpassword',
          ),
          throwsA(isA<DioException>()),
        );
      });
    });

    group('register', () {
      test('register with valid data returns user entity', () async {
        final response = MockResponse();
        when(() => response.statusCode).thenReturn(201);
        when(() => response.data).thenReturn({
          'user': {
            'id': 'new-uuid',
            'email': 'new@example.com',
            'display_name': 'New User',
            'avatar_url': null,
            'is_email_verified': false,
          },
          'message': 'Registration successful',
        });
        when(() => apiClient.post('/auth/register', data: any(named: 'data')))
            .thenAnswer((_) async => response);

        final result = await authRepository.register(
          email: 'new@example.com',
          password: 'SecurePass123',
          displayName: 'New User',
        );

        expect(result.email, 'new@example.com');
        expect(result.displayName, 'New User');
        expect(result.isEmailVerified, false);
      });

      test('register with existing email throws conflict error', () async {
        final response = MockResponse();
        when(() => response.statusCode).thenReturn(409);
        when(() => response.data).thenReturn({
          'error': 'AUTH_EMAIL_EXISTS',
          'message': 'Email is already registered',
        });
        when(() => apiClient.post('/auth/register', data: any(named: 'data')))
            .thenAnswer((_) async => response);

        expect(
          () => authRepository.register(
            email: 'existing@example.com',
            password: 'SecurePass123',
            displayName: 'User',
          ),
          throwsA(isA<DioException>()),
        );
      });
    });

    group('logout', () {
      test('logout succeeds without errors', () async {
        final response = MockResponse();
        when(() => response.statusCode).thenReturn(200);
        when(() => apiClient.post('/auth/logout'))
            .thenAnswer((_) async => response);

        await expectLater(authRepository.logout(), completes);
      });
    });

    group('refreshToken', () {
      test('refresh token completes without error', () async {
        final response = MockResponse();
        when(() => response.statusCode).thenReturn(200);
        when(() => response.data).thenReturn({
          'access_token': 'new-access-token',
          'refresh_token': 'new-refresh-token',
          'expires_in': 3600,
        });
        when(() => apiClient.post('/auth/refresh', data: any(named: 'data')))
            .thenAnswer((_) async => response);

        await expectLater(authRepository.refreshToken(), completes);
      });
    });

    group('getCurrentUser', () {
      test('getCurrentUser returns user entity', () async {
        final response = MockResponse();
        when(() => response.statusCode).thenReturn(200);
        when(() => response.data).thenReturn({
          'id': 'user-uuid',
          'email': 'user@example.com',
          'display_name': 'Current User',
          'avatar_url': 'https://example.com/avatar.png',
          'is_email_verified': true,
        });
        when(() => apiClient.get('/auth/me')).thenAnswer((_) async => response);

        final result = await authRepository.getCurrentUser();

        expect(result.uuid, 'user-uuid');
        expect(result.email, 'user@example.com');
        expect(result.displayName, 'Current User');
        expect(result.avatarUrl, 'https://example.com/avatar.png');
        expect(result.isEmailVerified, true);
      });
    });

    group('isAuthenticated', () {
      test('isAuthenticated returns false when not logged in', () {
        expect(authRepository.isAuthenticated(), false);
      });

      test('isAuthenticated returns true after login', () async {
        final response = MockResponse();
        when(() => response.statusCode).thenReturn(200);
        when(() => response.data).thenReturn({
          'user': {
            'id': 'test-uuid',
            'email': 'test@example.com',
            'display_name': 'Test User',
            'avatar_url': null,
            'is_email_verified': true,
          },
          'access_token': 'test-access-token',
          'refresh_token': 'test-refresh-token',
          'expires_in': 3600,
        });
        when(() => apiClient.post('/auth/login', data: any(named: 'data')))
            .thenAnswer((_) async => response);

        await authRepository.login(
          email: 'test@example.com',
          password: 'password123',
        );

        expect(authRepository.isAuthenticated(), true);
      });
    });

    group('getAccessToken', () {
      test('getAccessToken returns null when not logged in', () {
        expect(authRepository.getAccessToken(), null);
      });

      test('getAccessToken returns token after login', () async {
        final response = MockResponse();
        when(() => response.statusCode).thenReturn(200);
        when(() => response.data).thenReturn({
          'user': {
            'id': 'test-uuid',
            'email': 'test@example.com',
            'display_name': 'Test User',
            'avatar_url': null,
            'is_email_verified': true,
          },
          'access_token': 'test-access-token',
          'refresh_token': 'test-refresh-token',
          'expires_in': 3600,
        });
        when(() => apiClient.post('/auth/login', data: any(named: 'data')))
            .thenAnswer((_) async => response);

        await authRepository.login(
          email: 'test@example.com',
          password: 'password123',
        );

        expect(authRepository.getAccessToken(), 'test-access-token');
      });
    });
  });
}
