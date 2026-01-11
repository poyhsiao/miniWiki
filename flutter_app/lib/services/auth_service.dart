import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:miniwiki/domain/repositories/auth_repository.dart';
import 'package:miniwiki/data/repositories/auth_repository_impl.dart';

/// Authentication state sealed class
sealed class AuthState {
  const AuthState();

  bool get isAuthenticated => this is Authenticated;
  bool get isLoading => this is Loading;
  bool get isUnauthenticated => this is Unauthenticated;
}

class Unauthenticated extends AuthState {
  const Unauthenticated();
}

class Loading extends AuthState {
  const Loading();
}

class Authenticated extends AuthState {
  final String userId;
  final String email;
  final String displayName;
  final String? avatarUrl;

  const Authenticated({
    required this.userId,
    required this.email,
    required this.displayName,
    this.avatarUrl,
  });
}

class AuthError extends AuthState {
  final String message;

  const AuthError(this.message);
}

/// Authentication service that handles all auth-related operations
class AuthService {
  final AuthRepository _repository;

  AuthService(this._repository);

  /// Check if user is currently authenticated
  bool isAuthenticated() => _repository.isAuthenticated();

  /// Get the current access token
  String? getAccessToken() => _repository.getAccessToken();

  /// Login with email and password
  Future<AuthState> login({
    required String email,
    required String password,
  }) async {
    try {
      final tokens = await _repository.login(
        email: email,
        password: password,
      );

      return Authenticated(
        userId: tokens.user.uuid,
        email: tokens.user.email,
        displayName: tokens.user.displayName,
        avatarUrl: tokens.user.avatarUrl,
      );
    } catch (e) {
      return AuthError(e.toString());
    }
  }

  /// Register a new user
  Future<AuthState> register({
    required String email,
    required String password,
    required String displayName,
  }) async {
    try {
      final user = await _repository.register(
        email: email,
        password: password,
        displayName: displayName,
      );

      return Authenticated(
        userId: user.uuid,
        email: user.email,
        displayName: user.displayName,
        avatarUrl: user.avatarUrl,
      );
    } catch (e) {
      return AuthError(e.toString());
    }
  }

  /// Logout the current user
  Future<void> logout() async {
    await _repository.logout();
  }

  /// Get the current user
  Future<AuthState> getCurrentUser() async {
    try {
      final user = await _repository.getCurrentUser();

      return Authenticated(
        userId: user.uuid,
        email: user.email,
        displayName: user.displayName,
        avatarUrl: user.avatarUrl,
      );
    } catch (e) {
      return const Unauthenticated();
    }
  }

  /// Refresh the access token
  Future<void> refreshToken() async {
    await _repository.refreshToken();
  }
}

/// Provider for the AuthService
final authServiceProvider = Provider<AuthService>((ref) {
  final repository = ref.watch(authRepositoryProvider);
  return AuthService(repository);
});
