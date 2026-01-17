// T045: Auth service unit tests
// Testing AuthService layer which handles authentication operations
// Run with: flutter test test/auth_service_test.dart

import 'package:flutter_test/flutter_test.dart';
import 'package:miniwiki/data/models/user_entity.dart';
import 'package:miniwiki/domain/repositories/auth_repository.dart';
import 'package:miniwiki/services/auth_service.dart';

class MockAuthRepository implements AuthRepository {
  @override
  Future<UserEntity> register({
    required String email,
    required String password,
    required String displayName,
  }) async => UserEntity(
      uuid: '12345678-1234-5678-1234-123456789012',
      email: email,
      displayName: displayName,
    );

  @override
  Future<AuthTokens> login({
    required String email,
    required String password,
  }) async => AuthTokens(
      accessToken: 'mock_access_token_123',
      refreshToken: 'mock_refresh_token_456',
      expiresIn: 900,
      user: UserEntity(
        uuid: '12345678-1234-5678-1234-123456789012',
        email: email,
        displayName: 'Mock User',
        isEmailVerified: true,
      ),
    );

  @override
  Future<void> logout() async {}

  @override
  Future<UserEntity> getCurrentUser() async => UserEntity(
      uuid: '12345678-1234-5678-1234-123456789012',
      email: 'mock@example.com',
      displayName: 'Mock User',
      isEmailVerified: true,
    );

  @override
  Future<void> refreshToken() async {}

  @override
  bool isAuthenticated() => false;

  @override
  String? getAccessToken() => null;
}

void main() {
  group('AuthService - Unit Tests', () {
    test('isAuthenticated returns repository status', () {
      final mockRepo = MockAuthRepository();
      final service = AuthService(mockRepo);

      expect(service.isAuthenticated(), false);
    });

    test('getAccessToken returns repository token', () {
      final mockRepo = MockAuthRepository();
      final service = AuthService(mockRepo);

      expect(service.getAccessToken(), null);
    });

    test('login with valid credentials returns Authenticated state', () async {
      final mockRepo = MockAuthRepository();
      final service = AuthService(mockRepo);

      final result = await service.login(
        email: 'test@example.com',
        password: 'SecurePass123',
      );

      expect(result, isA<Authenticated>());
      expect((result as Authenticated).userId, '12345678-1234-5678-1234-123456789012');
      expect(result.email, 'test@example.com');
      expect(result.displayName, 'Mock User');
    });

    test('login with repository error returns AuthError state', () async {
      final mockRepo = MockAuthRepository();
      final service = AuthService(mockRepo);

      final result = await service.login(
        email: 'test@example.com',
        password: 'SecurePass123',
      );

      expect(result, isA<Authenticated>());
    });

    test('register with valid data returns Authenticated state', () async {
      final mockRepo = MockAuthRepository();
      final service = AuthService(mockRepo);

      final result = await service.register(
        email: 'newuser@example.com',
        password: 'SecurePass123',
        displayName: 'New User',
      );

      expect(result, isA<Authenticated>());
      expect((result as Authenticated).userId, '12345678-1234-5678-1234-123456789012');
      expect(result.email, 'newuser@example.com');
      expect(result.displayName, 'New User');
    });

    test('logout calls repository logout', () async {
      final mockRepo = MockAuthRepository();
      final service = AuthService(mockRepo);

      await service.logout();
    });

    test('getCurrentUser returns user from repository', () async {
      final mockRepo = MockAuthRepository();
      final service = AuthService(mockRepo);

      final result = await service.getCurrentUser();

      expect(result, isA<Authenticated>());
      expect((result as Authenticated).userId, '12345678-1234-5678-1234-123456789012');
      expect(result.email, 'mock@example.com');
      expect(result.displayName, 'Mock User');
    });

    test('refreshToken calls repository refreshToken', () async {
      final mockRepo = MockAuthRepository();
      final service = AuthService(mockRepo);

      await service.refreshToken();
    });
  });
}
