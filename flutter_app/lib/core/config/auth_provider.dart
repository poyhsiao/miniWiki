import 'package:riverpod_annotation/riverpod_annotation.dart';

part 'auth_provider.g.dart';

sealed class AuthState {
  const AuthState();
  bool get isAuthenticated => this is Authenticated;
  bool get isLoading => this is Loading;
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

  const Authenticated({
    required this.userId,
    required this.email,
    required this.displayName,
  });
}

@riverpod
class Auth extends _$Auth {
  @override
  AuthState build() => const Unauthenticated();

  Future<void> login(String email, String password) async {
    state = const Loading();
    // TODO: Implement login logic
    state = Authenticated(
      userId: 'uuid',
      email: email,
      displayName: 'User',
    );
  }

  Future<void> register({
    required String email,
    required String password,
    required String displayName,
  }) async {
    state = const Loading();
    // TODO: Implement register logic
    state = Authenticated(
      userId: 'uuid',
      email: email,
      displayName: displayName,
    );
  }

  Future<void> logout() async {
    state = const Unauthenticated();
  }
}
