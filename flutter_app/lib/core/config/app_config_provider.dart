import 'package:riverpod_annotation/riverpod_annotation.dart';

part 'app_config_provider.g.dart';

@riverpod
class AppConfig extends _$AppConfig {
  @override
  String build() {
    return 'http://localhost:8080/api/v1';
  }

  String get wsUrl => 'ws://localhost:8080/ws';
}
