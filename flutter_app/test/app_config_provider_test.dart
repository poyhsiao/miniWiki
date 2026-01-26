import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:miniwiki/core/config/app_config_provider.dart';

void main() {
  group('AppConfigProvider Tests', () {
    test('AppConfigProvider returns default API URL', () {
      // Arrange
      final container = ProviderContainer();

      // Act
      final apiUrl = container.read(appConfigProvider);

      // Assert
      expect(apiUrl, 'http://localhost:8080/api/v1');

      // Cleanup
      container.dispose();
    });

    test('AppConfigProvider wsUrl returns WebSocket URL', () {
      // Arrange
      final container = ProviderContainer();

      // Act
      final wsUrl = container.read(appConfigProvider.notifier).wsUrl;

      // Assert
      expect(wsUrl, 'ws://localhost:8080/ws');

      // Cleanup
      container.dispose();
    });

    test('AppConfigProvider uses correct HTTP protocol', () {
      // Arrange
      final container = ProviderContainer();

      // Act
      final apiUrl = container.read(appConfigProvider);

      // Assert
      expect(apiUrl, startsWith('http://'));

      // Cleanup
      container.dispose();
    });

    test('AppConfigProvider wsUrl uses correct WS protocol', () {
      // Arrange
      final container = ProviderContainer();

      // Act
      final wsUrl = container.read(appConfigProvider.notifier).wsUrl;

      // Assert
      expect(wsUrl, startsWith('ws://'));

      // Cleanup
      container.dispose();
    });

    test('AppConfigProvider API URL includes version', () {
      // Arrange
      final container = ProviderContainer();

      // Act
      final apiUrl = container.read(appConfigProvider);

      // Assert
      expect(apiUrl, contains('/api/v1'));

      // Cleanup
      container.dispose();
    });

    test('AppConfigProvider WS URL includes ws path', () {
      // Arrange
      final container = ProviderContainer();

      // Act
      final wsUrl = container.read(appConfigProvider.notifier).wsUrl;

      // Assert
      expect(wsUrl, contains('/ws'));

      // Cleanup
      container.dispose();
    });

    test('AppConfigProvider both URLs use same host', () {
      // Arrange
      final container = ProviderContainer();

      // Act
      final apiUrl = container.read(appConfigProvider);
      final wsUrl = container.read(appConfigProvider.notifier).wsUrl;

      // Assert - Both should use localhost:8080
      expect(apiUrl, contains('localhost:8080'));
      expect(wsUrl, contains('localhost:8080'));

      // Cleanup
      container.dispose();
    });

    test('AppConfigProvider notifier can be accessed', () {
      // Arrange
      final container = ProviderContainer();

      // Act
      final notifier = container.read(appConfigProvider.notifier);

      // Assert
      expect(notifier, isNotNull);
      expect(notifier, isA<AppConfig>());

      // Cleanup
      container.dispose();
    });

    test('AppConfigProvider multiple reads return same value', () {
      // Arrange
      final container = ProviderContainer();

      // Act
      final value1 = container.read(appConfigProvider);
      final value2 = container.read(appConfigProvider);

      // Assert
      expect(value1, equals(value2));
      expect(value1, 'http://localhost:8080/api/v1');

      // Cleanup
      container.dispose();
    });
  });
}
