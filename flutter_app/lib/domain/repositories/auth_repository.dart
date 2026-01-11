import 'package:miniwiki/data/models/user_entity.dart';

/// Authentication tokens returned after successful login
class AuthTokens {
  final String accessToken;
  final String refreshToken;
  final int expiresIn;
  final UserEntity user;

  const AuthTokens({
    required this.accessToken,
    required this.refreshToken,
    required this.expiresIn,
    required this.user,
  });
}

/// Result of a login attempt
class LoginResult {
  final bool success;
  final String? errorMessage;
  final AuthTokens? tokens;

  const LoginResult({
    required this.success,
    this.errorMessage,
    this.tokens,
  });

  bool get isSuccess => success && tokens != null;
}

/// Repository interface for authentication operations
abstract class AuthRepository {
  /// Register a new user account
  Future<UserEntity> register({
    required String email,
    required String password,
    required String displayName,
  });

  /// Login with email and password
  Future<AuthTokens> login({
    required String email,
    required String password,
  });

  /// Logout the current user
  Future<void> logout();

  /// Get the currently authenticated user
  Future<UserEntity> getCurrentUser();

  /// Refresh the access token
  Future<void> refreshToken();

  /// Check if user is currently authenticated
  bool isAuthenticated();

  /// Get the current access token
  String? getAccessToken();
}
