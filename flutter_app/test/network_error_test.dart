import 'package:flutter_test/flutter_test.dart';
import 'package:miniwiki/core/network/network_error.dart';

void main() {
  group('NetworkError Tests', () {
    test('NetworkError can be created with message only', () {
      // Arrange & Act
      final error = NetworkError('Something went wrong');

      // Assert
      expect(error.message, 'Something went wrong');
      expect(error.statusCode, isNull);
    });

    test('NetworkError can be created with message and status code', () {
      // Arrange & Act
      final error = NetworkError('Not found', 404);

      // Assert
      expect(error.message, 'Not found');
      expect(error.statusCode, 404);
    });

    test('NetworkError toString without status code', () {
      // Arrange
      final error = NetworkError('Connection failed');

      // Act
      final string = error.toString();

      // Assert
      expect(string, 'NetworkError: Connection failed');
    });

    test('NetworkError toString with status code', () {
      // Arrange
      final error = NetworkError('Unauthorized', 401);

      // Act
      final string = error.toString();

      // Assert
      expect(string, 'NetworkError: Unauthorized (Status: 401)');
    });

    test('NetworkError fromJson with message', () {
      // Arrange
      final json = {'message': 'Invalid request'};

      // Act
      final error = NetworkError.fromJson(json);

      // Assert
      expect(error.message, 'Invalid request');
      expect(error.statusCode, isNull);
    });

    test('NetworkError fromJson with error field', () {
      // Arrange
      final json = {'error': 'Server error'};

      // Act
      final error = NetworkError.fromJson(json);

      // Assert
      expect(error.message, 'Server error');
    });

    test('NetworkError fromJson with message and status code', () {
      // Arrange
      final json = {'message': 'Forbidden'};

      // Act
      final error = NetworkError.fromJson(json, 403);

      // Assert
      expect(error.message, 'Forbidden');
      expect(error.statusCode, 403);
    });

    test('NetworkError fromJson with empty json uses default message', () {
      // Arrange
      final json = <String, dynamic>{};

      // Act
      final error = NetworkError.fromJson(json);

      // Assert
      expect(error.message, 'Request failed');
    });

    test('NetworkError fromJson prefers message over error field', () {
      // Arrange
      final json = {
        'message': 'Custom message',
        'error': 'Default error',
      };

      // Act
      final error = NetworkError.fromJson(json);

      // Assert
      expect(error.message, 'Custom message');
    });

    test('NetworkError with common status codes', () {
      // Arrange & Act
      final badRequest = NetworkError('Bad request', 400);
      final unauthorized = NetworkError('Unauthorized', 401);
      final forbidden = NetworkError('Forbidden', 403);
      final notFound = NetworkError('Not found', 404);
      final serverError = NetworkError('Server error', 500);

      // Assert
      expect(badRequest.statusCode, 400);
      expect(unauthorized.statusCode, 401);
      expect(forbidden.statusCode, 403);
      expect(notFound.statusCode, 404);
      expect(serverError.statusCode, 500);
    });

    test('NetworkError implements Exception', () {
      // Arrange
      final error = NetworkError('Test error');

      // Assert
      expect(error, isA<Exception>());
    });

    test('NetworkError with empty message', () {
      // Arrange & Act
      final error = NetworkError('');

      // Assert
      expect(error.message, '');
      expect(error.toString(), 'NetworkError: ');
    });

    test('NetworkError with null status code defaults to null', () {
      // Arrange & Act
      final error = NetworkError('Test');

      // Assert
      expect(error.statusCode, isNull);
    });

    test('NetworkError fromJson with null status code', () {
      // Arrange
      final json = {'message': 'Test'};

      // Act
      final error = NetworkError.fromJson(json);

      // Assert
      expect(error.statusCode, isNull);
    });

    test('NetworkError toString handles special characters in message', () {
      // Arrange
      final error = NetworkError('Error: "test" with \'quotes\' & symbols!@#\$%');

      // Act
      final string = error.toString();

      // Assert
      expect(string, contains('Error: "test"'));
      expect(string, contains('NetworkError:'));
    });

    test('NetworkError with zero status code', () {
      // Arrange & Act
      final error = NetworkError('Network error', 0);

      // Assert
      expect(error.statusCode, 0);
      expect(error.toString(), contains('(Status: 0)'));
    });
  });
}
