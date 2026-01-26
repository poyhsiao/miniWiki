import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:miniwiki/core/config/shared_preferences_provider.dart';
import 'package:shared_preferences/shared_preferences.dart';

void main() {
  // Setup mock preferences
  setUpAll(() async {
    SharedPreferences.setMockInitialValues({});
  });

  group('SharedPreferencesProvider Tests', () {
    test('SharedPreferencesProvider can be created', () {
      // Arrange
      final container = ProviderContainer();

      // Act
      final provider = sharedPreferencesNotifierProvider;

      // Assert
      expect(provider, isNotNull);

      // Cleanup
      container.dispose();
    });

    test('SharedPreferencesProvider returns Future<SharedPreferences>', () async {
      // Arrange
      final container = ProviderContainer();

      // Act
      final prefsFuture = container.read(sharedPreferencesNotifierProvider.future);

      // Assert
      expect(prefsFuture, isA<Future<SharedPreferences>>());

      // Cleanup
      container.dispose();
    });

    test('SharedPreferencesProvider notifier can be accessed', () {
      // Arrange
      final container = ProviderContainer();

      // Act
      final notifier = container.read(sharedPreferencesNotifierProvider.notifier);

      // Assert
      expect(notifier, isNotNull);
      expect(notifier, isA<SharedPreferencesNotifier>());

      // Cleanup
      container.dispose();
    });

    test('SharedPreferencesProvider build returns SharedPreferences instance', () async {
      // Arrange
      final container = ProviderContainer();
      final notifier = container.read(sharedPreferencesNotifierProvider.notifier);

      // Act
      final prefs = await notifier.build();

      // Assert
      expect(prefs, isA<SharedPreferences>());

      // Cleanup
      container.dispose();
    });

    test('SharedPreferencesProvider uses SharedPreferences.getInstance', () async {
      // Arrange
      final container = ProviderContainer();

      // Act
      final prefs = await container.read(sharedPreferencesNotifierProvider.future);

      // Assert
      expect(prefs, isNotNull);
      expect(prefs, isA<SharedPreferences>());

      // Cleanup
      container.dispose();
    });

    test('SharedPreferencesProvider can be read multiple times', () async {
      // Arrange
      final container = ProviderContainer();

      // Act
      final prefs1 = await container.read(sharedPreferencesNotifierProvider.future);
      final prefs2 = await container.read(sharedPreferencesNotifierProvider.future);

      // Assert
      expect(prefs1, equals(prefs2));

      // Cleanup
      container.dispose();
    });

    test('SharedPreferencesProvider notifier build method is async', () {
      // Arrange
      final container = ProviderContainer();
      final notifier = container.read(sharedPreferencesNotifierProvider.notifier);

      // Act
      final result = notifier.build();

      // Assert
      expect(result, isA<Future<SharedPreferences>>());

      // Cleanup
      container.dispose();
    });
  });

  group('SharedPreferencesNotifier Tests', () {
    test('SharedPreferencesNotifier extends build method', () async {
      // Arrange
      final container = ProviderContainer();
      final notifier = container.read(sharedPreferencesNotifierProvider.notifier);

      // Act
      final prefs = await notifier.build();

      // Assert
      expect(prefs, isNotNull);

      // Cleanup
      container.dispose();
    });

    test('SharedPreferencesNotifier can store and retrieve values', () async {
      // Arrange
      final container = ProviderContainer();
      final prefs = await container.read(sharedPreferencesNotifierProvider.future);

      // Act
      await prefs.setString('test_key', 'test_value');
      final result = prefs.getString('test_key');

      // Assert
      expect(result, 'test_value');

      // Cleanup
      container.dispose();
    });
  });
}
