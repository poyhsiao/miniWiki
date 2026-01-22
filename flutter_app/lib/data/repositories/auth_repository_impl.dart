import 'package:miniwiki/core/network/api_client.dart';
import 'package:miniwiki/data/models/user_entity.dart';
import 'package:miniwiki/domain/repositories/auth_repository.dart';
import 'package:riverpod/riverpod.dart';

class AuthRepositoryImpl implements AuthRepository {
  final ApiClient _apiClient;
  String? _accessToken;

  AuthRepositoryImpl(this._apiClient);

  @override
  Future<UserEntity> register({
    required String email,
    required String password,
    required String displayName,
  }) async {
    final response = await _apiClient.post('/auth/register', data: {
      'email': email,
      'password': password,
      'display_name': displayName,
    });

    final data = response.data as Map<String, dynamic>;
    final userData = data['user'] as Map<String, dynamic>;

    return UserEntity(
      uuid: _requireString(userData['id'], 'id'),
      email: (userData['email'] as String?) ?? '',
      displayName: (userData['display_name'] as String?) ?? '',
      avatarUrl: userData['avatar_url'] as String?,
      isEmailVerified: (userData['is_email_verified'] as bool?) ?? false,
    );
  }

  @override
  Future<AuthTokens> login({
    required String email,
    required String password,
  }) async {
    final response = await _apiClient.post('/auth/login', data: {
      'email': email,
      'password': password,
    });

    final data = response.data as Map<String, dynamic>;
    final userData = data['user'] as Map<String, dynamic>;

    _accessToken = data['access_token'] as String;
    _apiClient.setAuthToken(_accessToken!);

    return AuthTokens(
      accessToken: data['access_token'] as String,
      refreshToken: data['refresh_token'] as String,
      expiresIn: data['expires_in'] as int,
      user: UserEntity(
        uuid: _requireString(userData['id'], 'id'),
        email: (userData['email'] as String?) ?? '',
        displayName: (userData['display_name'] as String?) ?? '',
        avatarUrl: userData['avatar_url'] as String?,
        isEmailVerified: (userData['is_email_verified'] as bool?) ?? false,
      ),
    );
  }

  @override
  Future<void> logout() async {
    await _apiClient.post('/auth/logout');
    _apiClient.clearAuthToken();
    _accessToken = null;
  }

  @override
  Future<UserEntity> getCurrentUser() async {
    final response = await _apiClient.get('/auth/me');
    final data = response.data as Map<String, dynamic>;

    return UserEntity(
      uuid: _requireString(data['id'], 'id'),
      email: (data['email'] as String?) ?? '',
      displayName: (data['display_name'] as String?) ?? '',
      avatarUrl: data['avatar_url'] as String?,
      isEmailVerified: (data['is_email_verified'] as bool?) ?? false,
    );
  }

  String _requireString(value, String fieldName) {
    if (value == null || (value is String && value.isEmpty)) {
      throw StateError('Missing required field: $fieldName');
    }
    if (value is! String) {
      throw StateError('Unexpected type for field: $fieldName. Expected String, got ${value.runtimeType}');
    }
    return value;
  }

  @override
  Future<void> refreshToken() async {
    // Token refresh logic
  }

  @override
  bool isAuthenticated() => _accessToken != null;

  @override
  String? getAccessToken() => _accessToken;
}

final authRepositoryProvider = Provider<AuthRepository>((ref) {
  final apiClient = ref.watch(apiClientProvider);
  return AuthRepositoryImpl(apiClient);
});
